//! asi — Interactive vTPU frontend
//!
//! A REPL for interacting with sentrons, memory, and the execution engine
//! in real-time. No deps beyond std.

use std::io::{self, Write, BufRead};
use vtpu_runtime::*;

struct Vm {
    sentrons: Vec<Sentron>,
    mem: Memory,
    active: usize,
}

impl Vm {
    fn new() -> Self {
        let mut vm = Vm { sentrons: Vec::new(), mem: Memory::new(), active: 0 };
        vm.sentrons.push(Sentron::new(0, PhextCoord::zero(), 0, 0));
        vm
    }
    fn s(&self) -> &Sentron { &self.sentrons[self.active] }
    fn s_mut(&mut self) -> &mut Sentron { &mut self.sentrons[self.active] }
}

fn help() {
    println!("
  asi — vTPU interactive frontend

  Memory:
    write <coord> <value>     Write i64 to phext coordinate (e.g. write 1.1.1 42)
    read <coord>              Read i64 from phext coordinate
    ppt                       Show PPT stats

  Registers:
    reg                       Show general registers (r0-r15)
    preg                      Show phext registers (p0-p7)
    set r<N> <value>          Set general register
    set p<N> <coord>          Set phext register

  Execution:
    mov r<D> <imm>            DMOV rd, imm
    add r<D> r<A> r<B>        DADD rd, rs1, rs2
    mul r<D> r<A> r<B>        DMUL rd, rs1, rs2
    sub r<D> r<A> r<B>        DSUB rd, rs1, rs2
    fma r<D> r<A> r<B> r<C>  DFMA rd, rs1, rs2, rs3
    gather r<D> p<C>          SGATHER rd, coord_idx
    scatter p<C> r<S>         SSCATTR coord_idx, rs

  Sentrons:
    spawn <id> <coord>        Create new sentron
    select <id>               Switch active sentron
    status                    Show all sentrons

  Other:
    help                      This message
    quit                      Exit
");
}

fn parse_coord(s: &str) -> Option<PhextCoord> {
    let nums: Vec<u16> = s.split(|c| c == '.' || c == '/')
        .filter_map(|n| n.parse().ok()).collect();
    if nums.is_empty() { return None; }
    let mut dims = [0u16; 11];
    for (i, &v) in nums.iter().enumerate().take(11) { dims[i] = v; }
    Some(PhextCoord::new(dims))
}

fn parse_reg(s: &str) -> Option<u8> {
    if s.starts_with('r') || s.starts_with('p') { s[1..].parse().ok() } else { None }
}

fn run1(vm: &mut Vm, siw: SIW) -> ExecStats {
    let idx = vm.active;
    vm.sentrons[idx].spawn(vec![siw]);
    exec_run(&mut vm.sentrons[idx], &mut vm.mem)
}

fn exec_line(vm: &mut Vm, line: &str) {
    let p: Vec<&str> = line.split_whitespace().collect();
    if p.is_empty() { return; }

    match p[0] {
        "help" | "h" | "?" => help(),
        "quit" | "exit" | "q" => std::process::exit(0),

        "write" | "w" if p.len() >= 3 => {
            if let (Some(c), Ok(v)) = (parse_coord(p[1]), p[2].parse::<i64>()) {
                vm.mem.scatter_i64(&c, v);
                println!("  wrote {} to {:?}", v, c);
            }
        }
        "read" if p.len() >= 2 => {
            if let Some(c) = parse_coord(p[1]) {
                println!("  {:?} = {}", c, vm.mem.gather_i64(&c));
            }
        }
        "ppt" => {
            let s = vm.mem.ppt.stats();
            println!("  hits: {} misses: {} rate: {:.1}% regions: {}", s.ptc_hits, s.ptc_misses, s.ptc_hit_rate * 100.0, s.regions_allocated);
        }
        "reg" => {
            let mut any = false;
            for i in 0..16 { if vm.s().regs.general[i] != 0 { println!("  r{:<2} = {}", i, vm.s().regs.general[i]); any = true; } }
            if !any { println!("  (all zero)"); }
        }
        "preg" => {
            for i in 0..8 { let c = vm.s().regs.phext[i]; if c != PhextCoord::zero() { println!("  p{} = {:?}", i, c); } }
        }
        "set" if p.len() >= 3 => {
            if let Some(reg) = parse_reg(p[1]) {
                if p[1].starts_with('r') {
                    if let Ok(v) = p[2].parse::<i64>() { vm.s_mut().regs.general[reg as usize] = v; println!("  r{} = {}", reg, v); }
                } else if let Some(c) = parse_coord(p[2]) {
                    vm.s_mut().regs.phext[reg as usize] = c; println!("  p{} = {:?}", reg, c);
                }
            }
        }

        "mov" if p.len() >= 3 => {
            if let (Some(rd), Ok(imm)) = (parse_reg(p[1]), p[2].parse::<i64>()) {
                let st = run1(vm, SIW::new(DenseOp::DMOV { rd, imm }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()));
                println!("  r{} = {} ({:.1} ops/cyc)", rd, imm, st.ops_per_cycle());
            }
        }
        "add" if p.len() >= 4 => {
            if let (Some(rd), Some(a), Some(b)) = (parse_reg(p[1]), parse_reg(p[2]), parse_reg(p[3])) {
                let st = run1(vm, SIW::new(DenseOp::DADD { rd, rs1: a, rs2: b }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()));
                println!("  r{} = {} ({:.1} ops/cyc)", rd, vm.s().regs.general[rd as usize], st.ops_per_cycle());
            }
        }
        "sub" if p.len() >= 4 => {
            if let (Some(rd), Some(a), Some(b)) = (parse_reg(p[1]), parse_reg(p[2]), parse_reg(p[3])) {
                let st = run1(vm, SIW::new(DenseOp::DSUB { rd, rs1: a, rs2: b }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()));
                println!("  r{} = {} ({:.1} ops/cyc)", rd, vm.s().regs.general[rd as usize], st.ops_per_cycle());
            }
        }
        "mul" if p.len() >= 4 => {
            if let (Some(rd), Some(a), Some(b)) = (parse_reg(p[1]), parse_reg(p[2]), parse_reg(p[3])) {
                let st = run1(vm, SIW::new(DenseOp::DMUL { rd, rs1: a, rs2: b }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()));
                println!("  r{} = {} ({:.1} ops/cyc)", rd, vm.s().regs.general[rd as usize], st.ops_per_cycle());
            }
        }
        "fma" if p.len() >= 5 => {
            if let (Some(rd), Some(a), Some(b), Some(c)) = (parse_reg(p[1]), parse_reg(p[2]), parse_reg(p[3]), parse_reg(p[4])) {
                let st = run1(vm, SIW::new(DenseOp::DFMA { rd, rs1: a, rs2: b, rs3: c }, SparseOp::SNOP, CoordOp::CNOP, PhextCoord::zero()));
                println!("  r{} = {} ({:.1} ops/cyc)", rd, vm.s().regs.general[rd as usize], st.ops_per_cycle());
            }
        }
        "gather" if p.len() >= 3 => {
            if let (Some(rd), Some(ci)) = (parse_reg(p[1]), parse_reg(p[2])) {
                run1(vm, SIW::new(DenseOp::DNOP, SparseOp::SGATHER { rd, coord_idx: ci, width: 8 }, CoordOp::CNOP, PhextCoord::zero()));
                println!("  r{} = {} (from p{}={:?})", rd, vm.s().regs.general[rd as usize], ci, vm.s().regs.phext[ci as usize]);
            }
        }
        "scatter" if p.len() >= 3 => {
            if let (Some(ci), Some(rs)) = (parse_reg(p[1]), parse_reg(p[2])) {
                let val = vm.s().regs.general[rs as usize];
                run1(vm, SIW::new(DenseOp::DNOP, SparseOp::SSCATTR { coord_idx: ci, rs, width: 8 }, CoordOp::CNOP, PhextCoord::zero()));
                println!("  scattered {} to {:?}", val, vm.s().regs.phext[ci as usize]);
            }
        }
        "spawn" if p.len() >= 3 => {
            if let (Ok(id), Some(c)) = (p[1].parse::<u16>(), parse_coord(p[2])) {
                vm.sentrons.push(Sentron::new(id, c, 0, 0));
                println!("  sentron {} at {:?}", id, c);
            }
        }
        "select" | "sel" if p.len() >= 2 => {
            if let Ok(i) = p[1].parse::<usize>() {
                if i < vm.sentrons.len() { vm.active = i; println!("  active: sentron {}", vm.sentrons[i].id); }
                else { println!("  no sentron at index {}", i); }
            }
        }
        "status" | "st" => {
            for (i, s) in vm.sentrons.iter().enumerate() {
                let m = if i == vm.active { "→" } else { " " };
                println!("  {} [{}] id={} home={:?} state={:?}", m, i, s.id, s.home, s.state);
            }
            let ppt = vm.mem.ppt.stats();
            println!("  mem: {} regions, {:.1}% PTC hit", ppt.regions_allocated, ppt.ptc_hit_rate * 100.0);
        }
        _ => println!("  unknown: '{}'. type 'help'", p[0]),
    }
}

fn main() {
    println!("asi — vTPU interactive frontend");
    println!("type 'help' for commands\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut vm = Vm::new();

    loop {
        print!("vtpu[{}]> ", vm.active);
        stdout.flush().unwrap();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap() == 0 { break; }
        exec_line(&mut vm, line.trim());
    }
}
