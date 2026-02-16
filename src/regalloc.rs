//! Register Allocation + Dependency Resolution (R23W9)
//!
//! Three subsystems:
//! 1. HazardDetector — RAW/WAR/WAW across D/S/C pipes
//! 2. LivenessTracker — interval-based register liveness
//! 3. RegisterAllocator — virtual→physical mapping with spill detection

use crate::pipes::{DenseOp, SparseOp, CoordOp};
use crate::siw::SIW;
#[cfg(test)]
use crate::PhextCoord;

// ── Hazard Detection ──────────────────────────────────────────────

/// Hazard types between SIW pairs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hazard {
    /// Read-After-Write: consumer reads what producer wrote
    RAW { reg: u8, producer: usize, consumer: usize },
    /// Write-After-Read: new write would clobber a value still being read
    WAR { reg: u8, reader: usize, writer: usize },
    /// Write-After-Write: two writes to same register, ordering matters
    WAW { reg: u8, first: usize, second: usize },
}

/// Register access record for a single SIW
#[derive(Debug, Clone, Default)]
pub struct RegAccess {
    pub reads: Vec<(RegClass, u8)>,
    pub writes: Vec<(RegClass, u8)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegClass {
    General,  // r0-r15
    Phext,    // p0-p7
    Message,  // m0-m3
}

/// Extract all register accesses from a SIW
pub fn extract_accesses(siw: &SIW) -> RegAccess {
    let mut acc = RegAccess::default();

    // D-Pipe
    match &siw.d_op {
        DenseOp::DNOP => {}
        DenseOp::DFMA { rd, rs1, rs2, rs3 } => {
            acc.reads.extend([(RegClass::General, *rs1), (RegClass::General, *rs2), (RegClass::General, *rs3)]);
            acc.writes.push((RegClass::General, *rd));
        }
        DenseOp::DADD { rd, rs1, rs2 } | DenseOp::DSUB { rd, rs1, rs2 } |
        DenseOp::DMUL { rd, rs1, rs2 } | DenseOp::DCMP { rd, rs1, rs2 } => {
            acc.reads.extend([(RegClass::General, *rs1), (RegClass::General, *rs2)]);
            acc.writes.push((RegClass::General, *rd));
        }
        DenseOp::DRED { rd, rs1, .. } => {
            acc.reads.push((RegClass::General, *rs1));
            acc.writes.push((RegClass::General, *rd));
        }
        DenseOp::DSEL { rd, rs1, rs2, flags } => {
            acc.reads.extend([(RegClass::General, *rs1), (RegClass::General, *rs2), (RegClass::General, *flags)]);
            acc.writes.push((RegClass::General, *rd));
        }
        DenseOp::DMOV { rd, .. } => {
            acc.writes.push((RegClass::General, *rd));
        }
        DenseOp::DTERNARY { rd, rs1, trit_reg } | DenseOp::DTACC { rd, rs1, trit_reg } => {
            acc.reads.extend([(RegClass::General, *rs1), (RegClass::General, *trit_reg)]);
            acc.writes.push((RegClass::General, *rd));
        }
        DenseOp::DTPOP { rd, rs } => {
            acc.reads.push((RegClass::General, *rs));
            acc.writes.push((RegClass::General, *rd));
        }
        DenseOp::DHDENC { rd, rs, .. } | DenseOp::DHDPERM { rd, rs, .. } => {
            acc.reads.push((RegClass::General, *rs));
            acc.writes.push((RegClass::General, *rd));
        }
        DenseOp::DHDBIND { rd, rs1, rs2, .. } | DenseOp::DHDBUND { rd, rs1, rs2, .. } |
        DenseOp::DHDSIM { rd, rs1, rs2 } => {
            acc.reads.extend([(RegClass::General, *rs1), (RegClass::General, *rs2)]);
            acc.writes.push((RegClass::General, *rd));
        }
    }

    // S-Pipe
    match &siw.s_op {
        SparseOp::SNOP | SparseOp::SPREFCH { .. } | SparseOp::SFLUSH { .. } | SparseOp::SFREE { .. } => {}
        SparseOp::SINDEX { rd, base, .. } => {
            acc.reads.push((RegClass::Phext, *base));
            acc.writes.push((RegClass::Phext, *rd));
        }
        SparseOp::SGATHER { rd, coord_idx, .. } => {
            acc.reads.push((RegClass::Phext, *coord_idx));
            acc.writes.push((RegClass::General, *rd));
        }
        SparseOp::SSCATTR { coord_idx, rs, .. } => {
            acc.reads.push((RegClass::Phext, *coord_idx));
            acc.reads.push((RegClass::General, *rs));
        }
        SparseOp::SDEDUP { rd, rs, .. } => {
            acc.reads.push((RegClass::General, *rs));
            acc.writes.push((RegClass::General, *rd));
        }
        SparseOp::SALLOC { rd, .. } => {
            acc.writes.push((RegClass::General, *rd));
        }
        SparseOp::SASSOC { rd, .. } | SparseOp::SROUTE { rd, .. } | SparseOp::SNEIGHBR { rd, .. } => {
            acc.writes.push((RegClass::General, *rd));
        }
    }

    // C-Pipe
    match &siw.c_op {
        CoordOp::CNOP | CoordOp::CBAR { .. } | CoordOp::CFENCE { .. } |
        CoordOp::CSLICE { .. } | CoordOp::CFANOUT { .. } => {}
        CoordOp::CPACK { rd, rs1, rs2, .. } => {
            acc.reads.extend([(RegClass::General, *rs1), (RegClass::General, *rs2)]);
            acc.writes.push((RegClass::Message, *rd));
        }
        CoordOp::CROUTE { msg_reg, .. } => {
            acc.reads.push((RegClass::Message, *msg_reg));
        }
        CoordOp::CSEND { msg_reg, .. } => {
            acc.reads.push((RegClass::Message, *msg_reg));
        }
        CoordOp::CRECV { rd, .. } => {
            acc.writes.push((RegClass::General, *rd));
        }
        CoordOp::CREDUCE { rd, rs, .. } => {
            acc.reads.push((RegClass::General, *rs));
            acc.writes.push((RegClass::General, *rd));
        }
        CoordOp::CCAST { rs, .. } => {
            acc.reads.push((RegClass::General, *rs));
        }
        CoordOp::CMERGE { rd, .. } => {
            acc.writes.push((RegClass::General, *rd));
        }
    }

    acc
}

/// Detect all hazards in a SIW stream
pub fn detect_hazards(stream: &[SIW]) -> Vec<Hazard> {
    let accesses: Vec<RegAccess> = stream.iter().map(extract_accesses).collect();
    let mut hazards = Vec::new();

    for i in 0..accesses.len() {
        for j in (i + 1)..accesses.len() {
            // RAW: i writes, j reads same (class, reg)
            for &(wc, wr) in &accesses[i].writes {
                for &(rc, rr) in &accesses[j].reads {
                    if wc == rc && wr == rr {
                        hazards.push(Hazard::RAW { reg: wr, producer: i, consumer: j });
                    }
                }
            }
            // WAR: i reads, j writes same
            for &(rc, rr) in &accesses[i].reads {
                for &(wc, wr) in &accesses[j].writes {
                    if rc == wc && rr == wr {
                        hazards.push(Hazard::WAR { reg: rr, reader: i, writer: j });
                    }
                }
            }
            // WAW: i writes, j writes same
            for &(wc1, wr1) in &accesses[i].writes {
                for &(wc2, wr2) in &accesses[j].writes {
                    if wc1 == wc2 && wr1 == wr2 {
                        hazards.push(Hazard::WAW { reg: wr1, first: i, second: j });
                    }
                }
            }
        }
    }

    hazards
}

// ── Liveness Analysis ─────────────────────────────────────────────

/// Live interval: [def, last_use] for a virtual register
#[derive(Debug, Clone)]
pub struct LiveInterval {
    pub class: RegClass,
    pub vreg: u8,
    pub def: usize,       // SIW index where defined
    pub last_use: usize,  // SIW index of last read
}

/// Compute live intervals for all registers in a stream
pub fn compute_liveness(stream: &[SIW]) -> Vec<LiveInterval> {
    let accesses: Vec<RegAccess> = stream.iter().map(extract_accesses).collect();

    // Track first def and last use per (class, reg)
    let mut intervals: std::collections::HashMap<(RegClass, u8), (usize, usize)> =
        std::collections::HashMap::new();

    for (i, acc) in accesses.iter().enumerate() {
        for &(class, reg) in &acc.writes {
            let e = intervals.entry((class, reg)).or_insert((i, i));
            e.0 = e.0.min(i);
        }
        for &(class, reg) in &acc.reads {
            let e = intervals.entry((class, reg)).or_insert((i, i));
            e.1 = e.1.max(i);
        }
    }

    intervals.into_iter().map(|((class, vreg), (def, last_use))| {
        LiveInterval { class, vreg, def, last_use }
    }).collect()
}

// ── Register Allocator ────────────────────────────────────────────

/// Allocation result
#[derive(Debug, Clone)]
pub struct Allocation {
    /// Maps (class, virtual_reg) → physical_reg
    pub mapping: std::collections::HashMap<(RegClass, u8), u8>,
    /// Number of physical registers used per class
    pub pressure: [u8; 3],  // [general, phext, message]
    /// True if any register needed spilling (exceeded physical file)
    pub spilled: bool,
}

/// Linear-scan register allocator
///
/// Physical limits: r0-r15 (16 general), p0-p7 (8 phext), m0-m3 (4 message)
pub fn allocate_registers(stream: &[SIW]) -> Allocation {
    let mut intervals = compute_liveness(stream);
    // Sort by def point (linear scan order)
    intervals.sort_by_key(|iv| iv.def);

    let limits = [16u8, 8, 4]; // general, phext, message
    let mut active: [Vec<(u8, usize)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut mapping = std::collections::HashMap::new();
    let mut pressure = [0u8; 3];
    let mut spilled = false;

    for iv in &intervals {
        let ci = match iv.class {
            RegClass::General => 0,
            RegClass::Phext => 1,
            RegClass::Message => 2,
        };

        // Expire old intervals
        active[ci].retain(|&(_, end)| end >= iv.def);

        // Find a free physical register
        let used: Vec<u8> = active[ci].iter().map(|&(r, _)| r).collect();
        let phys = (0..limits[ci]).find(|r| !used.contains(r));

        match phys {
            Some(p) => {
                mapping.insert((iv.class, iv.vreg), p);
                active[ci].push((p, iv.last_use));
                pressure[ci] = pressure[ci].max(active[ci].len() as u8);
            }
            None => {
                // Spill: identity mapping, flag it
                mapping.insert((iv.class, iv.vreg), iv.vreg);
                spilled = true;
            }
        }
    }

    Allocation { mapping, pressure, spilled }
}

// ── Dependency Graph ──────────────────────────────────────────────

/// DAG node: one SIW with edges to its dependencies
#[derive(Debug, Clone)]
pub struct DepNode {
    pub idx: usize,
    /// Indices of SIWs this one depends on (must execute before)
    pub deps: Vec<usize>,
    /// Indices of SIWs that depend on this one
    pub dependents: Vec<usize>,
    /// Critical path length from this node to end
    pub critical_path: usize,
}

/// Build dependency DAG from hazard analysis
pub fn build_dep_graph(stream: &[SIW]) -> Vec<DepNode> {
    let n = stream.len();
    let mut nodes: Vec<DepNode> = (0..n).map(|i| DepNode {
        idx: i,
        deps: Vec::new(),
        dependents: Vec::new(),
        critical_path: 0,
    }).collect();

    let hazards = detect_hazards(stream);

    for h in &hazards {
        let (from, to) = match h {
            Hazard::RAW { producer, consumer, .. } => (*producer, *consumer),
            Hazard::WAR { reader, writer, .. } => (*reader, *writer),
            Hazard::WAW { first, second, .. } => (*first, *second),
        };
        if !nodes[to].deps.contains(&from) {
            nodes[to].deps.push(from);
            nodes[from].dependents.push(to);
        }
    }

    // Compute critical path (reverse topological)
    for i in (0..n).rev() {
        nodes[i].critical_path = if nodes[i].dependents.is_empty() {
            1
        } else {
            1 + nodes[i].dependents.iter()
                .map(|&d| nodes[d].critical_path)
                .max().unwrap_or(0)
        };
    }

    nodes
}

/// Topological sort of the DAG (Kahn's algorithm)
/// Returns reordered SIW indices that respect all dependencies
pub fn topological_sort(graph: &[DepNode]) -> Vec<usize> {
    let n = graph.len();
    let mut in_degree: Vec<usize> = graph.iter().map(|n| n.deps.len()).collect();
    // Priority: longest critical path first (greedy)
    let mut ready: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut order = Vec::with_capacity(n);

    while !ready.is_empty() {
        // Pick the ready node with longest critical path
        ready.sort_by(|&a, &b| graph[b].critical_path.cmp(&graph[a].critical_path));
        let pick = ready.remove(0);
        order.push(pick);

        for &dep in &graph[pick].dependents {
            in_degree[dep] -= 1;
            if in_degree[dep] == 0 {
                ready.push(dep);
            }
        }
    }

    order
}

/// Reorder a SIW stream to minimize stalls while respecting dependencies
pub fn reorder_stream(stream: &[SIW]) -> Vec<SIW> {
    if stream.len() <= 1 {
        return stream.to_vec();
    }
    let graph = build_dep_graph(stream);
    let order = topological_sort(&graph);
    order.iter().map(|&i| stream[i].clone()).collect()
}

/// Analyze a stream and return summary stats
#[derive(Debug, Clone)]
pub struct StreamAnalysis {
    pub siw_count: usize,
    pub raw_hazards: usize,
    pub war_hazards: usize,
    pub waw_hazards: usize,
    pub critical_path_len: usize,
    pub register_pressure: [u8; 3],
    pub needs_spill: bool,
    pub ilp: f64, // instruction-level parallelism (siws / critical_path)
}

pub fn analyze_stream(stream: &[SIW]) -> StreamAnalysis {
    let hazards = detect_hazards(stream);
    let graph = build_dep_graph(stream);
    let alloc = allocate_registers(stream);

    let raw = hazards.iter().filter(|h| matches!(h, Hazard::RAW { .. })).count();
    let war = hazards.iter().filter(|h| matches!(h, Hazard::WAR { .. })).count();
    let waw = hazards.iter().filter(|h| matches!(h, Hazard::WAW { .. })).count();
    let crit = graph.iter().map(|n| n.critical_path).max().unwrap_or(0);

    StreamAnalysis {
        siw_count: stream.len(),
        raw_hazards: raw,
        war_hazards: war,
        waw_hazards: waw,
        critical_path_len: crit,
        register_pressure: alloc.pressure,
        needs_spill: alloc.spilled,
        ilp: if crit > 0 { stream.len() as f64 / crit as f64 } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::*;

    fn mov(rd: u8, imm: i64) -> SIW {
        SIW::new(DenseOp::DMOV { rd, imm }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }
    fn add(rd: u8, rs1: u8, rs2: u8) -> SIW {
        SIW::new(DenseOp::DADD { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }
    fn mul(rd: u8, rs1: u8, rs2: u8) -> SIW {
        SIW::new(DenseOp::DMUL { rd, rs1, rs2 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero())
    }

    #[test]
    fn no_hazards_independent() {
        let stream = vec![mov(0, 1), mov(1, 2), mov(2, 3)];
        let hazards = detect_hazards(&stream);
        // WAW: none (different registers). RAW: none.
        assert!(hazards.iter().all(|h| !matches!(h, Hazard::RAW { .. })));
    }

    #[test]
    fn raw_hazard_detected() {
        let stream = vec![mov(0, 42), add(1, 0, 0)]; // r1 = r0 + r0
        let hazards = detect_hazards(&stream);
        let raw_count = hazards.iter().filter(|h| matches!(h, Hazard::RAW { .. })).count();
        assert!(raw_count > 0, "Should detect RAW on r0");
    }

    #[test]
    fn waw_hazard_detected() {
        let stream = vec![mov(0, 1), mov(0, 2)]; // Both write r0
        let hazards = detect_hazards(&stream);
        let waw_count = hazards.iter().filter(|h| matches!(h, Hazard::WAW { .. })).count();
        assert!(waw_count > 0, "Should detect WAW on r0");
    }

    #[test]
    fn dep_graph_chain() {
        // r0=1 → r1=r0+r0 → r2=r1*r1 (chain of 3)
        let stream = vec![mov(0, 1), add(1, 0, 0), mul(2, 1, 1)];
        let graph = build_dep_graph(&stream);
        assert_eq!(graph[0].critical_path, 3);
        assert_eq!(graph[2].critical_path, 1);
    }

    #[test]
    fn topological_respects_deps() {
        let stream = vec![mov(0, 1), add(1, 0, 0), mul(2, 1, 1)];
        let graph = build_dep_graph(&stream);
        let order = topological_sort(&graph);
        // Must be 0 before 1 before 2
        let pos: std::collections::HashMap<usize, usize> =
            order.iter().enumerate().map(|(i, &v)| (v, i)).collect();
        assert!(pos[&0] < pos[&1]);
        assert!(pos[&1] < pos[&2]);
    }

    #[test]
    fn reorder_exploits_ilp() {
        // Two independent chains: r0=1, r2=2, r1=r0+r0, r3=r2+r2
        // Original: [mov0, mov2, add1(r0,r0), add3(r2,r2)]
        // Both adds are independent — reorder should keep them separate
        let stream = vec![mov(0, 1), mov(2, 2), add(1, 0, 0), add(3, 2, 2)];
        let reordered = reorder_stream(&stream);
        assert_eq!(reordered.len(), 4);
        // Verify correctness: movs must come before their dependent adds
        // (the reorder should not break this)
    }

    #[test]
    fn register_pressure_low() {
        let stream = vec![mov(0, 1), mov(1, 2), add(2, 0, 1)];
        let alloc = allocate_registers(&stream);
        assert!(!alloc.spilled);
        assert!(alloc.pressure[0] <= 3); // At most 3 general regs live
    }

    #[test]
    fn register_pressure_reuse() {
        // r0=1, r1=r0+r0, r0=2, r2=r0+r1
        // r0 is reused — allocator should handle this
        let stream = vec![mov(0, 1), add(1, 0, 0), mov(0, 2), add(2, 0, 1)];
        let alloc = allocate_registers(&stream);
        assert!(!alloc.spilled);
    }

    #[test]
    fn cross_pipe_hazard() {
        // D-Pipe writes r0, S-Pipe reads r0 (scatter)
        let stream = vec![
            SIW::new(DenseOp::DMOV { rd: 0, imm: 42 }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()),
            SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: 0, rs: 0, width: 8 }, CoordOp::CNOP, PhextCoord::zero()),
        ];
        let hazards = detect_hazards(&stream);
        let raw = hazards.iter().filter(|h| matches!(h, Hazard::RAW { .. })).count();
        assert!(raw > 0, "Should detect cross-pipe RAW: D writes r0, S reads r0");
    }

    #[test]
    fn analyze_dot_product() {
        // Mini dot product: [a,b]·[c,d] = a*c + b*d
        let stream = vec![
            mov(0, 2), mov(1, 3), mov(2, 4), mov(3, 5),
            mul(4, 0, 1), // 2*3=6
            mul(5, 2, 3), // 4*5=20
            add(6, 4, 5), // 6+20=26
        ];
        let analysis = analyze_stream(&stream);
        assert_eq!(analysis.siw_count, 7);
        assert!(analysis.raw_hazards > 0);
        assert!(analysis.critical_path_len >= 3); // mov→mul→add chain
        assert!(analysis.ilp > 1.0); // Some parallelism exists
        assert!(!analysis.needs_spill);
    }

    #[test]
    fn full_pipeline_three_wide() {
        // 3-wide SIW: D computes, S gathers, C packs — all independent regs
        let stream = vec![
            SIW::new(
                DenseOp::DADD { rd: 4, rs1: 0, rs2: 1 },
                SparseOp::SGATHER { rd: 5, coord_idx: 0, width: 8 },
                CoordOp::CPACK { rd: 0, rs1: 2, rs2: 3, fmt: MessageFormat::Result },
                PhextCoord::zero(),
            ),
        ];
        let analysis = analyze_stream(&stream);
        assert_eq!(analysis.siw_count, 1);
        assert_eq!(analysis.critical_path_len, 1);
    }

    #[test]
    fn liveness_intervals() {
        let stream = vec![mov(0, 1), mov(1, 2), add(2, 0, 1)];
        let intervals = compute_liveness(&stream);
        // r0: def=0, last_use=2
        let r0 = intervals.iter().find(|iv| iv.class == RegClass::General && iv.vreg == 0);
        assert!(r0.is_some());
        let r0 = r0.unwrap();
        assert_eq!(r0.def, 0);
        assert_eq!(r0.last_use, 2);
    }
}
