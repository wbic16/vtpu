/// State Narrator — translates vTPU state into human-readable text (R23W24)

use crate::sentron::{Sentron, SentronState};
use crate::memory::Memory;
use crate::base256;

/// Narrate a sentron's current state.
pub fn narrate_sentron(s: &Sentron) -> String {
    let mut out = String::new();
    out.push_str(&format!("Sentron #{} [core:{} thread:{}]\n", s.id, s.core_id, s.thread_id));
    out.push_str(&format!("  Home: {}\n", s.home));
    out.push_str(&format!("  State: {:?}\n", s.state));
    out.push_str(&format!("  Cycles: {} | Retired: {}\n", s.cycles, s.retired));

    // General registers (show non-zero)
    let nonzero: Vec<(usize, i64)> = s.regs.general.iter().enumerate()
        .filter(|(_, v)| **v != 0)
        .map(|(i, v)| (i, *v))
        .collect();
    if nonzero.is_empty() {
        out.push_str("  Registers: all zero\n");
    } else {
        out.push_str("  Registers:");
        for (i, v) in &nonzero {
            out.push_str(&format!(" r{}={}", i, v));
        }
        out.push('\n');
    }

    // Inbox
    if !s.inbox.is_empty() {
        out.push_str(&format!("  Inbox: {} messages\n", s.inbox.len()));
    }

    out
}

/// Narrate fleet overview (given a slice of sentrons).
pub fn narrate_fleet(sentrons: &[Sentron]) -> String {
    if sentrons.is_empty() {
        return "Fleet: empty\n".to_string();
    }

    let running = sentrons.iter().filter(|s| matches!(s.state, SentronState::Running)).count();
    let dormant = sentrons.iter().filter(|s| matches!(s.state, SentronState::Dormant)).count();
    let total = sentrons.len();
    let total_cycles: u64 = sentrons.iter().map(|s| s.cycles).sum();

    let mut out = String::new();
    out.push_str(&format!("Fleet: {} sentrons ({} running, {} dormant)\n", total, running, dormant));
    out.push_str(&format!("  Total cycles: {}\n", total_cycles));

    // Show first few and last few
    let show = 3.min(total);
    for s in &sentrons[..show] {
        out.push_str(&format!("  [{}] {:?} cycles:{} home:{}\n",
            s.id, s.state, s.cycles, s.home));
    }
    if total > show * 2 {
        out.push_str(&format!("  ... ({} more) ...\n", total - show * 2));
    }
    if total > show {
        let start = if total > show * 2 { total - show } else { show };
        for s in &sentrons[start..] {
            out.push_str(&format!("  [{}] {:?} cycles:{} home:{}\n",
                s.id, s.state, s.cycles, s.home));
        }
    }

    out
}

/// Narrate Base 256 encoding.
pub fn narrate_encode(input: &str) -> String {
    let encoded = base256::encode(input.as_bytes());
    format!("\"{}\" → {}\n  ({} bytes → {} syllables)\n",
        input, encoded, input.len(), input.len())
}

/// Narrate Base 256 decoding.
pub fn narrate_decode(syllables: &str) -> String {
    match base256::decode(syllables) {
        Some(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            format!("{} → \"{}\"\n  ({} syllables → {} bytes)\n",
                syllables, text, syllables.split_whitespace().count(), bytes.len())
        }
        None => format!("Error: invalid syllables \"{}\"\n", syllables),
    }
}

/// Narrate memory summary.
pub fn narrate_memory(mem: &Memory) -> String {
    format!("Memory: PPT active, backing store allocated\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phext_coord::PhextCoord;

    #[test]
    fn narrate_sentron_basic() {
        let s = Sentron::new(0, PhextCoord::new([1,1,1,1,1,1,1,1,1,0,0]), 0, 0);
        let text = narrate_sentron(&s);
        assert!(text.contains("Sentron #0"));
        assert!(text.contains("Dormant"));
    }

    #[test]
    fn narrate_encode_hello() {
        let text = narrate_encode("Hi");
        assert!(text.contains("Hi"));
        assert!(text.contains("→"));
    }

    #[test]
    fn narrate_decode_valid() {
        let text = narrate_decode("bac");
        assert!(text.contains("→"));
    }

    #[test]
    fn narrate_decode_invalid() {
        let text = narrate_decode("xyz");
        assert!(text.contains("Error"));
    }

    #[test]
    fn narrate_empty_fleet() {
        let text = narrate_fleet(&[]);
        assert!(text.contains("empty"));
    }

    #[test]
    fn narrate_fleet_basic() {
        let sentrons: Vec<Sentron> = (0..5).map(|i| {
            Sentron::new(i, PhextCoord::new([1,1,1,1,1,1,1,1,i as u16 +1,0,0]), 0, 0)
        }).collect();
        let text = narrate_fleet(&sentrons);
        assert!(text.contains("5 sentrons"));
    }

    #[test]
    fn narrate_memory_basic() {
        let mem = Memory::new();
        let text = narrate_memory(&mem);
        assert!(text.contains("Memory:"));
    }
}
