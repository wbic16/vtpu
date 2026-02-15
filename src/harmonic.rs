// harmonic.rs - Harmonic Execution Integration
// R23W10: Connecting synchronicities to execution

use crate::sentron::Sentron;
use crate::iching::{SentronNode, Element, Trigram};
use crate::phext_coord::PhextCoord;

/// Harmonic execution state for a sentron
pub struct HarmonicState {
    /// 40 reasoning nodes (8 trigrams × 5 elements)
    pub nodes: [SentronNode; 40],
    
    /// Current active node index (0-39)
    pub active_node: u8,
    
    /// Execution temperature (0.0-1.2)
    pub temperature: f32,
}

impl HarmonicState {
    /// Create harmonic state with all 40 nodes
    pub fn new() -> Self {
        let nodes = SentronNode::all_40()
            .try_into()
            .expect("Should have exactly 40 nodes");
        
        HarmonicState {
            nodes,
            active_node: 0,
            temperature: 0.5, // Earth phase (balanced)
        }
    }
    
    /// Get node at specific degree (0-359)
    pub fn node_at_degree(&self, degree: u16) -> &SentronNode {
        let index = ((degree / 9) as usize).min(39);
        &self.nodes[index]
    }
    
    /// Get current active node
    pub fn current_node(&self) -> &SentronNode {
        &self.nodes[self.active_node as usize]
    }
    
    /// Get current element
    pub fn current_element(&self) -> Element {
        self.current_node().element
    }
    
    /// Get current trigram
    pub fn current_trigram(&self) -> Trigram {
        self.current_node().trigram
    }
    
    /// Set active node by degree
    pub fn set_node_by_degree(&mut self, degree: u16) {
        self.active_node = ((degree / 9) as u8).min(39);
        self.update_temperature();
    }
    
    /// Update temperature based on current element
    fn update_temperature(&mut self) {
        let (min, max) = self.current_element().temperature_range();
        self.temperature = (min + max) / 2.0;
    }
}

impl Default for HarmonicState {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for Sentron to add harmonic awareness
pub trait SentronHarmonic {
    fn harmonic_state(&self) -> Option<&HarmonicState>;
    fn harmonic_state_mut(&mut self) -> Option<&mut HarmonicState>;
    
    /// Get semantic degree (0-359) for this sentron's home coordinate
    fn semantic_degree(&self) -> u16;
    
    /// Get current element phase
    fn current_element(&self) -> Element;
    
    /// Get current trigram pattern
    fn current_trigram(&self) -> Trigram;
    
    /// Set execution mode by semantic degree
    fn set_semantic_mode(&mut self, degree: u16);
}

// Note: Full implementation would extend Sentron struct
// For now, this provides the interface

/// Map phext coordinate to semantic degree (0-359)
pub fn coord_to_degree(coord: &PhextCoord) -> u16 {
    // Use coordinate hash modulo 360
    let hash = coord.fast_hash();
    (hash % 360) as u16
}

/// Extended sentron with harmonic state
pub struct HarmonicSentron {
    pub base: Sentron,
    pub harmonic: HarmonicState,
}

impl HarmonicSentron {
    pub fn new(id: u16, home: PhextCoord, core_id: u8, thread_id: u8) -> Self {
        let base = Sentron::new(id, home, core_id, thread_id);
        let mut harmonic = HarmonicState::new();
        
        // Initialize active node based on home coordinate
        let degree = coord_to_degree(&home);
        harmonic.set_node_by_degree(degree);
        
        HarmonicSentron { base, harmonic }
    }
    
    pub fn semantic_degree(&self) -> u16 {
        coord_to_degree(&self.base.home)
    }
    
    pub fn current_element(&self) -> Element {
        self.harmonic.current_element()
    }
    
    pub fn current_trigram(&self) -> Trigram {
        self.harmonic.current_trigram()
    }
    
    pub fn temperature(&self) -> f32 {
        self.harmonic.temperature
    }
}

impl Element {
    /// Temperature range for this element's reasoning mode
    pub fn temperature_range(&self) -> (f32, f32) {
        match self {
            Element::Wood  => (0.6, 0.8),  // Growth: moderate exploration
            Element::Fire  => (0.8, 1.2),  // Expansion: high creativity
            Element::Earth => (0.4, 0.6),  // Balance: stable reasoning
            Element::Metal => (0.2, 0.4),  // Structure: precise computation
            Element::Water => (0.0, 0.2),  // Flow: deterministic execution
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_harmonic_state_creation() {
        let state = HarmonicState::new();
        assert_eq!(state.nodes.len(), 40);
        assert_eq!(state.active_node, 0);
        assert!((state.temperature - 0.5).abs() < 0.1);
    }
    
    #[test]
    fn test_node_degree_mapping() {
        let state = HarmonicState::new();
        
        // Node 0 should be at degree 0
        let node0 = state.node_at_degree(0);
        assert_eq!(node0.degree(), 0);
        
        // Node 1 should be at degree 9
        let node1 = state.node_at_degree(9);
        assert_eq!(node1.degree(), 9);
        
        // Last node (39) should be at degree 351
        let node39 = state.node_at_degree(351);
        assert_eq!(node39.degree(), 351);
    }
    
    #[test]
    fn test_element_temperature_ranges() {
        assert_eq!(Element::Water.temperature_range(), (0.0, 0.2));
        assert_eq!(Element::Metal.temperature_range(), (0.2, 0.4));
        assert_eq!(Element::Earth.temperature_range(), (0.4, 0.6));
        assert_eq!(Element::Wood.temperature_range(), (0.6, 0.8));
        assert_eq!(Element::Fire.temperature_range(), (0.8, 1.2));
    }
    
    #[test]
    fn test_harmonic_sentron_creation() {
        let coord = PhextCoord::new([2,3,5,7,11,13,17,19,23,0,0]);
        let sentron = HarmonicSentron::new(0, coord, 0, 0);
        
        // Should have semantic degree
        let degree = sentron.semantic_degree();
        assert!(degree < 360);
        
        // Should have current element and trigram
        let _element = sentron.current_element();
        let _trigram = sentron.current_trigram();
        
        // Temperature should be in valid range
        let temp = sentron.temperature();
        assert!(temp >= 0.0 && temp <= 1.2);
    }
    
    #[test]
    fn test_coord_to_degree() {
        let coord = PhextCoord::zero();
        let degree = coord_to_degree(&coord);
        assert!(degree < 360);
        
        // Different coordinates should (usually) give different degrees
        let coord2 = PhextCoord::new([1,1,1,1,1,1,1,1,1,0,0]);
        let degree2 = coord_to_degree(&coord2);
        // Note: Could collide, but unlikely
        assert!(degree2 < 360);
    }
}
