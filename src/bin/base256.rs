//! base256 — CLI tool for Base256 phonetic encoding
//!
//! Usage:
//!   base256 encode <bytes>      Encode hex bytes to syllables
//!   base256 decode <syllables>  Decode syllables to hex bytes
//!   base256 coord <coord>       Format a phext coordinate
//!   base256 table               Print full lookup table

use std::env;
use vtpu_runtime::base256::{encode, decode, decode_sequence, format_bytes};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "encode" | "e" => {
            if args.len() < 3 {
                eprintln!("Usage: base256 encode <hex-bytes>");
                eprintln!("Example: base256 encode 42A5FF");
                return;
            }
            cmd_encode(&args[2]);
        }
        "decode" | "d" => {
            if args.len() < 3 {
                eprintln!("Usage: base256 decode <syllables>");
                eprintln!("Example: base256 decode hef-red-zom");
                return;
            }
            cmd_decode(&args[2]);
        }
        "coord" | "c" => {
            if args.len() < 3 {
                eprintln!("Usage: base256 coord <coord>");
                eprintln!("Example: base256 coord 1.5.2/3.7.3/9.1.1");
                return;
            }
            cmd_coord(&args[2]);
        }
        "table" | "t" => {
            cmd_table();
        }
        "help" | "-h" | "--help" => {
            print_usage();
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("base256 — Phonetic encoding for byte streams");
    println!();
    println!("Usage:");
    println!("  base256 encode <hex-bytes>      Encode hex bytes to syllables");
    println!("  base256 decode <syllables>      Decode syllables to hex bytes");
    println!("  base256 coord <phext-coord>     Format a phext coordinate");
    println!("  base256 table                   Print full 0x00-0xFF lookup table");
    println!();
    println!("Examples:");
    println!("  base256 encode 42A5FF           → haf-red-zom");
    println!("  base256 decode haf-red-zom      → 42A5FF");
    println!("  base256 coord 1.5.2/3.7.3/9.1.1 → bad-bed-baf / bam-bem-bam / bid-bad-bad");
}

fn cmd_encode(hex: &str) {
    // Parse hex string to bytes
    let hex_clean = hex.replace([' ', '-', ',', ':', '.'], "");
    if hex_clean.len() % 2 != 0 {
        eprintln!("Error: Hex string must have even length");
        return;
    }

    let mut bytes = Vec::new();
    for i in (0..hex_clean.len()).step_by(2) {
        let byte_str = &hex_clean[i..i+2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(b) => bytes.push(b),
            Err(e) => {
                eprintln!("Error parsing hex '{}': {}", byte_str, e);
                return;
            }
        }
    }

    let syllables = format_bytes(&bytes);
    println!("{}", syllables);
}

fn cmd_decode(input: &str) {
    match decode_sequence(input) {
        Some(bytes) => {
            let hex: String = bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join("");
            println!("{}", hex);
        }
        None => {
            eprintln!("Error: Invalid syllable sequence");
        }
    }
}

fn cmd_coord(coord_str: &str) {
    // Parse coordinate like "1.5.2/3.7.3/9.1.1"
    // Split by / to get 3 groups, then by . to get dims
    let groups: Vec<&str> = coord_str.split('/').collect();
    if groups.len() != 3 {
        eprintln!("Error: Coordinate must have 3 groups separated by /");
        eprintln!("Example: 1.5.2/3.7.3/9.1.1");
        return;
    }

    let mut dims = Vec::new();
    for group in groups {
        for dim_str in group.split('.') {
            match dim_str.parse::<u16>() {
                Ok(d) => dims.push(d),
                Err(e) => {
                    eprintln!("Error parsing dimension '{}': {}", dim_str, e);
                    return;
                }
            }
        }
    }

    if dims.len() != 9 {
        eprintln!("Error: Coordinate must have 9 dimensions (got {})", dims.len());
        eprintln!("Example: 1.5.2/3.7.3/9.1.1");
        return;
    }

    // Encode each dimension's low byte
    let bytes: Vec<u8> = dims.iter().map(|&d| d as u8).collect();
    let syllables: Vec<String> = bytes.iter().map(|&b| encode(b)).collect();
    
    // Format as 3 groups
    let formatted = format!("{} / {} / {}",
        syllables[0..3].join("-"),
        syllables[3..6].join("-"),
        syllables[6..9].join("-")
    );
    
    println!("{}", formatted);
}

fn cmd_table() {
    println!("Base256 Full Lookup Table");
    println!();
    println!("| Hex  | Dec | Binary     | Syllable |");
    println!("|------|-----|------------|----------|");
    
    for byte in 0u8..=255u8 {
        let syllable = encode(byte);
        let binary = format!("{:08b}", byte);
        println!("| {:02X}   | {:3} | {} | {:8} |", byte, byte, binary, syllable);
    }
}
