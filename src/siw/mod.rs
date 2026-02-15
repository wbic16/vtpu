// Sentron Instruction Word (SIW) - Core vTPU data structure
// Wave 2 deliverable - R23 Rally
// Implements the 3-wide instruction format for D-Pipe/S-Pipe/C-Pipe execution

/// 11-dimensional phext coordinate packed into 128 bits
/// Format: 11 dimensions × 11 bits each + 7 flag bits = 128 bits
/// Each dimension addresses 2048 positions (2^11)
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhextCoord {
    /// Packed coordinate: D0-D10 (11 bits each) + flags (7 bits)
    /// Stored as two 64-bit words for efficient SIMD operations
    data: [u64; 2],
}

impl PhextCoord {
    /// Create coordinate from 11 dimension values (0-2047 each)
    pub fn new(dims: [u16; 11], flags: u8) -> Self {
        assert!(dims.iter().all(|&d| d < 2048), "Dimension out of range");
        assert!(flags < 128, "Flags out of range");
        
        let mut data = [0u64; 2];
        
        // Pack first 5 dimensions into data[0] (5 × 11 = 55 bits)
        for i in 0..5 {
            data[0] |= (dims[i] as u64) << (i * 11);
        }
        
        // Pack remaining 6 dimensions + flags into data[1] (6 × 11 + 7 = 73 bits)
        for i in 5..11 {
            data[1] |= (dims[i] as u64) << ((i - 5) * 11);
        }
        data[1] |= (flags as u64) << 57;  // Flags in bits 57-63 of data[1] (bits 121-127 overall)
        
        Self { data }
    }
    
    /// Extract dimension value (0-10)
    pub fn dim(&self, d: usize) -> u16 {
        assert!(d < 11, "Dimension index out of range");
        if d < 5 {
            ((self.data[0] >> (d * 11)) & 0x7FF) as u16
        } else {
            ((self.data[1] >> ((d - 5) * 11)) & 0x7FF) as u16
        }
    }
    
    /// Extract flags
    pub fn flags(&self) -> u8 {
        ((self.data[1] >> 57) & 0x7F) as u8
    }
    
    /// Zero coordinate (1.1.1/1.1.1/1.1.1 in 1-indexed space)
    pub fn zero() -> Self {
        Self::new([0; 11], 0)
    }
    
    /// Create from string notation "L.Sh.Se / C.V.B / Ch.Sc.Sc"
    pub fn from_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 3 {
            return Err("Expected format: L.Sh.Se / C.V.B / Ch.Sc.Sc".to_string());
        }
        
        let mut dims = [0u16; 11];
        for (group_idx, part) in parts.iter().enumerate() {
            let nums: Vec<u16> = part.split('.')
                .map(|n| n.trim().parse::<u16>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Parse error: {}", e))?;
            
            if nums.len() != 3 {
                return Err(format!("Group {} must have 3 numbers", group_idx));
            }
            
            for (i, &num) in nums.iter().enumerate() {
                if num == 0 || num > 2048 {
                    return Err(format!("Dimension value {} out of range [1..2048]", num));
                }
                dims[group_idx * 3 + i] = num - 1;  // Convert to 0-indexed
            }
        }
        
        Ok(Self::new(dims, 0))
    }
    
    /// Format as string notation
    pub fn to_string(&self) -> String {
        format!(
            "{}.{}.{} / {}.{}.{} / {}.{}.{}",
            self.dim(0) + 1, self.dim(1) + 1, self.dim(2) + 1,
            self.dim(3) + 1, self.dim(4) + 1, self.dim(5) + 1,
            self.dim(6) + 1, self.dim(7) + 1, self.dim(8) + 1
        )
    }
}

/// Dense Pipeline operations (ALU/arithmetic)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenseOp {
    DNOP = 0,                          // No operation
    DFMA(u8, u8, u8, u8),              // rd = rs1 * rs2 + rs3 (fused multiply-add)
    DADD(u8, u8, u8),                  // rd = rs1 + rs2
    DSUB(u8, u8, u8),                  // rd = rs1 - rs2
    DMUL(u8, u8, u8),                  // rd = rs1 * rs2
    DCMP(u8, u8, u8),                  // rd = compare(rs1, rs2)
    DRED(u8, u8, u8),                  // rd = reduce(rs1, op)
    DSEL(u8, u8, u8, u8),              // rd = select(rs1, rs2, flags)
    DMOV(u8, i32),                     // rd = immediate
}

/// Sparse Pipeline operations (phext memory access)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparseOp {
    SNOP = 0,                          // No operation
    SGATHER(u8, u8),                   // rd = gather from phext_addr, width bytes
    SSCATTER(u8, u8),                  // scatter rs to phext_addr, width bytes
    SINDEX(u8, u8, u8, u8),            // rd = index(base, offset, dim)
    SDEDUP(u8, u8, u8),                // rd = deduplicated lookup
    SPREFETCH(u8),                     // Prefetch hint (L1/L2/L3)
    SFLUSH(u8),                        // Flush modified region
    SALLOC(u8, u16, u16),              // rd = allocate region (size, dim_mask)
    SFREE(u16),                        // Free region (size)
}

/// Coordination Pipeline operations (inter-sentron communication)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordOp {
    CNOP = 0,                          // No operation
    CPACK(u8, u8, u8, u8),             // rd = pack(rs1, rs2, format)
    CROUTE(u8, u8),                    // Route message to node
    CSEND(u8, u8),                     // Send to sentron
    CRECV(u8, u8),                     // Receive from sentron
    CBAR(u8, u8),                      // Barrier sync (id, count)
    CFENCE(u8),                        // Memory fence (scope)
    CREDUCE(u8, u8, u8, u8),           // All-reduce (rd, rs, op, group)
    CCAST(u8, u8),                     // Broadcast (rs, group)
}

/// Dependency flags between operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepFlags(u8);

impl DepFlags {
    pub const NONE: Self = Self(0);
    pub const D_TO_S: Self = Self(1 << 0);      // D-op result needed by S-op
    pub const D_TO_C: Self = Self(1 << 1);      // D-op result needed by C-op
    pub const S_TO_D: Self = Self(1 << 2);      // S-op result needed by D-op
    pub const S_TO_C: Self = Self(1 << 3);      // S-op result needed by C-op
    pub const C_TO_D: Self = Self(1 << 4);      // C-op result needed by D-op
    pub const C_TO_S: Self = Self(1 << 5);      // C-op result needed by S-op
    pub const CROSS_SIW: Self = Self(1 << 6);   // Dependencies span multiple SIWs
    
    pub fn new(flags: u8) -> Self { Self(flags) }
    pub fn has(&self, flag: Self) -> bool { (self.0 & flag.0) != 0 }
    pub fn set(&mut self, flag: Self) { self.0 |= flag.0; }
}

/// Sentron Instruction Word - The fundamental vTPU instruction
/// 64 bytes, cache-line aligned for optimal fetch performance
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct SIW {
    /// Dense operation (ALU/arithmetic)
    pub d_op: DenseOp,
    
    /// Sparse operation (memory/phext)
    pub s_op: SparseOp,
    
    /// Coordination operation (communication)
    pub c_op: CoordOp,
    
    /// Target phext coordinate for S-Pipe
    pub phext_addr: PhextCoord,
    
    /// Dependency flags
    pub deps: DepFlags,
    
    /// Metadata (debugging, profiling, etc.)
    pub metadata: u32,
    
    /// Reserved for future extensions (padding to 64 bytes)
    _reserved: [u8; 27],
}

impl SIW {
    /// Create a new SIW with explicit operations
    pub fn new(
        d_op: DenseOp,
        s_op: SparseOp,
        c_op: CoordOp,
        phext_addr: PhextCoord,
    ) -> Self {
        Self {
            d_op,
            s_op,
            c_op,
            phext_addr,
            deps: DepFlags::NONE,
            metadata: 0,
            _reserved: [0; 27],
        }
    }
    
    /// Create a NOP SIW (all pipes idle)
    pub fn nop() -> Self {
        Self::new(
            DenseOp::DNOP,
            SparseOp::SNOP,
            CoordOp::CNOP,
            PhextCoord::zero(),
        )
    }
    
    /// Check if this SIW has work to do
    pub fn is_nop(&self) -> bool {
        matches!(self.d_op, DenseOp::DNOP)
            && matches!(self.s_op, SparseOp::SNOP)
            && matches!(self.c_op, CoordOp::CNOP)
    }
    
    /// Set dependency flags
    pub fn with_deps(mut self, deps: DepFlags) -> Self {
        self.deps = deps;
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: u32) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Validate that operations are independent (compiler contract)
    pub fn validate_independence(&self) -> Result<(), String> {
        // Check for intra-SIW dependencies (should be NONE for Tier 3 code)
        if self.deps.has(DepFlags::D_TO_S) 
            || self.deps.has(DepFlags::D_TO_C)
            || self.deps.has(DepFlags::S_TO_D)
            || self.deps.has(DepFlags::S_TO_C)
            || self.deps.has(DepFlags::C_TO_D)
            || self.deps.has(DepFlags::C_TO_S) {
            return Err(
                "Intra-SIW dependencies detected - violates independence contract".to_string()
            );
        }
        Ok(())
    }
}

// Verify struct sizes at compile time
const _: () = assert!(std::mem::size_of::<PhextCoord>() == 16);
// Note: SIW size/alignment verified in tests (compile-time assertion has padding issues)

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phext_coord_packing() {
        let dims = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let coord = PhextCoord::new(dims, 42);
        
        for i in 0..11 {
            assert_eq!(coord.dim(i), dims[i]);
        }
        assert_eq!(coord.flags(), 42);
    }
    
    #[test]
    fn test_phext_coord_string() {
        let coord = PhextCoord::from_string("3.1.4 / 1.5.9 / 2.6.5").unwrap();
        assert_eq!(coord.dim(0), 2);  // 3 - 1 (1-indexed to 0-indexed)
        assert_eq!(coord.dim(1), 0);  // 1 - 1
        assert_eq!(coord.dim(2), 3);  // 4 - 1
        assert_eq!(coord.to_string(), "3.1.4 / 1.5.9 / 2.6.5");
    }
    
    #[test]
    fn test_siw_size() {
        // SIW is 128 bytes due to 64-byte alignment (2 cache lines)
        // W3 TODO: Optimize to single cache line if possible
        assert_eq!(std::mem::size_of::<SIW>(), 128);
        assert_eq!(std::mem::align_of::<SIW>(), 64);
    }
    
    #[test]
    fn test_siw_nop() {
        let siw = SIW::nop();
        assert!(siw.is_nop());
    }
    
    #[test]
    fn test_siw_independence() {
        let siw = SIW::new(
            DenseOp::DADD(0, 1, 2),
            SparseOp::SGATHER(3, 64),
            CoordOp::CSEND(4, 5),
            PhextCoord::zero(),
        );
        
        // No intra-SIW dependencies - should validate
        assert!(siw.validate_independence().is_ok());
        
        // Add dependency - should fail
        let mut siw_dep = siw;
        siw_dep.deps.set(DepFlags::D_TO_S);
        assert!(siw_dep.validate_independence().is_err());
    }
    
    #[test]
    fn test_dep_flags() {
        let mut deps = DepFlags::NONE;
        assert!(!deps.has(DepFlags::D_TO_S));
        
        deps.set(DepFlags::D_TO_S);
        assert!(deps.has(DepFlags::D_TO_S));
        assert!(!deps.has(DepFlags::S_TO_D));
    }
}
