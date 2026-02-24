//! Bickford's Demon — Admission Gate
//!
//! "Nothing enters without a place. Nothing persists without structure.
//!  Nothing scales without constraint."
//!
//! The Demon enforces four admittance rules at every layer boundary:
//! - Unique Placement: every datum has exactly one coordinate
//! - Atomic Meaning: the coordinate IS the identity
//! - Context Explicitness: no implicit external references
//! - Boundary Integrity: ownership is always clear
//!
//! Rejection diagnoses:
//! - Underplaced: insufficient coordinate specificity
//! - Overloaded: coordinate already occupied
//! - Leaking: references external state without binding
//! - Unowned: no clear provenance

use crate::PhextCoord;
use std::collections::HashSet;

/// Rejection category for failed admission
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    /// Lacks sufficient coordinate specificity
    Underplaced {
        provided: PhextCoord,
        reason: String,
    },
    /// Coordinate already occupied or oversubscribed
    Overloaded {
        coord: PhextCoord,
        existing: String,
        attempted: String,
    },
    /// References external state without explicit binding
    Leaking {
        coord: PhextCoord,
        external_refs: Vec<String>,
    },
    /// No clear ownership or provenance
    Unowned {
        coord: PhextCoord,
        reason: String,
    },
}

impl std::fmt::Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rejection::Underplaced { provided, reason } => {
                write!(f, "UNDERPLACED at {}: {}", provided, reason)
            }
            Rejection::Overloaded { coord, existing, attempted } => {
                write!(f, "OVERLOADED at {}: '{}' blocks '{}'", coord, existing, attempted)
            }
            Rejection::Leaking { coord, external_refs } => {
                write!(f, "LEAKING at {}: unbound refs {:?}", coord, external_refs)
            }
            Rejection::Unowned { coord, reason } => {
                write!(f, "UNOWNED at {}: {}", coord, reason)
            }
        }
    }
}

impl std::error::Error for Rejection {}

/// Result type for admission attempts
pub type AdmitResult<T> = Result<T, Rejection>;

/// Trait for types that can be placed in coordinate space
pub trait Placeable {
    /// The coordinate where this item lives
    fn coordinate(&self) -> PhextCoord;
    
    /// Human-readable identity (for diagnostics)
    fn identity(&self) -> String;
    
    /// External references this item depends on (for leak detection)
    fn external_refs(&self) -> Vec<String> {
        vec![]
    }
    
    /// Owner/provenance of this item
    fn owner(&self) -> Option<String> {
        None
    }
    
    /// Validate coordinate specificity (default: all dimensions > 0)
    fn is_sufficiently_placed(&self) -> bool {
        let c = self.coordinate();
        // All 11 dimensions must be > 0 for full placement
        (0..11).all(|i| c.get_dim(i) > 0)
    }
}

/// The Admission Gate — enforces Bickford's Demon at runtime
pub struct Demon {
    /// Occupied coordinates (for overload detection)
    occupied: HashSet<PhextCoord>,
    /// Coordinate -> identity mapping (for diagnostics)
    registry: std::collections::HashMap<PhextCoord, String>,
    /// Known external bindings (for leak detection)
    bindings: HashSet<String>,
    /// Strict mode: reject on any warning
    strict: bool,
}

impl Default for Demon {
    fn default() -> Self {
        Self::new()
    }
}

impl Demon {
    pub fn new() -> Self {
        Self {
            occupied: HashSet::new(),
            registry: std::collections::HashMap::new(),
            bindings: HashSet::new(),
            strict: true,
        }
    }
    
    /// Create a permissive demon (warnings instead of rejections)
    pub fn permissive() -> Self {
        Self {
            strict: false,
            ..Self::new()
        }
    }
    
    /// Register a known external binding (makes it "not leaking")
    pub fn bind(&mut self, external: &str) {
        self.bindings.insert(external.to_string());
    }
    
    /// Attempt to admit an item into coordinate space
    pub fn admit<T: Placeable>(&mut self, item: &T) -> AdmitResult<PhextCoord> {
        let coord = item.coordinate();
        let identity = item.identity();
        
        // Check 1: Underplaced?
        if !item.is_sufficiently_placed() {
            return Err(Rejection::Underplaced {
                provided: coord,
                reason: "one or more dimensions at origin (0)".to_string(),
            });
        }
        
        // Check 2: Overloaded?
        if self.occupied.contains(&coord) {
            let existing = self.registry.get(&coord)
                .cloned()
                .unwrap_or_else(|| "<unknown>".to_string());
            return Err(Rejection::Overloaded {
                coord,
                existing,
                attempted: identity,
            });
        }
        
        // Check 3: Leaking?
        let external_refs = item.external_refs();
        let unbound: Vec<String> = external_refs.iter()
            .filter(|r| !self.bindings.contains(*r))
            .cloned()
            .collect();
        if !unbound.is_empty() && self.strict {
            return Err(Rejection::Leaking {
                coord,
                external_refs: unbound,
            });
        }
        
        // Check 4: Unowned?
        if item.owner().is_none() && self.strict {
            return Err(Rejection::Unowned {
                coord,
                reason: "no owner specified".to_string(),
            });
        }
        
        // Admission granted
        self.occupied.insert(coord);
        self.registry.insert(coord, identity);
        
        Ok(coord)
    }
    
    /// Release a coordinate (for GC or explicit removal)
    pub fn release(&mut self, coord: &PhextCoord) -> bool {
        self.registry.remove(coord);
        self.occupied.remove(coord)
    }
    
    /// Check if a coordinate is occupied
    pub fn is_occupied(&self, coord: &PhextCoord) -> bool {
        self.occupied.contains(coord)
    }
    
    /// Get the identity at a coordinate
    pub fn identity_at(&self, coord: &PhextCoord) -> Option<&String> {
        self.registry.get(coord)
    }
    
    /// Number of occupied coordinates
    pub fn population(&self) -> usize {
        self.occupied.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test item implementing Placeable
    struct TestDatum {
        coord: PhextCoord,
        name: String,
        refs: Vec<String>,
        owner: Option<String>,
    }

    impl TestDatum {
        fn new(coord: PhextCoord, name: &str) -> Self {
            Self {
                coord,
                name: name.to_string(),
                refs: vec![],
                owner: Some("test".to_string()),
            }
        }
        
        fn with_refs(mut self, refs: Vec<&str>) -> Self {
            self.refs = refs.iter().map(|s| s.to_string()).collect();
            self
        }
        
        fn unowned(mut self) -> Self {
            self.owner = None;
            self
        }
    }

    impl Placeable for TestDatum {
        fn coordinate(&self) -> PhextCoord {
            self.coord
        }
        
        fn identity(&self) -> String {
            self.name.clone()
        }
        
        fn external_refs(&self) -> Vec<String> {
            self.refs.clone()
        }
        
        fn owner(&self) -> Option<String> {
            self.owner.clone()
        }
    }

    fn valid_coord() -> PhextCoord {
        // All 11 dimensions at 1 (fully placed)
        PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
    }

    fn underplaced_coord() -> PhextCoord {
        // Dimension 7 at 0 (underplaced)
        PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1])
    }

    #[test]
    fn test_admit_valid() {
        let mut demon = Demon::new();
        let datum = TestDatum::new(valid_coord(), "test_item");
        
        let result = demon.admit(&datum);
        assert!(result.is_ok());
        assert_eq!(demon.population(), 1);
    }

    #[test]
    fn test_reject_underplaced() {
        let mut demon = Demon::new();
        let datum = TestDatum::new(underplaced_coord(), "underplaced_item");
        
        let result = demon.admit(&datum);
        assert!(matches!(result, Err(Rejection::Underplaced { .. })));
    }

    #[test]
    fn test_reject_overloaded() {
        let mut demon = Demon::new();
        let coord = valid_coord();
        
        let first = TestDatum::new(coord, "first");
        let second = TestDatum::new(coord, "second");
        
        assert!(demon.admit(&first).is_ok());
        
        let result = demon.admit(&second);
        assert!(matches!(result, Err(Rejection::Overloaded { .. })));
    }

    #[test]
    fn test_reject_leaking() {
        let mut demon = Demon::new();
        let datum = TestDatum::new(valid_coord(), "leaky")
            .with_refs(vec!["external_system", "another_ref"]);
        
        let result = demon.admit(&datum);
        assert!(matches!(result, Err(Rejection::Leaking { .. })));
    }

    #[test]
    fn test_bound_refs_not_leaking() {
        let mut demon = Demon::new();
        demon.bind("external_system");
        demon.bind("another_ref");
        
        let datum = TestDatum::new(valid_coord(), "bound_item")
            .with_refs(vec!["external_system", "another_ref"]);
        
        let result = demon.admit(&datum);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_unowned() {
        let mut demon = Demon::new();
        let datum = TestDatum::new(valid_coord(), "orphan").unowned();
        
        let result = demon.admit(&datum);
        assert!(matches!(result, Err(Rejection::Unowned { .. })));
    }

    #[test]
    fn test_permissive_admits_unowned() {
        let mut demon = Demon::permissive();
        let datum = TestDatum::new(valid_coord(), "orphan").unowned();
        
        let result = demon.admit(&datum);
        assert!(result.is_ok());
    }

    #[test]
    fn test_release() {
        let mut demon = Demon::new();
        let coord = valid_coord();
        let datum = TestDatum::new(coord, "releasable");
        
        assert!(demon.admit(&datum).is_ok());
        assert!(demon.is_occupied(&coord));
        
        assert!(demon.release(&coord));
        assert!(!demon.is_occupied(&coord));
        
        // Can now admit another item at same coordinate
        let second = TestDatum::new(coord, "replacement");
        assert!(demon.admit(&second).is_ok());
    }

    #[test]
    fn test_identity_at() {
        let mut demon = Demon::new();
        let coord = valid_coord();
        let datum = TestDatum::new(coord, "named_item");
        
        assert!(demon.admit(&datum).is_ok());
        assert_eq!(demon.identity_at(&coord), Some(&"named_item".to_string()));
    }
}
