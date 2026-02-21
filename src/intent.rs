/// Intent Parser — Natural Language → vTPU Commands (R23W25)
///
/// Zero dependencies. Pattern matching on keywords and structure.
/// Not an LLM — a deterministic parser that handles common phrasings.

use crate::repl::Command;

/// Parse natural English into a REPL Command.
pub fn parse_intent(input: &str) -> Command {
    let lower = input.trim().to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    if words.is_empty() {
        return Command::Unknown(String::new());
    }

    // Try exact REPL parse first (passthrough for power users)
    let exact = crate::repl::parse(input);
    if !matches!(exact, Command::Unknown(_)) {
        return exact;
    }

    // Quit intents
    if matches_any(&lower, &["bye", "goodbye", "see you", "shut down", "stop"]) {
        return Command::Quit;
    }

    // Help intents
    if matches_any(&lower, &["what can you do", "what commands", "how do i", "tell me about"]) {
        return Command::Help;
    }

    // Status intents
    if matches_any(&lower, &["how are we", "how's it going", "health", "overview", "what's happening"]) {
        return Command::Status;
    }

    // Fleet intents
    if matches_any(&lower, &["show fleet", "fleet status", "how's the fleet", "all sentrons", "show all"]) {
        return Command::Fleet;
    }

    // Memory intents
    if matches_any(&lower, &["how much memory", "memory usage", "memory status", "how full"]) {
        return Command::Memory;
    }

    // Sentron intents — "show sentron 3", "inspect sentron 5", "what is sentron 0"
    if let Some(id) = extract_sentron_id(&lower) {
        return Command::Sentron(id);
    }

    // Encode intents — "encode hello", "pronounce phext", "how do you say hello"
    if let Some(text) = extract_encode(&lower, input.trim()) {
        return Command::Encode(text);
    }

    // Decode intents — "decode bac wom", "what byte is bac"
    if let Some(syls) = extract_decode(&lower) {
        return Command::Decode(syls);
    }

    // Run intents — "run 10 cycles", "execute 5", "step 3 times"
    if let Some(n) = extract_run(&lower) {
        return Command::Run(n);
    }

    // Write intents — "write hello to 1.1.1/1.1.1/1.1.1", "store data at COORD"
    if let Some((coord, data)) = extract_write(&lower, input.trim()) {
        return Command::Write(coord, data);
    }

    // Read intents — "what's at 1.1.1/1.1.1/1.1.1", "read from COORD", "show COORD"
    if let Some(coord) = extract_coord_from_text(&lower) {
        return Command::Read(coord);
    }

    Command::Unknown(input.trim().to_string())
}

/// Check if input contains any of the given phrases.
fn matches_any(input: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| input.contains(p))
}

/// Extract a sentron ID from natural language.
fn extract_sentron_id(lower: &str) -> Option<usize> {
    // Patterns: "sentron N", "show sentron N", "inspect sentron N", "what is sentron N"
    if let Some(pos) = lower.find("sentron") {
        let after = &lower[pos + 7..];
        return extract_first_number(after);
    }
    // "show me #3", "inspect #5"
    if let Some(pos) = lower.find('#') {
        let after = &lower[pos + 1..];
        return extract_first_number(after);
    }
    None
}

/// Extract encode text.
fn extract_encode(lower: &str, original: &str) -> Option<String> {
    for prefix in &["encode ", "pronounce ", "how do you say ", "spell ", "base256 "] {
        if lower.starts_with(prefix) {
            let text = &original[prefix.len()..];
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    if lower.contains("in base 256") || lower.contains("in base256") {
        // Extract the subject before "in base"
        if let Some(pos) = lower.find(" in base") {
            let text = &original[..pos];
            let text = text.trim_start_matches("what is ").trim_start_matches("how is ");
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

/// Extract decode syllables.
fn extract_decode(lower: &str) -> Option<String> {
    for prefix in &["decode ", "what byte is ", "what bytes are "] {
        if lower.starts_with(prefix) {
            let syls = &lower[prefix.len()..];
            if !syls.is_empty() {
                return Some(syls.to_string());
            }
        }
    }
    None
}

/// Extract run count.
fn extract_run(lower: &str) -> Option<usize> {
    if lower.contains("run") || lower.contains("execute") || lower.contains("step") || lower.contains("cycle") {
        return extract_first_number(lower).or(Some(1));
    }
    None
}

/// Extract write intent: coordinate + data.
fn extract_write(lower: &str, original: &str) -> Option<(String, String)> {
    if !(lower.contains("write") || lower.contains("store") || lower.contains("put")) {
        return None;
    }

    let coord = extract_coord_from_text(lower)?;

    // Find the data portion — everything that's not the coord and not the verb
    let coord_pos = lower.find(&coord.to_lowercase())?;
    let remaining = original.replace(&original[coord_pos..coord_pos + coord.len()], "");
    let data: String = remaining
        .replace("write", "").replace("store", "").replace("put", "")
        .replace(" to ", " ").replace(" at ", " ").replace(" in ", " ")
        .trim().to_string();

    if data.is_empty() {
        return None;
    }

    Some((coord, data))
}

/// Extract a phext coordinate (L.S.Se/C.V.B/Ch.Sc.Scr) from free text.
fn extract_coord_from_text(text: &str) -> Option<String> {
    // Look for pattern: digits.digits.digits/digits.digits.digits/digits.digits.digits
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            // Try to match a full coordinate from here
            let start = i;
            if let Some(coord) = try_parse_coord_at(&text[start..]) {
                return Some(coord);
            }
        }
        i += 1;
    }
    None
}

/// Try to parse a coordinate starting at the given string position.
fn try_parse_coord_at(s: &str) -> Option<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut dots = 0;
    let mut slashes = 0;

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if ch == '.' && !current.is_empty() {
            parts.push(current.clone());
            current.clear();
            dots += 1;
        } else if ch == '/' && !current.is_empty() {
            parts.push(current.clone());
            current.clear();
            slashes += 1;
        } else {
            if !current.is_empty() {
                parts.push(current.clone());
            }
            break;
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }

    // Valid coordinate: 9 numbers, 6 dots, 2 slashes
    if parts.len() == 9 && slashes == 2 {
        Some(format!("{}.{}.{}/{}.{}.{}/{}.{}.{}",
            parts[0], parts[1], parts[2],
            parts[3], parts[4], parts[5],
            parts[6], parts[7], parts[8]))
    } else {
        None
    }
}

/// Extract the first number from a string.
fn extract_first_number(s: &str) -> Option<usize> {
    let mut num_str = String::new();
    let mut found = false;
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            found = true;
        } else if found {
            break;
        }
    }
    if found { num_str.parse().ok() } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Passthrough to REPL parser ===

    #[test]
    fn passthrough_status() {
        assert_eq!(parse_intent("status"), Command::Status);
    }

    #[test]
    fn passthrough_help() {
        assert_eq!(parse_intent("help"), Command::Help);
    }

    #[test]
    fn passthrough_quit() {
        assert_eq!(parse_intent("quit"), Command::Quit);
    }

    // === Natural language status ===

    #[test]
    fn natural_status() {
        assert_eq!(parse_intent("how are we"), Command::Status);
        assert_eq!(parse_intent("how's it going"), Command::Status);
        assert_eq!(parse_intent("health"), Command::Status);
    }

    // === Natural language fleet ===

    #[test]
    fn natural_fleet() {
        assert_eq!(parse_intent("show fleet"), Command::Fleet);
        assert_eq!(parse_intent("how's the fleet"), Command::Fleet);
    }

    // === Natural language memory ===

    #[test]
    fn natural_memory() {
        assert_eq!(parse_intent("how much memory"), Command::Memory);
        assert_eq!(parse_intent("memory usage"), Command::Memory);
    }

    // === Natural language quit ===

    #[test]
    fn natural_quit() {
        assert_eq!(parse_intent("bye"), Command::Quit);
        assert_eq!(parse_intent("goodbye"), Command::Quit);
        assert_eq!(parse_intent("see you"), Command::Quit);
    }

    // === Natural language help ===

    #[test]
    fn natural_help() {
        assert_eq!(parse_intent("what can you do"), Command::Help);
        assert_eq!(parse_intent("what commands"), Command::Help);
    }

    // === Sentron inspection ===

    #[test]
    fn natural_sentron() {
        assert_eq!(parse_intent("show sentron 3"), Command::Sentron(3));
        assert_eq!(parse_intent("inspect sentron 0"), Command::Sentron(0));
        assert_eq!(parse_intent("what is sentron 5"), Command::Sentron(5));
    }

    #[test]
    fn natural_sentron_hash() {
        assert_eq!(parse_intent("show me #7"), Command::Sentron(7));
    }

    // === Encode ===

    #[test]
    fn natural_encode() {
        assert_eq!(parse_intent("encode hello"), Command::Encode("hello".to_string()));
        assert_eq!(parse_intent("pronounce phext"), Command::Encode("phext".to_string()));
    }

    #[test]
    fn natural_encode_base256() {
        assert_eq!(parse_intent("how do you say hello"), Command::Encode("hello".to_string()));
    }

    // === Decode ===

    #[test]
    fn natural_decode() {
        assert_eq!(parse_intent("decode bac wom"), Command::Decode("bac wom".to_string()));
        assert_eq!(parse_intent("what byte is bac"), Command::Decode("bac".to_string()));
    }

    // === Run ===

    #[test]
    fn natural_run() {
        assert_eq!(parse_intent("run 10 cycles"), Command::Run(10));
        assert_eq!(parse_intent("execute 5"), Command::Run(5));
    }

    #[test]
    fn natural_run_default() {
        assert_eq!(parse_intent("step"), Command::Run(1));
    }

    // === Read via coordinate detection ===

    #[test]
    fn natural_read() {
        assert_eq!(parse_intent("what's at 1.1.1/1.1.1/1.1.2"), Command::Read("1.1.1/1.1.1/1.1.2".to_string()));
        assert_eq!(parse_intent("show me 3.3.3/5.5.5/7.7.7"), Command::Read("3.3.3/5.5.5/7.7.7".to_string()));
    }

    // === Coordinate extraction ===

    #[test]
    fn extract_coord_simple() {
        assert_eq!(extract_coord_from_text("read from 1.1.1/1.1.1/1.1.1 please"), Some("1.1.1/1.1.1/1.1.1".to_string()));
    }

    #[test]
    fn extract_coord_embedded() {
        assert_eq!(extract_coord_from_text("what's at coordinate 13.13.13/13.13.13/13.13.13?"), Some("13.13.13/13.13.13/13.13.13".to_string()));
    }

    #[test]
    fn extract_coord_none() {
        assert_eq!(extract_coord_from_text("no coordinate here"), None);
    }

    // === Number extraction ===

    #[test]
    fn extract_number() {
        assert_eq!(extract_first_number(" 42 things"), Some(42));
        assert_eq!(extract_first_number("no numbers"), None);
        assert_eq!(extract_first_number(" 0"), Some(0));
    }

    // === Edge cases ===

    #[test]
    fn empty_input() {
        assert_eq!(parse_intent(""), Command::Unknown(String::new()));
    }

    #[test]
    fn whitespace_only() {
        assert_eq!(parse_intent("   "), Command::Unknown(String::new()));
    }

    // === Edge cases: incomplete commands ===

    #[test]
    fn sentron_no_id() {
        // "sentron" alone with no number — should still parse as sentron 0 (default)
        assert_eq!(parse_intent("sentron"), Command::Sentron(0));
    }

    #[test]
    fn encode_empty_text() {
        // "encode" with no text
        assert_eq!(parse_intent("encode"), Command::Encode(String::new()));
    }

    #[test]
    fn decode_empty() {
        assert_eq!(parse_intent("decode"), Command::Decode(String::new()));
    }

    #[test]
    fn run_zero() {
        assert_eq!(parse_intent("run 0"), Command::Run(0));
    }

    #[test]
    fn run_large() {
        // Should parse without panic
        assert_eq!(parse_intent("run 999999"), Command::Run(999999));
    }

    // === Edge cases: injection attempts (should be harmless) ===

    #[test]
    fn semicolon_injection() {
        // Should not execute anything dangerous — just treated as text
        let cmd = parse_intent("status; rm -rf /");
        // Either parses as status (ignoring the rest) or as unknown
        assert!(matches!(cmd, Command::Status | Command::Unknown(_)));
    }

    #[test]
    fn shell_expansion() {
        let cmd = parse_intent("encode $(whoami)");
        assert_eq!(cmd, Command::Encode("$(whoami)".to_string()));
    }

    #[test]
    fn path_traversal() {
        // "../../etc/passwd" is not a valid phext coordinate
        let cmd = parse_intent("read ../../etc/passwd");
        assert_eq!(cmd, Command::Read("../../etc/passwd".to_string()));
    }

    // === Edge cases: unicode ===

    #[test]
    fn encode_emoji() {
        assert_eq!(parse_intent("encode 🦋"), Command::Encode("🦋".to_string()));
    }

    #[test]
    fn encode_accented() {
        assert_eq!(parse_intent("encode café"), Command::Encode("café".to_string()));
    }

    // === Edge cases: coordinate parsing ===

    #[test]
    fn coord_all_zeros() {
        let coord = extract_coord_from_text("read 0.0.0/0.0.0/0.0.0");
        assert_eq!(coord, Some("0.0.0/0.0.0/0.0.0".to_string()));
    }

    #[test]
    fn coord_max_dims() {
        let coord = extract_coord_from_text("2047.2047.2047/2047.2047.2047/2047.2047.2047");
        assert_eq!(coord, Some("2047.2047.2047/2047.2047.2047/2047.2047.2047".to_string()));
    }

    #[test]
    fn coord_partial_one_group() {
        // "1.1.1" alone is not a valid 9-dim coordinate
        assert_eq!(extract_coord_from_text("1.1.1"), None);
    }

    #[test]
    fn coord_partial_two_groups() {
        assert_eq!(extract_coord_from_text("1.1.1/1.1.1"), None);
    }

    #[test]
    fn coord_missing_dots() {
        // "1/1/1" is not valid (only 3 numbers, need 9)
        assert_eq!(extract_coord_from_text("1/1/1"), None);
    }

    // === Edge cases: bare coordinate as input ===

    #[test]
    fn bare_coordinate_reads() {
        // A bare coordinate should be treated as a read
        let cmd = parse_intent("1.1.1/1.1.1/1.1.1");
        assert_eq!(cmd, Command::Read("1.1.1/1.1.1/1.1.1".to_string()));
    }

    // === Edge cases: single words that aren't commands ===

    #[test]
    fn single_word_unknown() {
        assert!(matches!(parse_intent("what"), Command::Unknown(_)));
        assert!(matches!(parse_intent("the"), Command::Unknown(_)));
    }

    // === Edge cases: Base 256 decode case sensitivity ===

    #[test]
    fn decode_preserves_case() {
        // decode should work case-insensitively (handled by base256 module)
        let cmd = parse_intent("decode bac");
        assert_eq!(cmd, Command::Decode("bac".to_string()));
    }

    #[test]
    fn decode_multiple() {
        let cmd = parse_intent("decode bac wom bac");
        assert_eq!(cmd, Command::Decode("bac wom bac".to_string()));
    }

    // === Edge cases: negative numbers ===

    #[test]
    fn negative_sentron_id() {
        // "-1" — the negative sign should prevent number extraction
        let cmd = parse_intent("show sentron -1");
        // Should parse the 1, not crash
        assert_eq!(cmd, Command::Sentron(1));
    }
}
