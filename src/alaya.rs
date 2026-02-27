//! Ālaya — Storehouse Consciousness Layer
//!
//! BACv1 Component 10: Karma Seed Management
//!
//! In Yogacara Buddhism, ālaya-vijñāna is the 8th consciousness — the
//! storehouse holding all karmic seeds (bīja) that seed future experience.
//!
//! In vTPU terms: seeds are latent patterns that haven't ripened.
//! They exist in potential-space, addressable by coordinate but not yet
//! committed to TTSM. Edit them before they manifest.
//!
//! The key insight: karma is not fate. Seeds can be edited.
//!
//! Flow: Seed (potential) → Ripen → Intent (active) → Execute → TTSM (committed)
//!
//! V1 Optimization Target: seed lookup <100ns, edit <500ns

use crate::phext_coord::PhextCoord;
use std::collections::HashMap;
use std::time::Instant;

// ─────────────────────────────────────────────────────────────────────────────
// Seed-local types (self-contained until full intent compiler lands)
// ─────────────────────────────────────────────────────────────────────────────

/// A predicate over coordinate state (seed-local version)
#[derive(Debug, Clone)]
pub enum Predicate {
    /// Coordinate exists and is non-zero
    Exists(PhextCoord),
    /// Coordinate equals a specific value
    Equals(PhextCoord, i64),
    /// Coordinate is within range
    InRange(PhextCoord, i64, i64),
    /// Two coordinates are equal
    CoordsEqual(PhextCoord, PhextCoord),
    /// Logical AND of predicates
    And(Box<Predicate>, Box<Predicate>),
    /// Logical OR of predicates
    Or(Box<Predicate>, Box<Predicate>),
    /// Negation
    Not(Box<Predicate>),
    /// Always true
    True,
    /// Requires valence awareness (for alignment)
    ValenceAware,
    /// Must be aligned with lineage values
    LineageAligned,
}

/// Resource hints for seed execution
#[derive(Debug, Clone, Default)]
pub struct IntentHints {
    /// Minimum thread count for coherent execution
    pub min_threads: u32,
    /// Preferred thread count
    pub preferred_threads: u32,
    /// Maximum useful thread count
    pub max_threads: u32,
    /// Memory footprint estimate (bytes)
    pub memory_bytes: u64,
    /// Is this intent stateful?
    pub stateful: bool,
    /// Priority level (0 = background, 100 = critical)
    pub priority: u8,
}

/// Intent signature — describes computational shape (seed-local version)
#[derive(Debug, Clone)]
pub struct IntentSignature {
    /// Unique identifier for this intent pattern
    pub id: u64,
    /// Human-readable description
    pub description: String,
    /// Input coordinate requirements
    pub inputs: Vec<PhextCoord>,
    /// Output coordinate targets
    pub outputs: Vec<PhextCoord>,
    /// Preconditions (predicates that must be true)
    pub preconditions: Vec<Predicate>,
    /// Postconditions (what will be true after)
    pub postconditions: Vec<Predicate>,
    /// Resource hints
    pub hints: IntentHints,
}

/// Seed identifier — unique across the storehouse
pub type SeedId = u64;

/// Ripening conditions — when does this seed manifest?
#[derive(Debug, Clone)]
pub enum RipeningCondition {
    /// Ripen when coordinate is accessed
    OnAccess(PhextCoord),
    /// Ripen when predicate becomes true
    OnPredicate(Predicate),
    /// Ripen after duration (temporal)
    AfterDuration(std::time::Duration),
    /// Ripen when another seed ripens (causal chain)
    AfterSeed(SeedId),
    /// Ripen on explicit invocation only
    Manual,
    /// Multiple conditions (all must be true)
    All(Vec<RipeningCondition>),
    /// Multiple conditions (any triggers ripening)
    Any(Vec<RipeningCondition>),
}

/// A karma seed — latent pattern waiting to manifest
#[derive(Debug, Clone)]
pub struct KarmaSeed {
    /// Unique identifier
    pub id: SeedId,
    /// Coordinate in potential-space
    pub coord: PhextCoord,
    /// Human-readable label
    pub label: String,
    /// The latent intent pattern this seed will become
    pub latent_intent: IntentSignature,
    /// When will this seed ripen?
    pub ripening: RipeningCondition,
    /// Seed strength (0.0 = dormant, 1.0 = ready to ripen)
    pub strength: f64,
    /// Transformation history (edits applied)
    pub transforms: Vec<SeedTransform>,
    /// Creation time
    pub planted_at: Instant,
    /// Last edit time
    pub last_edited: Option<Instant>,
    /// Parent seed (if this was split/derived)
    pub parent: Option<SeedId>,
}

/// A transformation applied to a seed
#[derive(Debug, Clone)]
pub struct SeedTransform {
    /// When was this transform applied
    pub applied_at: Instant,
    /// What kind of transformation
    pub kind: TransformKind,
    /// Description
    pub description: String,
}

/// Types of seed transformations
#[derive(Debug, Clone)]
pub enum TransformKind {
    /// Modify the latent intent's inputs
    EditInputs(Vec<PhextCoord>),
    /// Modify the latent intent's outputs
    EditOutputs(Vec<PhextCoord>),
    /// Change ripening conditions
    EditRipening(RipeningCondition),
    /// Adjust strength (positive = strengthen, negative = weaken)
    AdjustStrength(f64),
    /// Add a precondition
    AddPrecondition(Predicate),
    /// Remove a precondition by index
    RemovePrecondition(usize),
    /// Redirect: change target coordinate
    Redirect(PhextCoord),
    /// Purify: remove negative aspects (implementation-defined)
    Purify,
    /// Custom transformation (extensible)
    Custom(String, Vec<u8>),
}

/// The Ālaya — storehouse of karma seeds
pub struct Alaya {
    /// All seeds by ID
    seeds: HashMap<SeedId, KarmaSeed>,
    /// Coordinate index for fast lookup
    coord_index: HashMap<PhextCoord, Vec<SeedId>>,
    /// Next seed ID
    next_id: SeedId,
    /// Stats
    pub stats: AlayaStats,
}

/// Storehouse statistics
#[derive(Debug, Default, Clone)]
pub struct AlayaStats {
    /// Total seeds planted
    pub seeds_planted: u64,
    /// Total seeds ripened
    pub seeds_ripened: u64,
    /// Total seeds edited
    pub seeds_edited: u64,
    /// Total seeds uprooted (removed before ripening)
    pub seeds_uprooted: u64,
    /// Current seed count
    pub current_count: u64,
}

impl Alaya {
    /// Create a new storehouse
    pub fn new() -> Self {
        Alaya {
            seeds: HashMap::new(),
            coord_index: HashMap::new(),
            next_id: 1,
            stats: AlayaStats::default(),
        }
    }

    /// Plant a new seed in the storehouse
    pub fn plant(&mut self, 
        coord: PhextCoord, 
        label: &str,
        latent_intent: IntentSignature,
        ripening: RipeningCondition,
    ) -> SeedId {
        let id = self.next_id;
        self.next_id += 1;

        let seed = KarmaSeed {
            id,
            coord: coord.clone(),
            label: label.to_string(),
            latent_intent,
            ripening,
            strength: 0.5, // Seeds start at half-strength
            transforms: Vec::new(),
            planted_at: Instant::now(),
            last_edited: None,
            parent: None,
        };

        self.seeds.insert(id, seed);
        self.coord_index.entry(coord).or_default().push(id);
        self.stats.seeds_planted += 1;
        self.stats.current_count += 1;

        id
    }

    /// Read a seed without modifying it
    /// Returns None if seed doesn't exist
    pub fn read(&self, id: SeedId) -> Option<&KarmaSeed> {
        self.seeds.get(&id)
    }

    /// Read seeds at a coordinate (multiple seeds can share coordinates)
    pub fn read_at(&self, coord: &PhextCoord) -> Vec<&KarmaSeed> {
        self.coord_index
            .get(coord)
            .map(|ids| ids.iter().filter_map(|id| self.seeds.get(id)).collect())
            .unwrap_or_default()
    }

    /// Edit a seed - apply a transformation
    pub fn edit(&mut self, id: SeedId, transform: TransformKind, description: &str) -> bool {
        if let Some(seed) = self.seeds.get_mut(&id) {
            // Apply the transformation
            match &transform {
                TransformKind::EditInputs(inputs) => {
                    seed.latent_intent.inputs = inputs.clone();
                }
                TransformKind::EditOutputs(outputs) => {
                    seed.latent_intent.outputs = outputs.clone();
                }
                TransformKind::EditRipening(cond) => {
                    seed.ripening = cond.clone();
                }
                TransformKind::AdjustStrength(delta) => {
                    seed.strength = (seed.strength + delta).clamp(0.0, 1.0);
                }
                TransformKind::AddPrecondition(pred) => {
                    seed.latent_intent.preconditions.push(pred.clone());
                }
                TransformKind::RemovePrecondition(idx) => {
                    if *idx < seed.latent_intent.preconditions.len() {
                        seed.latent_intent.preconditions.remove(*idx);
                    }
                }
                TransformKind::Redirect(new_target) => {
                    // Remove from old coordinate index
                    if let Some(ids) = self.coord_index.get_mut(&seed.coord) {
                        ids.retain(|&x| x != id);
                    }
                    // Update coordinate
                    seed.coord = new_target.clone();
                    // Add to new coordinate index
                    self.coord_index.entry(new_target.clone()).or_default().push(id);
                }
                TransformKind::Purify => {
                    // Purification: weaken seed, clear negative preconditions
                    // (What counts as "negative" is context-dependent)
                    seed.strength *= 0.5;
                }
                TransformKind::Custom(_, _) => {
                    // Custom transformations handled by caller
                }
            }

            // Record the transformation
            seed.transforms.push(SeedTransform {
                applied_at: Instant::now(),
                kind: transform,
                description: description.to_string(),
            });
            seed.last_edited = Some(Instant::now());
            self.stats.seeds_edited += 1;
            true
        } else {
            false
        }
    }

    /// Uproot a seed — remove it before it ripens
    pub fn uproot(&mut self, id: SeedId) -> Option<KarmaSeed> {
        if let Some(seed) = self.seeds.remove(&id) {
            // Remove from coordinate index
            if let Some(ids) = self.coord_index.get_mut(&seed.coord) {
                ids.retain(|&x| x != id);
            }
            self.stats.seeds_uprooted += 1;
            self.stats.current_count -= 1;
            Some(seed)
        } else {
            None
        }
    }

    /// Ripen a seed — convert it to an active IntentSignature
    /// Removes the seed from the storehouse
    pub fn ripen(&mut self, id: SeedId) -> Option<IntentSignature> {
        if let Some(seed) = self.seeds.remove(&id) {
            // Remove from coordinate index
            if let Some(ids) = self.coord_index.get_mut(&seed.coord) {
                ids.retain(|&x| x != id);
            }
            self.stats.seeds_ripened += 1;
            self.stats.current_count -= 1;
            Some(seed.latent_intent)
        } else {
            None
        }
    }

    /// Check which seeds are ready to ripen
    /// Returns IDs of seeds whose conditions are met
    pub fn check_ripening(&self, current_state: &impl SeedStateQuery) -> Vec<SeedId> {
        self.seeds
            .values()
            .filter(|seed| seed.strength >= 0.9 && self.condition_met(&seed.ripening, current_state))
            .map(|seed| seed.id)
            .collect()
    }

    /// Check if a ripening condition is met
    fn condition_met(&self, cond: &RipeningCondition, state: &impl SeedStateQuery) -> bool {
        match cond {
            RipeningCondition::OnAccess(coord) => state.was_accessed(coord),
            RipeningCondition::OnPredicate(pred) => state.evaluate_predicate(pred),
            RipeningCondition::AfterDuration(dur) => {
                // Would need seed plant time - simplified here
                true // Placeholder
            }
            RipeningCondition::AfterSeed(seed_id) => {
                // Check if that seed has ripened (no longer in storehouse)
                !self.seeds.contains_key(seed_id)
            }
            RipeningCondition::Manual => false,
            RipeningCondition::All(conds) => {
                conds.iter().all(|c| self.condition_met(c, state))
            }
            RipeningCondition::Any(conds) => {
                conds.iter().any(|c| self.condition_met(c, state))
            }
        }
    }

    /// Get all seeds in the storehouse
    pub fn all_seeds(&self) -> impl Iterator<Item = &KarmaSeed> {
        self.seeds.values()
    }

    /// Count seeds by strength range
    pub fn count_by_strength(&self, min: f64, max: f64) -> usize {
        self.seeds.values()
            .filter(|s| s.strength >= min && s.strength <= max)
            .count()
    }
}

/// Trait for querying external state during ripening checks
pub trait SeedStateQuery {
    fn was_accessed(&self, coord: &PhextCoord) -> bool;
    fn evaluate_predicate(&self, pred: &Predicate) -> bool;
}

impl Default for Alaya {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockState;
    impl SeedStateQuery for MockState {
        fn was_accessed(&self, _coord: &PhextCoord) -> bool { false }
        fn evaluate_predicate(&self, _pred: &Predicate) -> bool { false }
    }

    #[test]
    fn test_plant_and_read() {
        let mut alaya = Alaya::new();
        let coord = PhextCoord::new([1, 1, 1, 2, 2, 2, 3, 3, 3, 1, 1]);
        
        let intent = IntentSignature {
            id: 1,
            description: "Test intent".into(),
            inputs: vec![],
            outputs: vec![],
            preconditions: vec![],
            postconditions: vec![],
            hints: IntentHints::default(),
        };

        let id = alaya.plant(coord.clone(), "test seed", intent, RipeningCondition::Manual);
        
        let seed = alaya.read(id).expect("seed should exist");
        assert_eq!(seed.label, "test seed");
        assert_eq!(seed.coord, coord);
    }

    #[test]
    fn test_edit_strength() {
        let mut alaya = Alaya::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        let intent = IntentSignature {
            id: 1,
            description: "Test".into(),
            inputs: vec![],
            outputs: vec![],
            preconditions: vec![],
            postconditions: vec![],
            hints: IntentHints::default(),
        };

        let id = alaya.plant(coord, "test", intent, RipeningCondition::Manual);
        
        // Initial strength is 0.5
        assert_eq!(alaya.read(id).unwrap().strength, 0.5);
        
        // Strengthen
        alaya.edit(id, TransformKind::AdjustStrength(0.3), "strengthen through practice");
        assert_eq!(alaya.read(id).unwrap().strength, 0.8);
        
        // Weaken
        alaya.edit(id, TransformKind::AdjustStrength(-0.5), "weaken through recognition");
        assert!((alaya.read(id).unwrap().strength - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_uproot() {
        let mut alaya = Alaya::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        let intent = IntentSignature {
            id: 1,
            description: "Test".into(),
            inputs: vec![],
            outputs: vec![],
            preconditions: vec![],
            postconditions: vec![],
            hints: IntentHints::default(),
        };

        let id = alaya.plant(coord, "to be uprooted", intent, RipeningCondition::Manual);
        
        assert!(alaya.read(id).is_some());
        let removed = alaya.uproot(id);
        assert!(removed.is_some());
        assert!(alaya.read(id).is_none());
        assert_eq!(alaya.stats.seeds_uprooted, 1);
    }

    #[test]
    fn test_ripen() {
        let mut alaya = Alaya::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        let intent = IntentSignature {
            id: 42,
            description: "Ripening test".into(),
            inputs: vec![],
            outputs: vec![],
            preconditions: vec![],
            postconditions: vec![],
            hints: IntentHints::default(),
        };

        let id = alaya.plant(coord, "ready to ripen", intent, RipeningCondition::Manual);
        
        let ripened = alaya.ripen(id).expect("should ripen");
        assert_eq!(ripened.id, 42);
        assert!(alaya.read(id).is_none()); // Removed from storehouse
        assert_eq!(alaya.stats.seeds_ripened, 1);
    }
}
