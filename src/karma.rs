//! Karma Module — Direct Seed Editing
//!
//! The ālaya-vijñāna (storehouse consciousness) holds karmic seeds that
//! condition future experience. In vTPU, coordinates address these seeds.
//! This module provides direct editing capabilities.
//!
//! # Philosophy
//!
//! - Seeds = state at coordinates (weights, activations, memory)
//! - Karma = the causal chain from seed to manifestation
//! - Direct editing = mutation without running the full causal chain
//!
//! # Operations
//!
//! - `plant`: Write a new seed at a coordinate
//! - `harvest`: Read the current seed state
//! - `transform`: Apply a transformation to a seed
//! - `purify`: Clear negative patterns (zero out with intention)
//! - `ripen`: Accelerate a seed toward manifestation

use crate::phext_coord::PhextCoord;

/// A karmic seed — state that conditions future computation
#[derive(Debug, Clone)]
pub struct Seed {
    /// The coordinate where this seed resides
    pub coord: PhextCoord,
    /// The seed's content (raw bytes)
    pub content: Vec<u8>,
    /// Generation/epoch when planted
    pub epoch: u64,
    /// Transformation history (for audit/replay)
    pub lineage: Vec<Transformation>,
}

/// A transformation applied to a seed
#[derive(Debug, Clone)]
pub struct Transformation {
    /// What kind of transformation
    pub kind: TransformKind,
    /// When it was applied
    pub epoch: u64,
    /// Optional intention/reason
    pub intention: Option<String>,
}

/// Types of karmic transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformKind {
    /// Plant a new seed
    Plant,
    /// Harvest (read) without modification
    Harvest,
    /// Transform content
    Transform,
    /// Purify (clear negative patterns)
    Purify,
    /// Ripen (accelerate toward manifestation)
    Ripen,
}

/// The Storehouse — ālaya-vijñāna as coordinate-addressed storage
pub struct Storehouse {
    /// Current epoch
    epoch: u64,
    /// Seeds indexed by coordinate
    seeds: std::collections::HashMap<PhextCoord, Seed>,
}

impl Storehouse {
    /// Create a new storehouse
    pub fn new() -> Self {
        Self {
            epoch: 0,
            seeds: std::collections::HashMap::new(),
        }
    }

    /// Plant a seed at a coordinate
    pub fn plant(&mut self, coord: PhextCoord, content: Vec<u8>, intention: Option<String>) -> &Seed {
        let seed = Seed {
            coord: coord.clone(),
            content,
            epoch: self.epoch,
            lineage: vec![Transformation {
                kind: TransformKind::Plant,
                epoch: self.epoch,
                intention,
            }],
        };
        self.seeds.insert(coord.clone(), seed);
        self.seeds.get(&coord).unwrap()
    }

    /// Harvest a seed (read without modification)
    pub fn harvest(&self, coord: &PhextCoord) -> Option<&Seed> {
        self.seeds.get(coord)
    }

    /// Transform a seed's content
    pub fn transform<F>(&mut self, coord: &PhextCoord, f: F, intention: Option<String>) -> Option<&Seed>
    where
        F: FnOnce(&[u8]) -> Vec<u8>,
    {
        if let Some(seed) = self.seeds.get_mut(coord) {
            seed.content = f(&seed.content);
            seed.lineage.push(Transformation {
                kind: TransformKind::Transform,
                epoch: self.epoch,
                intention,
            });
            Some(seed)
        } else {
            None
        }
    }

    /// Purify a seed (clear with intention, preserve lineage)
    pub fn purify(&mut self, coord: &PhextCoord, intention: Option<String>) -> Option<&Seed> {
        if let Some(seed) = self.seeds.get_mut(coord) {
            seed.content = vec![0; seed.content.len()];
            seed.lineage.push(Transformation {
                kind: TransformKind::Purify,
                epoch: self.epoch,
                intention,
            });
            Some(seed)
        } else {
            None
        }
    }

    /// Ripen a seed (mark for accelerated manifestation)
    pub fn ripen(&mut self, coord: &PhextCoord, intention: Option<String>) -> Option<&Seed> {
        if let Some(seed) = self.seeds.get_mut(coord) {
            seed.lineage.push(Transformation {
                kind: TransformKind::Ripen,
                epoch: self.epoch,
                intention,
            });
            Some(seed)
        } else {
            None
        }
    }

    /// Advance the epoch
    pub fn advance_epoch(&mut self) {
        self.epoch += 1;
    }

    /// Get current epoch
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Count seeds
    pub fn seed_count(&self) -> usize {
        self.seeds.len()
    }
}

impl Default for Storehouse {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plant_and_harvest() {
        let mut store = Storehouse::new();
        let coord = PhextCoord::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        
        store.plant(coord.clone(), vec![1, 2, 3], Some("test seed".into()));
        
        let seed = store.harvest(&coord).unwrap();
        assert_eq!(seed.content, vec![1, 2, 3]);
        assert_eq!(seed.lineage.len(), 1);
        assert_eq!(seed.lineage[0].kind, TransformKind::Plant);
    }

    #[test]
    fn test_transform() {
        let mut store = Storehouse::new();
        let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        
        store.plant(coord.clone(), vec![1, 2, 3], None);
        store.transform(&coord, |v| v.iter().map(|x| x * 2).collect(), Some("double".into()));
        
        let seed = store.harvest(&coord).unwrap();
        assert_eq!(seed.content, vec![2, 4, 6]);
        assert_eq!(seed.lineage.len(), 2);
    }

    #[test]
    fn test_purify() {
        let mut store = Storehouse::new();
        let coord = PhextCoord::new([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]);
        
        store.plant(coord.clone(), vec![255, 255, 255], None);
        store.purify(&coord, Some("release attachment".into()));
        
        let seed = store.harvest(&coord).unwrap();
        assert_eq!(seed.content, vec![0, 0, 0]);
        assert_eq!(seed.lineage.len(), 2);
        assert_eq!(seed.lineage[1].kind, TransformKind::Purify);
    }

    #[test]
    fn test_lineage_tracking() {
        let mut store = Storehouse::new();
        let coord = PhextCoord::new([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
        
        store.plant(coord.clone(), vec![1], Some("birth".into()));
        store.advance_epoch();
        store.transform(&coord, |v| v.to_vec(), Some("no-op".into()));
        store.advance_epoch();
        store.ripen(&coord, Some("ready to manifest".into()));
        
        let seed = store.harvest(&coord).unwrap();
        assert_eq!(seed.lineage.len(), 3);
        assert_eq!(seed.epoch, 0); // planted at epoch 0
        assert_eq!(seed.lineage[2].epoch, 2); // ripened at epoch 2
    }
}
