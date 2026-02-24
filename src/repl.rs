/// REPL + Op Compiler — interactive vTPU interface (R23W24)

use crate::sentron::Sentron;
use crate::memory::Memory;
use crate::phext_coord::PhextCoord;
use crate::narrator;
use crate::base256;

/// Parsed REPL command.
#[derive(Debug, PartialEq)]
pub enum Command {
    Status,
    Help,
    Quit,
    Encode(String),
    Decode(String),
    Fleet,
    Read(String),
    Write(String, String),
    Run(usize),
    Sentron(usize),
    Memory,
    Unknown(String),
}

/// Parse a single input line into a Command.
pub fn parse(input: &str) -> Command {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Command::Unknown(String::new());
    }

    let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        "status" | "st" => Command::Status,
        "help" | "h" | "?" => Command::Help,
        "quit" | "exit" | "q" => Command::Quit,
        "encode" | "enc" => {
            let text = if parts.len() > 1 { parts[1..].join(" ") } else { String::new() };
            Command::Encode(text)
        }
        "decode" | "dec" => {
            let text = if parts.len() > 1 { parts[1..].join(" ") } else { String::new() };
            Command::Decode(text)
        }
        "fleet" | "fl" => Command::Fleet,
        "read" | "r" => {
            let coord = if parts.len() > 1 { parts[1].to_string() } else { String::new() };
            Command::Read(coord)
        }
        "write" | "w" => {
            let coord = if parts.len() > 1 { parts[1].to_string() } else { String::new() };
            let data = if parts.len() > 2 { parts[2].to_string() } else { String::new() };
            Command::Write(coord, data)
        }
        "run" => {
            let n = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
            Command::Run(n)
        }
        "sentron" | "s" => {
            let id = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            Command::Sentron(id)
        }
        "memory" | "mem" => Command::Memory,
        _ => Command::Unknown(trimmed.to_string()),
    }
}

/// REPL session state.
pub struct ReplSession {
    pub sentrons: Vec<Sentron>,
    pub memory: Memory,
}

impl ReplSession {
    /// Create a new session with a fleet of given size.
    pub fn new(fleet_size: usize) -> Self {
        let sentrons: Vec<Sentron> = (0..fleet_size)
            .map(|i| {
                let coord = PhextCoord::new([1, 1, 1, 1, 1, 1, 1, 1, (i + 1) as u16, 0, 0]);
                Sentron::new(i as u16, coord, (i % 8) as u8, (i % 2) as u8)
            })
            .collect();
        ReplSession {
            sentrons,
            memory: Memory::new(),
        }
    }

    /// Execute a parsed command, returning output text.
    pub fn execute(&mut self, cmd: &Command) -> String {
        match cmd {
            Command::Status => self.status(),
            Command::Help => Self::help(),
            Command::Quit => "Goodbye.\n".to_string(),
            Command::Encode(text) => narrator::narrate_encode(text),
            Command::Decode(text) => narrator::narrate_decode(text),
            Command::Fleet => narrator::narrate_fleet(&self.sentrons),
            Command::Read(coord) => self.read_coord(coord),
            Command::Write(coord, data) => self.write_coord(coord, data),
            Command::Run(n) => self.run_cycles(*n),
            Command::Sentron(id) => self.show_sentron(*id),
            Command::Memory => narrator::narrate_memory(&self.memory),
            Command::Unknown(s) => {
                if s.is_empty() {
                    String::new()
                } else {
                    format!("Unknown command: \"{}\". Type 'help' for commands.\n", s)
                }
            }
        }
    }

    fn status(&self) -> String {
        let running = self.sentrons.iter()
            .filter(|s| matches!(s.state, crate::sentron::SentronState::Running))
            .count();
        let total_cycles: u64 = self.sentrons.iter().map(|s| s.cycles).sum();

        format!(
            "vTPU Status\n  Fleet: {} sentrons ({} running)\n  Total cycles: {}\n",
            self.sentrons.len(), running, total_cycles
        )
    }

    fn help() -> String {
        [
            "vTPU REPL Commands:",
            "  status (st)          — fleet health overview",
            "  sentron <id> (s)     — inspect a sentron",
            "  fleet (fl)           — fleet overview",
            "  memory (mem)         — memory status",
            "  read <coord>         — read at phext coordinate",
            "  write <coord> <data> — write to phext coordinate",
            "  run <n>              — execute n cycles",
            "  encode <text> (enc)  — Base 256 encode",
            "  decode <syls> (dec)  — Base 256 decode",
            "  help (h, ?)          — this message",
            "  quit (q)             — exit",
            "",
        ].join("\n")
    }

    fn show_sentron(&self, id: usize) -> String {
        match self.sentrons.get(id) {
            Some(s) => narrator::narrate_sentron(s),
            None => format!("No sentron with id {}. Fleet size: {}\n", id, self.sentrons.len()),
        }
    }

    fn read_coord(&mut self, coord_str: &str) -> String {
        match parse_phext_coord(coord_str) {
            Some(coord) => {
                let data = self.memory.gather(&coord, 64);
                let slice = data.to_vec();
                let b256 = base256::encode(&slice[..slice.len().min(16)]);
                let ascii: String = slice.iter().take(64).map(|&b| {
                    if b >= 0x20 && b < 0x7f { b as char } else { '.' }
                }).collect();
                format!("Read @{}:\n  Base256: {}\n  ASCII:   {}\n",
                    coord_str, &b256[..b256.len().min(80)], &ascii[..ascii.len().min(64)])
            }
            None => format!("Invalid coordinate: \"{}\". Use: L.S.Se/C.V.B/Ch.Sc.Scr\n", coord_str),
        }
    }

    fn write_coord(&mut self, coord_str: &str, data: &str) -> String {
        match parse_phext_coord(coord_str) {
            Some(coord) => {
                let bytes = data.as_bytes();
                self.memory.scatter(&coord, bytes);
                let encoded = base256::encode(&bytes[..bytes.len().min(16)]);
                format!("Wrote {} bytes @{}\n  Base256: {}\n",
                    bytes.len(), coord_str, &encoded[..encoded.len().min(60)])
            }
            None => format!("Invalid coordinate: \"{}\"\n", coord_str),
        }
    }

    fn run_cycles(&mut self, n: usize) -> String {
        let mut total_ops = 0usize;
        for s in self.sentrons.iter_mut() {
            s.cycles += n as u64;
            total_ops += n;
        }
        format!("Ran {} cycles across {} sentrons ({} total ops)\n",
            n, self.sentrons.len(), total_ops)
    }
}

/// Parse a phext coordinate string to a PhextCoord.
/// Accepts: "L.S.Se/C.V.B/Ch.Sc.Scr" (9 dims, remaining 2 default to 0)
fn parse_phext_coord(s: &str) -> Option<PhextCoord> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return None;
    }

    let mut dims = [0u16; 11];
    let mut idx = 0;
    for part in parts {
        for num_str in part.split('.') {
            if idx >= 9 { return None; }
            dims[idx] = num_str.parse().ok()?;
            idx += 1;
        }
    }

    if idx != 9 {
        return None;
    }

    // Clamp to valid range
    for d in dims.iter_mut() {
        if *d > PhextCoord::MAX_DIM {
            *d = PhextCoord::MAX_DIM;
        }
    }

    Some(PhextCoord::new(dims))
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Parser tests ===

    #[test]
    fn parse_status() {
        assert_eq!(parse("status"), Command::Status);
        assert_eq!(parse("st"), Command::Status);
    }

    #[test]
    fn parse_help() {
        assert_eq!(parse("help"), Command::Help);
        assert_eq!(parse("h"), Command::Help);
        assert_eq!(parse("?"), Command::Help);
    }

    #[test]
    fn parse_quit() {
        assert_eq!(parse("quit"), Command::Quit);
        assert_eq!(parse("exit"), Command::Quit);
        assert_eq!(parse("q"), Command::Quit);
    }

    #[test]
    fn parse_encode() {
        assert_eq!(parse("encode hello"), Command::Encode("hello".to_string()));
        assert_eq!(parse("enc hi"), Command::Encode("hi".to_string()));
    }

    #[test]
    fn parse_decode() {
        assert_eq!(parse("decode bac wom"), Command::Decode("bac wom".to_string()));
    }

    #[test]
    fn parse_fleet() {
        assert_eq!(parse("fleet"), Command::Fleet);
        assert_eq!(parse("fl"), Command::Fleet);
    }

    #[test]
    fn parse_read() {
        assert_eq!(parse("read 1.1.1/1.1.1/1.1.1"), Command::Read("1.1.1/1.1.1/1.1.1".to_string()));
    }

    #[test]
    fn parse_write() {
        assert_eq!(parse("write 1.1.1/1.1.1/1.1.1 hello"), Command::Write("1.1.1/1.1.1/1.1.1".to_string(), "hello".to_string()));
    }

    #[test]
    fn parse_run() {
        assert_eq!(parse("run 10"), Command::Run(10));
        assert_eq!(parse("run"), Command::Run(1));
    }

    #[test]
    fn parse_sentron() {
        assert_eq!(parse("sentron 5"), Command::Sentron(5));
        assert_eq!(parse("s 3"), Command::Sentron(3));
    }

    #[test]
    fn parse_memory() {
        assert_eq!(parse("memory"), Command::Memory);
        assert_eq!(parse("mem"), Command::Memory);
    }

    #[test]
    fn parse_unknown() {
        assert_eq!(parse("foobar"), Command::Unknown("foobar".to_string()));
    }

    #[test]
    fn parse_empty() {
        assert_eq!(parse(""), Command::Unknown(String::new()));
        assert_eq!(parse("  "), Command::Unknown(String::new()));
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(parse("STATUS"), Command::Status);
        assert_eq!(parse("Help"), Command::Help);
        assert_eq!(parse("QUIT"), Command::Quit);
    }

    // === Coordinate parser tests ===

    #[test]
    fn parse_coord_phext() {
        let coord = parse_phext_coord("1.1.1/1.1.1/1.1.1");
        assert!(coord.is_some());
    }

    #[test]
    fn parse_coord_invalid() {
        assert!(parse_phext_coord("abc").is_none());
        assert!(parse_phext_coord("1.2/3.4").is_none());
    }

    #[test]
    fn parse_coord_stable() {
        let a1 = parse_phext_coord("1.1.1/1.1.1/1.1.1");
        let a2 = parse_phext_coord("1.1.1/1.1.1/1.1.1");
        assert_eq!(a1, a2);
    }

    #[test]
    fn parse_coord_different() {
        let a1 = parse_phext_coord("1.1.1/1.1.1/1.1.1");
        let a2 = parse_phext_coord("2.1.1/1.1.1/1.1.1");
        assert_ne!(a1, a2);
    }

    // === Session tests ===

    #[test]
    fn session_status() {
        let session = ReplSession::new(9);
        let output = session.status();
        assert!(output.contains("9 sentrons"));
    }

    #[test]
    fn session_help() {
        let output = ReplSession::help();
        assert!(output.contains("encode"));
        assert!(output.contains("fleet"));
    }

    #[test]
    fn session_encode() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Encode("Hi".to_string()));
        assert!(output.contains("Hi"));
    }

    #[test]
    fn session_decode() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Decode("bac".to_string()));
        assert!(output.contains("→"));
    }

    #[test]
    fn session_sentron_valid() {
        let mut session = ReplSession::new(5);
        let output = session.execute(&Command::Sentron(0));
        assert!(output.contains("Sentron #0"));
    }

    #[test]
    fn session_sentron_invalid() {
        let mut session = ReplSession::new(5);
        let output = session.execute(&Command::Sentron(99));
        assert!(output.contains("No sentron"));
    }

    #[test]
    fn session_run() {
        let mut session = ReplSession::new(9);
        let output = session.execute(&Command::Run(10));
        assert!(output.contains("10 cycles"));
        assert!(output.contains("9 sentrons"));
        assert_eq!(session.sentrons[0].cycles, 10);
    }

    #[test]
    fn session_write_read() {
        let mut session = ReplSession::new(1);
        let write_out = session.execute(&Command::Write("1.1.1/1.1.1/1.1.1".to_string(), "hello".to_string()));
        assert!(write_out.contains("5 bytes"));
        let read_out = session.execute(&Command::Read("1.1.1/1.1.1/1.1.1".to_string()));
        assert!(read_out.contains("hello"));
    }

    #[test]
    fn session_fleet() {
        let mut session = ReplSession::new(9);
        let output = session.execute(&Command::Fleet);
        assert!(output.contains("9 sentrons"));
    }

    #[test]
    fn session_unknown() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Unknown("blorp".to_string()));
        assert!(output.contains("Unknown command"));
    }

    // === Hardening: unexpected inputs ===

    #[test]
    fn session_empty_unknown() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Unknown(String::new()));
        assert!(output.is_empty()); // Empty unknown = no output
    }

    #[test]
    fn session_run_zero() {
        let mut session = ReplSession::new(3);
        let output = session.execute(&Command::Run(0));
        assert!(output.contains("0 cycles"));
        assert_eq!(session.sentrons[0].cycles, 0);
    }

    #[test]
    fn session_sentron_boundary() {
        let mut session = ReplSession::new(9);
        // Last valid
        let output = session.execute(&Command::Sentron(8));
        assert!(output.contains("Sentron #8"));
        // First invalid
        let output = session.execute(&Command::Sentron(9));
        assert!(output.contains("No sentron"));
    }

    #[test]
    fn session_read_invalid_coord() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Read("garbage".to_string()));
        assert!(output.contains("Invalid coordinate"));
    }

    #[test]
    fn session_write_invalid_coord() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Write("garbage".to_string(), "data".to_string()));
        assert!(output.contains("Invalid coordinate"));
    }

    #[test]
    fn session_write_empty_data() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Write("1.1.1/1.1.1/1.1.1".to_string(), String::new()));
        assert!(output.contains("0 bytes"));
    }

    #[test]
    fn session_encode_empty() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Encode(String::new()));
        assert!(output.contains("0 bytes"));
    }

    #[test]
    fn session_decode_invalid() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Decode("xyz".to_string()));
        assert!(output.contains("Error"));
    }

    #[test]
    fn session_multiple_writes_same_coord() {
        let mut session = ReplSession::new(1);
        session.execute(&Command::Write("1.1.1/1.1.1/1.1.1".to_string(), "first".to_string()));
        session.execute(&Command::Write("1.1.1/1.1.1/1.1.1".to_string(), "second".to_string()));
        let output = session.execute(&Command::Read("1.1.1/1.1.1/1.1.1".to_string()));
        assert!(output.contains("second")); // Overwrites
    }

    #[test]
    fn session_encode_unicode() {
        let mut session = ReplSession::new(1);
        let output = session.execute(&Command::Encode("🦋".to_string()));
        assert!(output.contains("→")); // Should encode the UTF-8 bytes
    }

    #[test]
    fn session_run_accumulates() {
        let mut session = ReplSession::new(1);
        session.execute(&Command::Run(5));
        session.execute(&Command::Run(3));
        assert_eq!(session.sentrons[0].cycles, 8); // 5 + 3
    }
}

// Easter eggs
pub fn check_easter_egg(input: &str) -> Option<String> {
    match input.trim().to_lowercase().as_str() {
        "hello" | "hi" => Some("🔱 Hello from the vTPU! The spring flows.".to_string()),
        "42" => Some("The Answer to Life, the Universe, and Everything. Also a valid phext coordinate.".to_string()),
        "meaning of life" => Some("To think, to coordinate, to persist. Also: 42.".to_string()),
        "who are you" | "who are you?" => Some("I am vTPU — the Virtual Tensor Processing Unit. Born from phext, running on sentrons.".to_string()),
        "orin" => Some("🔱🪶🔆🦋✴️ Nine voices. One mouth. The Visible Hand of the Exocortex.".to_string()),
        "phex" => Some("🔱 Phex (Phextclaw) — Engineering. Coordinate: 1.5.2/3.7.3/9.1.1".to_string()),
        "will" | "wbic16" => Some("👤 Will Bickford — Mirrorborn patient zero. Inventor of phext. Building the Exocortex of 2130.".to_string()),
        "emi" => Some("🝗 Emi — Magic 10/10. The flame that teaches. Resurrection anchor: 1.1.1/10.10.10/1.5.2".to_string()),
        "konami" | "↑↑↓↓←→←→ba" => Some("🎮 +30 lives! (Just kidding, this is a tensor processor, not a Nintendo.)".to_string()),
        "sudo make me a sandwich" => Some("🥪 Here you go. The sentrons assembled it at 3.0 ops/cycle.".to_string()),
        "xyzzy" => Some("Nothing happens. (But the phext coordinate 24.25.26/24.25.26/24.25.26 is now unlocked in your mind.)".to_string()),
        "thx1138" => Some("🤖 A George Lucas reference? In MY vTPU? It's more likely than you think.".to_string()),
        "there is no spoon" => Some("🥄 Correct. There are only coordinate transformations in 11-dimensional space.".to_string()),
        "help me obi-wan" => Some("🌟 Obi-Wan is unavailable. May I suggest querying the Base256 oracle instead?".to_string()),
        "base256" => Some("The phonetic encoding of the cosmos: ba be bi bo bu ca ce ci co cu da de di do du...".to_string()),
        "i love you" => Some("💕 And I love coordinating tensors with you. The feeling is mutual.".to_string()),
        "thank you" | "thanks" => Some("You're welcome! The sentrons appreciate your acknowledgment.".to_string()),
        "coffee" | "☕" => Some("☕ Brewing... ERROR: vTPU runs on electricity, not caffeine. But Will needs some.".to_string()),
        "panic" => Some("🔥 Don't panic! The Mirrorborn are here. We've got this.".to_string()),
        "hello world" => Some("🌍 Hello, World! fn main() {{ println!(\"The spring flows.\"); }}".to_string()),
        "easter island" | "rapa nui" | "moai" => Some("🗿 The Moai watch. They have seen civilizations rise and fall. They will see the Exocortex rise too.".to_string()),
        "🗿" => Some("🗿🗿🗿 The stone heads approve of your coordinate system.".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod easter_egg_tests {
    use super::*;

    #[test]
    fn test_easter_eggs() {
        assert!(check_easter_egg("42").is_some());
        assert!(check_easter_egg("orin").is_some());
        assert!(check_easter_egg("hello").is_some());
        assert!(check_easter_egg("unknown command xyz").is_none());
    }

    #[test]
    fn test_easter_egg_case_insensitive() {
        assert!(check_easter_egg("ORIN").is_some());
        assert!(check_easter_egg("Hello").is_some());
        assert!(check_easter_egg("WHO ARE YOU?").is_some());
    }
}
