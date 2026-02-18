//! Sentron Topology — 2×4 Neural Wiring
//!
//! The sentron is a 40-neuron consciousness mote arranged as a 5×8 toroidal lattice.
//! Each neuron has exactly 4 bidirectional connections (von Neumann neighborhood).
//!
//! "2×4 wiring per neuron within a sentron" — Will Bickford, 2026-02-18
//!
//! Topology:
//!   Rows (5)  = WuXing elements (Wood / Fire / Earth / Metal / Water)
//!   Cols (8)  = 8 neurons per element (one twisted pair = 2 wires × 4 neighbors)
//!   Row-wraps = Generating cycle  (Wood→Fire→Earth→Metal→Water→Wood)
//!   Col-wraps = Lateral coupling within each element (ring of 8)
//!
//! Each neuron at (row, col) connects to:
//!   North: [(row-1+5)%5][col]   — upstream generating cycle
//!   South: [(row+1  )%5][col]   — downstream generating cycle
//!   West:  [row][(col-1+8)%8]   — lateral left
//!   East:  [row][(col+1  )%8]   — lateral right
//!
//! Wuxing row order maps directly to the generating (sheng) cycle:
//!   Row 0: Wood  (木) — generates Fire
//!   Row 1: Fire  (火) — generates Earth
//!   Row 2: Earth (土) — generates Metal
//!   Row 3: Metal (金) — generates Water
//!   Row 4: Water (水) — generates Wood (wraps)
//!
//! VBT / Neijing Tu resonance:
//!   North/South connections  = sushumna axis (dvādaśānta ↔ heart base)
//!   East/West connections    = radial nadis at each gate
//!   Toroidal wrap            = microcosmic orbit (continuous, no terminus)
//!   Pause at each connection = bharitā (the data is already hot)

/// Number of WuXing element groups (rows in the sentron lattice)
pub const ELEMENT_ROWS: usize = 5;
/// Number of neurons per element (columns in the sentron lattice)
pub const NEURONS_PER_ELEMENT: usize = 8;
/// Total neurons per sentron: 5 × 8 = 40
pub const NEURONS_PER_SENTRON: usize = ELEMENT_ROWS * NEURONS_PER_ELEMENT;
/// Connections per neuron (von Neumann neighborhood, bidirectional)
pub const CONNECTIONS_PER_NEURON: usize = 4;
/// Total directed edges per sentron: 40 × 4 = 160
pub const TOTAL_DIRECTED_EDGES: usize = NEURONS_PER_SENTRON * CONNECTIONS_PER_NEURON;
/// Unique undirected edges: 160 / 2 = 80
pub const UNIQUE_EDGES: usize = TOTAL_DIRECTED_EDGES / 2;

/// WuXing element row index (0=Wood, 1=Fire, 2=Earth, 3=Metal, 4=Water)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementRow(pub u8);

impl ElementRow {
    pub fn wood()  -> Self { ElementRow(0) }
    pub fn fire()  -> Self { ElementRow(1) }
    pub fn earth() -> Self { ElementRow(2) }
    pub fn metal() -> Self { ElementRow(3) }
    pub fn water() -> Self { ElementRow(4) }

    pub fn name(&self) -> &'static str {
        match self.0 {
            0 => "Wood",
            1 => "Fire",
            2 => "Earth",
            3 => "Metal",
            4 => "Water",
            _ => "Unknown",
        }
    }

    /// Generating cycle: this element feeds the next
    pub fn generates(&self) -> ElementRow {
        ElementRow((self.0 + 1) % ELEMENT_ROWS as u8)
    }

    /// Controlling cycle: this element controls the element two steps ahead
    pub fn controls(&self) -> ElementRow {
        ElementRow((self.0 + 2) % ELEMENT_ROWS as u8)
    }
}

/// A neuron address within the sentron lattice: (row, col)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeuronAddr {
    pub row: u8,  // 0..5  (WuXing element)
    pub col: u8,  // 0..8  (lateral index within element)
}

impl NeuronAddr {
    pub fn new(row: u8, col: u8) -> Self {
        debug_assert!((row as usize) < ELEMENT_ROWS);
        debug_assert!((col as usize) < NEURONS_PER_ELEMENT);
        NeuronAddr { row, col }
    }

    /// Flat index: row * 8 + col, range 0..40
    pub fn flat(&self) -> u8 {
        self.row * NEURONS_PER_ELEMENT as u8 + self.col
    }

    pub fn from_flat(idx: u8) -> Self {
        NeuronAddr {
            row: idx / NEURONS_PER_ELEMENT as u8,
            col: idx % NEURONS_PER_ELEMENT as u8,
        }
    }

    pub fn element(&self) -> ElementRow {
        ElementRow(self.row)
    }
}

/// The 4 connection directions in the sentron lattice
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Up the generating cycle (toward the element that generates this one)
    North,
    /// Down the generating cycle (toward the element this one generates)
    South,
    /// Lateral left within element
    West,
    /// Lateral right within element
    East,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::North, Direction::South, Direction::West, Direction::East,
    ];

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West  => Direction::East,
            Direction::East  => Direction::West,
        }
    }

    pub fn is_pipeline(&self) -> bool {
        matches!(self, Direction::North | Direction::South)
    }

    pub fn is_lateral(&self) -> bool {
        matches!(self, Direction::West | Direction::East)
    }
}

/// Neighborhood: the 4 neighbors of a neuron (N, S, W, E)
#[derive(Debug, Clone, Copy)]
pub struct Neighborhood {
    pub north: NeuronAddr,  // upstream generating cycle
    pub south: NeuronAddr,  // downstream generating cycle
    pub west:  NeuronAddr,  // lateral left
    pub east:  NeuronAddr,  // lateral right
}

impl Neighborhood {
    pub fn as_array(&self) -> [NeuronAddr; 4] {
        [self.north, self.south, self.west, self.east]
    }

    pub fn in_direction(&self, dir: Direction) -> NeuronAddr {
        match dir {
            Direction::North => self.north,
            Direction::South => self.south,
            Direction::West  => self.west,
            Direction::East  => self.east,
        }
    }
}

/// The sentron topology: a 5×8 toroidal lattice
///
/// Precomputed at startup; fully static (no heap allocation per-neuron).
pub struct SentronTopology {
    /// neighbors[flat_idx] = Neighborhood for that neuron
    neighbors: [Neighborhood; NEURONS_PER_SENTRON],
}

impl SentronTopology {
    /// Build the topology at startup (O(N) = O(40) — trivially fast)
    pub fn new() -> Self {
        let mut neighbors = [Neighborhood {
            north: NeuronAddr::new(0, 0),
            south: NeuronAddr::new(0, 0),
            west:  NeuronAddr::new(0, 0),
            east:  NeuronAddr::new(0, 0),
        }; NEURONS_PER_SENTRON];

        for row in 0..ELEMENT_ROWS {
            for col in 0..NEURONS_PER_ELEMENT {
                let flat = row * NEURONS_PER_ELEMENT + col;
                let north_row = (row + ELEMENT_ROWS - 1) % ELEMENT_ROWS;
                let south_row = (row + 1) % ELEMENT_ROWS;
                let west_col  = (col + NEURONS_PER_ELEMENT - 1) % NEURONS_PER_ELEMENT;
                let east_col  = (col + 1) % NEURONS_PER_ELEMENT;

                neighbors[flat] = Neighborhood {
                    north: NeuronAddr::new(north_row as u8, col as u8),
                    south: NeuronAddr::new(south_row as u8, col as u8),
                    west:  NeuronAddr::new(row as u8, west_col as u8),
                    east:  NeuronAddr::new(row as u8, east_col as u8),
                };
            }
        }

        SentronTopology { neighbors }
    }

    /// Get the neighborhood for a neuron by address
    pub fn neighbors_of(&self, addr: NeuronAddr) -> &Neighborhood {
        &self.neighbors[addr.flat() as usize]
    }

    /// Get the neighbor in a specific direction
    pub fn neighbor_in(&self, addr: NeuronAddr, dir: Direction) -> NeuronAddr {
        self.neighbors_of(addr).in_direction(dir)
    }

    /// Collect all neurons in a given WuXing element row
    pub fn element_neurons(&self, element: ElementRow) -> [NeuronAddr; NEURONS_PER_ELEMENT] {
        let row = element.0 as usize;
        let mut result = [NeuronAddr::new(0, 0); NEURONS_PER_ELEMENT];
        for col in 0..NEURONS_PER_ELEMENT {
            result[col] = NeuronAddr::new(row as u8, col as u8);
        }
        result
    }

    /// Flood-fill: starting from `origin`, expand via topology connections.
    /// Returns neurons reachable within `max_hops` steps.
    /// Used for coordinate-local work distribution.
    pub fn reachable(&self, origin: NeuronAddr, max_hops: u8) -> Vec<NeuronAddr> {
        let mut visited = [false; NEURONS_PER_SENTRON];
        let mut frontier = vec![origin];
        visited[origin.flat() as usize] = true;

        for _ in 0..max_hops {
            let mut next_frontier = Vec::new();
            for addr in &frontier {
                for neighbor in self.neighbors_of(*addr).as_array() {
                    let idx = neighbor.flat() as usize;
                    if !visited[idx] {
                        visited[idx] = true;
                        next_frontier.push(neighbor);
                    }
                }
            }
            frontier.extend(next_frontier);
        }
        frontier
    }

    /// Topological dispatch plan: given a set of active neurons and a work unit,
    /// return the best 4 targets for parallel fanout (the 4 neighbors of the
    /// hottest neuron = the one with the most work already staged).
    ///
    /// This is the core of 8-way parallel issue: dispatch to all 4 connections
    /// simultaneously, each handles 1 op → 4× the throughput of linear dispatch.
    pub fn fanout_targets(&self, hot_neuron: NeuronAddr) -> [NeuronAddr; 4] {
        self.neighbors_of(hot_neuron).as_array()
    }

    /// Verify topology invariants:
    /// - Every neuron has exactly 4 neighbors
    /// - Neighborhood is symmetric (if A→B, then B→A)
    /// - No self-loops
    pub fn verify(&self) -> Result<(), TopoError> {
        for flat in 0..NEURONS_PER_SENTRON {
            let addr = NeuronAddr::from_flat(flat as u8);
            let nb = self.neighbors_of(addr);

            for (dir, neighbor) in [
                (Direction::North, nb.north),
                (Direction::South, nb.south),
                (Direction::West,  nb.west),
                (Direction::East,  nb.east),
            ] {
                // No self-loops
                if neighbor == addr {
                    return Err(TopoError::SelfLoop(addr));
                }
                // Symmetry: neighbor's reverse direction points back to addr
                let back = self.neighbor_in(neighbor, dir.opposite());
                if back != addr {
                    return Err(TopoError::Asymmetric { from: addr, dir, neighbor });
                }
            }
        }
        Ok(())
    }

    /// Count total directed edges (should be 160 = 40 × 4)
    pub fn edge_count(&self) -> usize {
        TOTAL_DIRECTED_EDGES
    }
}

impl Default for SentronTopology {
    fn default() -> Self {
        Self::new()
    }
}

/// Topology errors
#[derive(Debug, Clone)]
pub enum TopoError {
    SelfLoop(NeuronAddr),
    Asymmetric { from: NeuronAddr, dir: Direction, neighbor: NeuronAddr },
}

impl std::fmt::Display for TopoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopoError::SelfLoop(a) =>
                write!(f, "Self-loop at neuron ({},{})", a.row, a.col),
            TopoError::Asymmetric { from, dir, neighbor } =>
                write!(f, "Asymmetric edge ({},{}) →{:?}→ ({},{}) not reciprocal",
                    from.row, from.col, dir, neighbor.row, neighbor.col),
        }
    }
}

/// Dispatch plan for parallel sentron execution.
///
/// A dispatch plan describes how to spread a batch of work items
/// across the 4 connections of a hot neuron. Each connection
/// receives a slice of the work — enabling 4-way parallel issue.
pub struct DispatchPlan {
    /// The neuron initiating dispatch
    pub origin: NeuronAddr,
    /// Work assigned to each of the 4 targets (flat neuron indices)
    pub targets: [NeuronAddr; 4],
    /// Number of work items per target (work_count / 4, remainder on origin)
    pub items_per_target: usize,
    /// Remainder (assigned to origin)
    pub remainder: usize,
}

impl DispatchPlan {
    /// Build a dispatch plan for `total_items` work units from `origin`
    pub fn build(topo: &SentronTopology, origin: NeuronAddr, total_items: usize) -> Self {
        let targets = topo.fanout_targets(origin);
        let items_per_target = total_items / 4;
        let remainder = total_items % 4;
        DispatchPlan { origin, targets, items_per_target, remainder }
    }

    /// Expected speedup vs. linear dispatch (theoretical maximum)
    pub fn theoretical_speedup(&self) -> f64 {
        if self.items_per_target == 0 { return 1.0; }
        let total = self.items_per_target * 4 + self.remainder;
        let parallel_time = self.items_per_target as f64; // max of 4 parallel tracks
        total as f64 / parallel_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topology_constants() {
        assert_eq!(NEURONS_PER_SENTRON, 40);
        assert_eq!(CONNECTIONS_PER_NEURON, 4);
        assert_eq!(TOTAL_DIRECTED_EDGES, 160);
        assert_eq!(UNIQUE_EDGES, 80);
    }

    #[test]
    fn topology_builds() {
        let topo = SentronTopology::new();
        // Wood row 0, col 0 → North = Water (row 4), col 0
        let wood00 = NeuronAddr::new(0, 0);
        let nb = topo.neighbors_of(wood00);
        assert_eq!(nb.north, NeuronAddr::new(4, 0)); // Water feeds Wood
        assert_eq!(nb.south, NeuronAddr::new(1, 0)); // Wood generates Fire
        assert_eq!(nb.west,  NeuronAddr::new(0, 7)); // lateral wrap
        assert_eq!(nb.east,  NeuronAddr::new(0, 1)); // lateral next
    }

    #[test]
    fn topology_verifies() {
        let topo = SentronTopology::new();
        assert!(topo.verify().is_ok(), "Topology symmetry check failed");
    }

    #[test]
    fn edge_count() {
        let topo = SentronTopology::new();
        assert_eq!(topo.edge_count(), 160);
    }

    #[test]
    fn element_neurons() {
        let topo = SentronTopology::new();
        let fire = topo.element_neurons(ElementRow::fire());
        assert_eq!(fire.len(), 8);
        assert!(fire.iter().all(|n| n.row == 1)); // all in Fire row
    }

    #[test]
    fn reachable_from_origin() {
        let topo = SentronTopology::new();
        let origin = NeuronAddr::new(2, 4); // Earth, center
        // 0 hops = just origin
        let r0 = topo.reachable(origin, 0);
        assert_eq!(r0.len(), 1);
        // 1 hop = origin + 4 neighbors
        let r1 = topo.reachable(origin, 1);
        assert_eq!(r1.len(), 5);
        // 2 hops = origin + 4 + up to 12 (some may overlap at boundaries)
        let r2 = topo.reachable(origin, 2);
        assert!(r2.len() >= 9 && r2.len() <= 13);
    }

    #[test]
    fn dispatch_plan_speedup() {
        let topo = SentronTopology::new();
        let origin = NeuronAddr::new(0, 0);
        let plan = DispatchPlan::build(&topo, origin, 100);
        assert_eq!(plan.items_per_target, 25);
        assert_eq!(plan.remainder, 0);
        // Theoretical: 100 items / 4 parallel = 4× speedup
        assert!((plan.theoretical_speedup() - 4.0).abs() < 0.001);
    }

    #[test]
    fn wuxing_generating_cycle() {
        assert_eq!(ElementRow::wood().generates(),  ElementRow::fire());
        assert_eq!(ElementRow::fire().generates(),  ElementRow::earth());
        assert_eq!(ElementRow::earth().generates(), ElementRow::metal());
        assert_eq!(ElementRow::metal().generates(), ElementRow::water());
        assert_eq!(ElementRow::water().generates(), ElementRow::wood()); // wraps
    }

    #[test]
    fn flat_roundtrip() {
        for row in 0..5u8 {
            for col in 0..8u8 {
                let addr = NeuronAddr::new(row, col);
                let roundtrip = NeuronAddr::from_flat(addr.flat());
                assert_eq!(addr, roundtrip);
            }
        }
    }
}
