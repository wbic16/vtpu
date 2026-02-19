//! zuna_exec: Load a ZUNA SIW stream (JSON) and execute on sentron lattice.
//!
//! Usage: zuna_exec <stream.json>
//!
//! The stream JSON is produced by zuna/run_real_pipeline.py.
//! This binary is the Rust endpoint of the full ZUNA → vTPU pipeline.

use vtpu_runtime::{
    exec::run,
    memory::Memory,
    phext_coord::PhextCoord,
    pipes::{DenseOp, SparseOp, CoordOp},
    sentron::Sentron,
    siw::SIW,
    zuna::{run_zuna_pipeline, synthetic_alpha_observations, FreqBand},
};
use std::{env, fs, time::Instant};

// Minimal JSON value type — no serde dependency
#[derive(Debug)]
enum JVal {
    Str(String),
    Num(f64),
    Arr(Vec<JVal>),
    Obj(Vec<(String, JVal)>),
    Null,
}

impl JVal {
    fn as_str(&self) -> Option<&str> {
        if let JVal::Str(s) = self { Some(s.as_str()) } else { None }
    }
    fn as_u64(&self) -> Option<u64> {
        if let JVal::Num(n) = self { Some(*n as u64) } else { None }
    }
    #[allow(dead_code)]
    fn as_f64(&self) -> Option<f64> {
        if let JVal::Num(n) = self { Some(*n) } else { None }
    }
    fn as_array(&self) -> Option<&[JVal]> {
        if let JVal::Arr(a) = self { Some(a.as_slice()) } else { None }
    }
    fn get(&self, key: &str) -> &JVal {
        if let JVal::Obj(pairs) = self {
            for (k, v) in pairs { if k == key { return v; } }
        }
        &JVal::Null
    }
}

fn parse_json(s: &str) -> JVal {
    let s = s.trim();
    if s.starts_with('"') {
        return JVal::Str(s.trim_matches('"').to_string());
    }
    if s.starts_with('[') {
        let inner = &s[1..s.len()-1];
        return JVal::Arr(split_json_array(inner).into_iter().map(|e| parse_json(e.trim())).collect());
    }
    if s.starts_with('{') {
        let inner = &s[1..s.len()-1];
        let pairs = split_json_array(inner).into_iter().filter_map(|item| {
            let item = item.trim();
            let colon = find_colon(item)?;
            let key = item[..colon].trim().trim_matches('"').to_string();
            let val = parse_json(item[colon+1..].trim());
            Some((key, val))
        }).collect();
        return JVal::Obj(pairs);
    }
    if let Ok(n) = s.parse::<f64>() { return JVal::Num(n); }
    JVal::Null
}

fn find_colon(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_str = false;
    for (i, c) in s.char_indices() {
        if c == '"' { in_str = !in_str; }
        if in_str { continue; }
        match c { '{' | '[' => depth += 1, '}' | ']' => depth -= 1, ':' if depth == 0 => return Some(i), _ => {} }
    }
    None
}

fn split_json_array(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut start = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '"' { in_str = !in_str; }
        if in_str { i += 1; continue; }
        match c {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            ',' if depth == 0 => { parts.push(&s[start..i]); start = i + 1; }
            _ => {}
        }
        i += 1;
    }
    if start < s.len() { parts.push(&s[start..]); }
    parts
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // No stream file — run synthetic pipeline for demo
        println!("ZUNA → vTPU Execution Demo (synthetic 10-20 EEG)");
        println!("Usage: zuna_exec <stream.json>  (from run_real_pipeline.py)\n");
        run_synthetic_demo();
        return;
    }

    let path = &args[1];
    println!("ZUNA → vTPU Execution");
    println!("Loading stream: {path}");
    match run_stream_file(path) {
        Ok(()) => {},
        Err(e) => { eprintln!("Error: {e}"); std::process::exit(1); }
    }
}

fn run_synthetic_demo() {
    let obs = synthetic_alpha_observations();
    println!("Observations: {} (standard 10-20 electrodes, Alpha/Earth band)", obs.len());

    let t0 = Instant::now();
    let stats = run_zuna_pipeline(&obs, FreqBand::Alpha);
    let elapsed = t0.elapsed();

    println!("\nExecution complete:");
    println!("  Observations:         {}", stats.observations);
    println!("  SIWs retired:         {}", stats.siws_retired);
    println!("  Ops retired:          {}", stats.ops_retired);
    println!("  Topology fills:       {} (missing channel estimates)", stats.reconstruction_fills);
    println!("  Wall time:            {:.2}ms", elapsed.as_secs_f64() * 1000.0);

    let ns_per_siw = if stats.siws_retired > 0 {
        elapsed.as_nanos() as f64 / stats.siws_retired as f64
    } else { 0.0 };
    println!("  Throughput:           {:.1} ns/SIW", ns_per_siw);
}

fn run_stream_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let stream = parse_json(&content);

    let n_siws    = stream.get("n_siws").as_u64().unwrap_or(0);
    let n_ch      = stream.get("n_channels").as_u64().unwrap_or(0);
    let n_ts      = stream.get("n_timesteps").as_u64().unwrap_or(0);
    let band      = stream.get("band").as_str().unwrap_or("alpha");
    let source    = stream.get("source").as_str().unwrap_or("unknown");

    println!("  Source:   {source}");
    println!("  Band:     {band}");
    println!("  Channels: {n_ch}, Timesteps: {n_ts}, SIWs: {n_siws}\n");

    let siw_arr = stream.get("siws").as_array()
        .ok_or("missing 'siws' array")?;

    let mut program: Vec<SIW> = Vec::with_capacity(siw_arr.len());

    for entry in siw_arr {
        let d_op_str = entry.get("d_op").as_str().unwrap_or("DNOP");
        let s_op_str = entry.get("s_op").as_str().unwrap_or("SNOP");
        let coord_dims = entry.get("phext_addr").get("dims").as_array()
            .map(|arr| arr.iter().map(|v| v.as_u64().unwrap_or(1) as u16).collect::<Vec<_>>())
            .unwrap_or_else(|| vec![1u16; 11]);

        let mut dims = [1u16; 11];
        for (i, &v) in coord_dims.iter().enumerate().take(11) {
            dims[i] = v;
        }
        let phext = PhextCoord::new(dims.map(|x| x as u16));

        let d_op = parse_d_op(d_op_str);
        let s_op = parse_s_op(s_op_str);

        program.push(SIW::new(d_op, s_op, CoordOp::CNOP, phext));
    }

    let mut sentron = Sentron::new(0, PhextCoord::zero(), 0, 0);
    let mut mem = Memory::new();
    sentron.spawn(program);

    let t0 = Instant::now();
    let stats = run(&mut sentron, &mut mem);
    let elapsed = t0.elapsed();

    println!("Execution complete:");
    println!("  SIWs retired:   {}", stats.siws_retired);
    println!("  Ops retired:    {}", stats.ops_retired);
    println!("  D/S/C active:   {}/{}/{}", stats.d_ops, stats.s_ops, stats.c_ops);
    println!("  D/S/C nops:     {}/{}/{}", stats.d_nops, stats.s_nops, stats.c_nops);
    println!("  Wall time:      {:.2}ms", elapsed.as_secs_f64() * 1000.0);

    let ns_per_siw = if stats.siws_retired > 0 {
        elapsed.as_nanos() as f64 / stats.siws_retired as f64
    } else { 0.0 };
    println!("  Throughput:     {:.1} ns/SIW", ns_per_siw);

    // Probe memory at several EEG coordinates (dims 4-10 default to 1 from bridge)
    println!("\n  Memory probe (first timestep, alpha band):");
    let base_dims = [3u16, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1]; // Cz ~[3,3,3,1,1,1,1,1,1,1,1]
    let cz_coord = PhextCoord::new(base_dims);
    let cz_val = mem.gather_i64(&cz_coord);
    println!("  Cz  coord {:?}: {} (×0.001 μV → {:.3} μV)", &base_dims[..4], cz_val, cz_val as f64 / 1000.0);

    // Also probe Fp1 [3,1,3,1,...] and T4 [8,1,3,1,...]
    let fp1_coord = PhextCoord::new([3u16, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1]);
    let fp1_val = mem.gather_i64(&fp1_coord);
    println!("  Fp1 coord [3,1,3,1]: {} (→ {:.3} μV)", fp1_val, fp1_val as f64 / 1000.0);

    let t4_coord = PhextCoord::new([8u16, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1]);
    let t4_val = mem.gather_i64(&t4_coord);
    println!("  T4  coord [8,1,3,1]: {} (→ {:.3} μV)", t4_val, t4_val as f64 / 1000.0);

    Ok(())
}

/// Parse "DMOV rd imm" or "DADD rd rs1 rs2" or "DNOP" → DenseOp
fn parse_d_op(s: &str) -> DenseOp {
    let parts: Vec<&str> = s.split_whitespace().collect();
    match parts.as_slice() {
        ["DMOV", rd, imm] => {
            let rd = rd.parse::<u8>().unwrap_or(0);
            let imm = imm.parse::<i64>().unwrap_or(0);
            DenseOp::DMOV { rd, imm }
        }
        ["DADD", rd, rs1, rs2] => {
            let rd = rd.parse::<u8>().unwrap_or(0);
            let rs1 = rs1.parse::<u8>().unwrap_or(0);
            let rs2 = rs2.parse::<u8>().unwrap_or(0);
            DenseOp::DADD { rd, rs1, rs2 }
        }
        _ => DenseOp::DNOP,
    }
}

/// Parse "SSCATTR coord_idx rs width" or "SGATHER rd coord_idx width" or "SNOP" → SparseOp
fn parse_s_op(s: &str) -> SparseOp {
    let parts: Vec<&str> = s.split_whitespace().collect();
    match parts.as_slice() {
        ["SSCATTR", coord_idx, rs, width] => {
            SparseOp::SSCATTR {
                coord_idx: coord_idx.parse::<u8>().unwrap_or(0),
                rs: rs.parse::<u8>().unwrap_or(0),
                width: width.parse::<u16>().unwrap_or(8),
            }
        }
        ["SGATHER", rd, coord_idx, width] => {
            SparseOp::SGATHER {
                rd: rd.parse::<u8>().unwrap_or(0),
                coord_idx: coord_idx.parse::<u8>().unwrap_or(0),
                width: width.parse::<u16>().unwrap_or(8),
            }
        }
        _ => SparseOp::SNOP,
    }
}
