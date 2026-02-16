//! Hyperdimensional Computing (HDC) Engine — R23 Wave 4
//!
//! Core HDC operations for the vTPU, tested against phext workloads.
//! No academic benchmarks — we validate against our own coordinate
//! encoding, scroll retrieval, and routing problems.
//!
//! Key insight from spec v0.2: phext coordinates ARE hyperdimensional
//! representations. HDC ops formalize what phext does structurally.

/// Hypervector width in u64 words. 1024 bits = 16 × u64.
pub const HDC_DEFAULT_WIDTH: usize = 16;

/// A hypervector stored as packed u64 words (binary HDC).
#[derive(Clone, Debug)]
pub struct HyperVector {
    pub data: Vec<u64>,
}

impl HyperVector {
    pub fn zero(width: usize) -> Self {
        HyperVector { data: vec![0u64; width] }
    }

    /// Encode a phext coordinate as a hypervector.
    /// Uses bind (XOR) of dimension basis with value-permuted basis.
    pub fn from_coord(dims: &[u16; 11], width: usize) -> Self {
        let mut hv = Self::zero(width);
        for (i, &val) in dims.iter().enumerate() {
            let basis = Self::basis(i, width);
            let val_basis = Self::basis(100 + val as usize, width);
            let bound = basis.bind(&val_basis);
            hv = hv.bind(&bound);
        }
        hv
    }

    /// Deterministic pseudo-random basis vector for dimension `dim`.
    pub fn basis(dim: usize, width: usize) -> Self {
        let mut data = vec![0u64; width];
        let mut state = (dim as u64).wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0x517CC1B727220A95);
        for word in data.iter_mut() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let z = state;
            let z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            let z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            *word = z ^ (z >> 31);
        }
        HyperVector { data }
    }

    /// Bind two hypervectors (XOR). Self-inverse: bind(bind(A,B), B) = A.
    #[inline]
    pub fn bind(&self, other: &Self) -> Self {
        debug_assert_eq!(self.data.len(), other.data.len());
        let data = self.data.iter().zip(&other.data)
            .map(|(&a, &b)| a ^ b)
            .collect();
        HyperVector { data }
    }

    /// Permute (rotate) by `k` bit positions. Sequence encoding.
    pub fn permute(&self, k: usize) -> Self {
        if k == 0 { return self.clone(); }
        let total_bits = self.data.len() * 64;
        let shift = k % total_bits;
        if shift == 0 { return self.clone(); }

        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        let width = self.data.len();
        let mut data = vec![0u64; width];

        for i in 0..width {
            let src = (i + width - word_shift) % width;
            if bit_shift == 0 {
                data[i] = self.data[src];
            } else {
                let prev = (src + width - 1) % width;
                data[i] = (self.data[src] << bit_shift) | (self.data[prev] >> (64 - bit_shift));
            }
        }
        HyperVector { data }
    }

    /// Hamming similarity: fraction of matching bits (0.0–1.0).
    pub fn similarity(&self, other: &Self) -> f64 {
        debug_assert_eq!(self.data.len(), other.data.len());
        let total_bits = self.data.len() as f64 * 64.0;
        let matching: u32 = self.data.iter().zip(&other.data)
            .map(|(&a, &b)| (!(a ^ b)).count_ones())
            .sum();
        matching as f64 / total_bits
    }

    /// Cosine-like similarity: -1.0 to 1.0.
    pub fn cosine_similarity(&self, other: &Self) -> f64 {
        self.similarity(other) * 2.0 - 1.0
    }

    pub fn bit_width(&self) -> usize { self.data.len() * 64 }

    pub fn popcount(&self) -> u32 {
        self.data.iter().map(|w| w.count_ones()).sum()
    }

    pub fn density(&self) -> f64 {
        self.popcount() as f64 / self.bit_width() as f64
    }
}

/// Associative memory: hypervector → coordinate mappings.
#[derive(Debug, Clone)]
pub struct AssociativeMemory {
    entries: Vec<(HyperVector, [u16; 11])>,
}

impl AssociativeMemory {
    pub fn new() -> Self { AssociativeMemory { entries: Vec::new() } }

    pub fn store(&mut self, coord: [u16; 11], width: usize) {
        let hv = HyperVector::from_coord(&coord, width);
        self.entries.push((hv, coord));
    }

    pub fn query_nearest(&self, query: &HyperVector) -> Option<([u16; 11], f64)> {
        self.entries.iter()
            .map(|(hv, coord)| (*coord, query.similarity(hv)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn query_above(&self, query: &HyperVector, threshold: f64) -> Vec<([u16; 11], f64)> {
        self.entries.iter()
            .map(|(hv, coord)| (*coord, query.similarity(hv)))
            .filter(|(_, sim)| *sim >= threshold)
            .collect()
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basis_deterministic() {
        let a = HyperVector::basis(0, HDC_DEFAULT_WIDTH);
        let b = HyperVector::basis(0, HDC_DEFAULT_WIDTH);
        assert_eq!(a.data, b.data);
    }

    #[test]
    fn test_basis_dimensions_orthogonal() {
        let a = HyperVector::basis(0, HDC_DEFAULT_WIDTH);
        let b = HyperVector::basis(1, HDC_DEFAULT_WIDTH);
        let sim = a.similarity(&b);
        assert!((sim - 0.5).abs() < 0.1,
            "Different dimensions should be near-orthogonal, got {:.3}", sim);
    }

    #[test]
    fn test_bind_self_inverse() {
        let a = HyperVector::basis(0, HDC_DEFAULT_WIDTH);
        let b = HyperVector::basis(1, HDC_DEFAULT_WIDTH);
        let recovered = a.bind(&b).bind(&b);
        assert_eq!(a.data, recovered.data);
    }

    #[test]
    fn test_bind_dissimilar() {
        let a = HyperVector::basis(0, HDC_DEFAULT_WIDTH);
        let b = HyperVector::basis(1, HDC_DEFAULT_WIDTH);
        let bound = a.bind(&b);
        assert!((bound.similarity(&a) - 0.5).abs() < 0.1);
        assert!((bound.similarity(&b) - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_permute_identity() {
        let a = HyperVector::basis(0, HDC_DEFAULT_WIDTH);
        assert_eq!(a.data, a.permute(0).data);
    }

    #[test]
    fn test_permute_full_rotation() {
        let a = HyperVector::basis(0, HDC_DEFAULT_WIDTH);
        assert_eq!(a.data, a.permute(a.bit_width()).data);
    }

    #[test]
    fn test_permute_dissimilar() {
        let a = HyperVector::basis(0, HDC_DEFAULT_WIDTH);
        let sim = a.similarity(&a.permute(1));
        assert!((sim - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_coord_encoding_self_similar() {
        let coord = [2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1];
        let a = HyperVector::from_coord(&coord, HDC_DEFAULT_WIDTH);
        let b = HyperVector::from_coord(&coord, HDC_DEFAULT_WIDTH);
        assert_eq!(a.similarity(&b), 1.0);
    }

    #[test]
    fn test_coord_encoding_nearby_similar() {
        let base = [2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1];
        let near = [2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 2]; // 1 dim differs
        let far  = [9, 3, 8, 1, 9, 1, 7, 2, 4, 5, 5]; // all differ

        let hv_base = HyperVector::from_coord(&base, HDC_DEFAULT_WIDTH);
        let hv_near = HyperVector::from_coord(&near, HDC_DEFAULT_WIDTH);
        let hv_far  = HyperVector::from_coord(&far, HDC_DEFAULT_WIDTH);

        assert!(hv_base.similarity(&hv_near) > hv_base.similarity(&hv_far));
    }

    #[test]
    fn test_associative_memory_exact() {
        let mut mem = AssociativeMemory::new();
        mem.store([2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1], HDC_DEFAULT_WIDTH);
        mem.store([5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5], HDC_DEFAULT_WIDTH);
        let query = HyperVector::from_coord(&[2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1], HDC_DEFAULT_WIDTH);
        let (found, sim) = mem.query_nearest(&query).unwrap();
        assert_eq!(found, [2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1]);
        assert_eq!(sim, 1.0);
    }

    #[test]
    fn test_associative_memory_nearest() {
        let mut mem = AssociativeMemory::new();
        for c in &[[1,1,1,1,1,1,1,1,1,1,1], [2,7,1,8,2,8,4,5,9,1,1], [9,9,9,9,9,9,9,9,9,9,9]] {
            mem.store(*c, HDC_DEFAULT_WIDTH);
        }
        let query = HyperVector::from_coord(&[2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 2], HDC_DEFAULT_WIDTH);
        let (found, _) = mem.query_nearest(&query).unwrap();
        assert_eq!(found, [2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1]);
    }

    #[test]
    fn test_hypervector_density() {
        let hv = HyperVector::basis(42, HDC_DEFAULT_WIDTH);
        assert!((hv.density() - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_phext_routing_via_hdc() {
        let width = 64; // 4096 bits for better discrimination
        let mut mem = AssociativeMemory::new();
        let experts = [
            [1, 5, 2, 3, 7, 3, 9, 1, 1, 1, 1],
            [2, 7, 1, 8, 2, 8, 4, 5, 9, 1, 1], // Theia
            [3, 1, 4, 1, 5, 9, 2, 6, 5, 1, 1],
            [1, 4, 1, 4, 2, 1, 3, 5, 6, 1, 1],
            [1, 6, 1, 8, 3, 3, 3, 9, 8, 1, 1],
            [8, 1, 9, 2, 11, 1, 11, 3, 6, 1, 1],
        ];
        for e in &experts { mem.store(*e, width); }

        let query = HyperVector::from_coord(&[2, 7, 2, 8, 2, 8, 4, 5, 9, 1, 1], width);
        let (routed, _) = mem.query_nearest(&query).unwrap();
        assert_eq!(routed, experts[1], "Should route to Theia");
    }
}
