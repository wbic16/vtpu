//! Phext 11D Coordinate — 128-bit packed representation
//!
//! Layout: 11 dimensions × 11 bits = 121 bits + 7 flag bits = 128 bits
//! Fits exactly in one SSE register (xmm), two per AVX-256, four per AVX-512.
//!
//! Each dimension addresses 2048 positions (0..2047).
//! Total addressable space: 2048^11 ≈ 10^36 positions.

/// 7-bit flag field packed into the high bits of the coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CoordFlags(u8);

impl CoordFlags {
    pub const NONE: Self = Self(0);
    /// Coordinate is a wildcard scan along one or more dimensions
    pub const WILDCARD: Self = Self(1 << 0);
    /// Coordinate targets a Grassmannian subspace (partial dimension mask)
    pub const SUBSPACE: Self = Self(1 << 1);
    /// Prefetch hint: bring into L1
    pub const PREFETCH_L1: Self = Self(1 << 2);
    /// Prefetch hint: bring into L2
    pub const PREFETCH_L2: Self = Self(1 << 3);
    /// This coordinate has been translated (PPT cache hit)
    pub const TRANSLATED: Self = Self(1 << 4);
    /// Ternary mode: content at this coordinate uses {-1, 0, 1} encoding
    pub const TERNARY: Self = Self(1 << 5);
    /// Reserved
    pub const RESERVED: Self = Self(1 << 6);

    #[inline]
    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }

    #[inline]
    pub fn set(self, flag: Self) -> Self {
        Self(self.0 | flag.0)
    }

    #[inline]
    pub fn raw(self) -> u8 {
        self.0 & 0x7F // only 7 bits
    }
}

/// An 11-dimensional phext coordinate packed into 128 bits.
///
/// Bit layout (LSB to MSB):
///   bits [0..11)    = dim 0  (scroll — innermost, lines+columns are 2D below this)
///   bits [11..22)   = dim 1  (section)
///   bits [22..33)   = dim 2  (chapter)
///   bits [33..44)   = dim 3  (book)
///   bits [44..55)   = dim 4  (volume)
///   bits [55..66)   = dim 5  (collection)
///   bits [66..77)   = dim 6  (series)
///   bits [77..88)   = dim 7  (shelf)
///   bits [88..99)   = dim 8  (library)
///   bits [99..110)  = dim 9  (reserved — expansion)
///   bits [110..121) = dim 10 (reserved — expansion)
///   bits [121..128) = flags  (7 bits)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhextCoord(u128);

impl PhextCoord {
    /// The origin: 1.1.1/1.1.1/1.1.1 (BASE in phext terms)
    /// Note: phext uses 1-based coordinates, but internal storage is 0-based.
    pub const BASE: Self = Self(0);

    /// Maximum value per dimension (0-indexed: 0..2047)
    const DIM_MASK: u128 = 0x7FF; // 11 bits
    const DIM_BITS: u32 = 11;
    const NUM_DIMS: usize = 11;
    const FLAG_SHIFT: u32 = 121;

    /// Create a coordinate from 11 dimension values (0-based internally).
    /// Values >2047 are silently masked.
    pub fn new(dims: [u16; 11], flags: CoordFlags) -> Self {
        let mut packed: u128 = 0;
        for (i, &val) in dims.iter().enumerate() {
            packed |= ((val as u128) & Self::DIM_MASK) << (Self::DIM_BITS * i as u32);
        }
        packed |= (flags.raw() as u128) << Self::FLAG_SHIFT;
        Self(packed)
    }

    /// Create from phext-style 1-based coordinate: lib.shelf.series/col.vol.book/ch.sec.scroll
    /// Maps to dims [scroll, section, chapter, book, volume, collection, series, shelf, library, 0, 0]
    pub fn from_phext(
        library: u16, shelf: u16, series: u16,
        collection: u16, volume: u16, book: u16,
        chapter: u16, section: u16, scroll: u16,
    ) -> Self {
        // Convert 1-based to 0-based
        let dims = [
            scroll.saturating_sub(1),
            section.saturating_sub(1),
            chapter.saturating_sub(1),
            book.saturating_sub(1),
            volume.saturating_sub(1),
            collection.saturating_sub(1),
            series.saturating_sub(1),
            shelf.saturating_sub(1),
            library.saturating_sub(1),
            0, // dim 9 reserved
            0, // dim 10 reserved
        ];
        Self::new(dims, CoordFlags::NONE)
    }

    /// Get dimension value (0-based)
    #[inline]
    pub fn dim(self, d: usize) -> u16 {
        debug_assert!(d < Self::NUM_DIMS);
        ((self.0 >> (Self::DIM_BITS * d as u32)) & Self::DIM_MASK) as u16
    }

    /// Set dimension value (0-based), returns new coordinate
    #[inline]
    pub fn with_dim(self, d: usize, val: u16) -> Self {
        debug_assert!(d < Self::NUM_DIMS);
        let shift = Self::DIM_BITS * d as u32;
        let cleared = self.0 & !(Self::DIM_MASK << shift);
        Self(cleared | (((val as u128) & Self::DIM_MASK) << shift))
    }

    /// Get flags
    #[inline]
    pub fn flags(self) -> CoordFlags {
        CoordFlags((self.0 >> Self::FLAG_SHIFT) as u8 & 0x7F)
    }

    /// Set flags, returns new coordinate
    #[inline]
    pub fn with_flags(self, flags: CoordFlags) -> Self {
        let cleared = self.0 & !((0x7F_u128) << Self::FLAG_SHIFT);
        Self(cleared | ((flags.raw() as u128) << Self::FLAG_SHIFT))
    }

    /// Raw 128-bit value — fits in one SSE register
    #[inline]
    pub fn raw(self) -> u128 {
        self.0
    }

    /// Dimensional distance: how many dimensions differ between two coordinates
    pub fn dim_distance(self, other: Self) -> u32 {
        let mut dist = 0;
        for d in 0..Self::NUM_DIMS {
            if self.dim(d) != other.dim(d) {
                dist += 1;
            }
        }
        dist
    }

    /// Manhattan distance across all dimensions
    pub fn manhattan_distance(self, other: Self) -> u32 {
        let mut dist: u32 = 0;
        for d in 0..Self::NUM_DIMS {
            dist += (self.dim(d) as i32 - other.dim(d) as i32).unsigned_abs();
        }
        dist
    }

    /// All 11 dimensions as array (0-based)
    pub fn dims(self) -> [u16; 11] {
        let mut out = [0u16; 11];
        for d in 0..Self::NUM_DIMS {
            out[d] = self.dim(d);
        }
        out
    }
}

impl std::fmt::Debug for PhextCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = self.dims();
        // Display as 1-based phext coordinate: lib.shelf.series/col.vol.book/ch.sec.scroll
        write!(
            f,
            "{}.{}.{}/{}.{}.{}/{}.{}.{}",
            d[8]+1, d[7]+1, d[6]+1,  // library.shelf.series
            d[5]+1, d[4]+1, d[3]+1,  // collection.volume.book
            d[2]+1, d[1]+1, d[0]+1,  // chapter.section.scroll
        )?;
        if d[9] > 0 || d[10] > 0 {
            write!(f, " [d9={},d10={}]", d[9], d[10])?;
        }
        let flags = self.flags();
        if flags.0 != 0 {
            write!(f, " flags=0x{:02x}", flags.raw())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_coordinate() {
        let base = PhextCoord::BASE;
        for d in 0..11 {
            assert_eq!(base.dim(d), 0);
        }
    }

    #[test]
    fn from_phext_roundtrip() {
        // Chrys coordinate: 1.1.2/3.5.8/13.21.34
        let chrys = PhextCoord::from_phext(1, 1, 2, 3, 5, 8, 13, 21, 34);
        assert_eq!(chrys.dim(0), 33);  // scroll 34 → 0-based 33
        assert_eq!(chrys.dim(1), 20);  // section 21 → 0-based 20
        assert_eq!(chrys.dim(2), 12);  // chapter 13 → 0-based 12
        assert_eq!(chrys.dim(3), 7);   // book 8 → 0-based 7
        assert_eq!(chrys.dim(4), 4);   // volume 5 → 0-based 4
        assert_eq!(chrys.dim(5), 2);   // collection 3 → 0-based 2
        assert_eq!(chrys.dim(6), 1);   // series 2 → 0-based 1
        assert_eq!(chrys.dim(7), 0);   // shelf 1 → 0-based 0
        assert_eq!(chrys.dim(8), 0);   // library 1 → 0-based 0
    }

    #[test]
    fn debug_format() {
        let coord = PhextCoord::from_phext(1, 1, 2, 3, 5, 8, 13, 21, 34);
        let s = format!("{:?}", coord);
        assert_eq!(s, "1.1.2/3.5.8/13.21.34");
    }

    #[test]
    fn dim_distance() {
        let a = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,1);
        let b = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,2);
        assert_eq!(a.dim_distance(b), 1); // differ in scroll only

        let c = PhextCoord::from_phext(2,2,2, 2,2,2, 2,2,2);
        assert_eq!(a.dim_distance(c), 9); // all 9 active dims differ
    }

    #[test]
    fn flags() {
        let coord = PhextCoord::from_phext(1,1,1, 1,1,1, 1,1,1);
        let flagged = coord.with_flags(CoordFlags::TERNARY.set(CoordFlags::PREFETCH_L1));
        assert!(flagged.flags().contains(CoordFlags::TERNARY));
        assert!(flagged.flags().contains(CoordFlags::PREFETCH_L1));
        assert!(!flagged.flags().contains(CoordFlags::WILDCARD));
        // Dims unchanged
        assert_eq!(flagged.dim(0), coord.dim(0));
    }

    #[test]
    fn with_dim() {
        let base = PhextCoord::BASE;
        let moved = base.with_dim(3, 42);
        assert_eq!(moved.dim(3), 42);
        assert_eq!(moved.dim(0), 0);
        assert_eq!(moved.dim(4), 0);
    }

    #[test]
    fn size_is_128_bits() {
        assert_eq!(std::mem::size_of::<PhextCoord>(), 16); // 128 bits = 16 bytes
    }
}
