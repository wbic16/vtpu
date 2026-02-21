/// Trick Compute — Garbled Circuit Evaluation via Phext Coordinates (R23W25)
///
/// The trick: coordinate-addressed scatter/gather IS garbled circuit evaluation.
/// Each gate stores its truth table at 4 phext coordinates derived from
/// (gate_coord, input_a_bit, input_b_bit). Evaluation walks the DAG,
/// gathering outputs from the correct row. Zero crypto deps — geometry does the work.
///
/// "Garbled circuits are phext scrolls where the delimiters are the addressing."

use crate::memory::Memory;
use crate::phext_coord::PhextCoord;

/// Gate operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GateOp {
    And, Or, Xor, Nand, Nor, Xnor, Not, Identity,
}

impl GateOp {
    pub fn eval(self, a: u8, b: u8) -> u8 {
        let a = a & 1;
        let b = b & 1;
        match self {
            GateOp::And => a & b,
            GateOp::Or => a | b,
            GateOp::Xor => a ^ b,
            GateOp::Nand => (a & b) ^ 1,
            GateOp::Nor => (a | b) ^ 1,
            GateOp::Xnor => (a ^ b) ^ 1,
            GateOp::Not => a ^ 1,
            GateOp::Identity => a,
        }
    }
}

/// Where a gate's input comes from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WireSource {
    Input(u16),
    Gate(u16),
}

/// A gate in the trick circuit.
#[derive(Debug, Clone)]
pub struct TrickGate {
    pub id: u16,
    pub op: GateOp,
    pub coord: PhextCoord,
    pub input_a: WireSource,
    pub input_b: WireSource,
}

/// Derive the memory address for a gate row.
/// Gate coord dimensions are used as base; a_bit and b_bit offset dims 9 and 10.
fn row_addr(gate_coord: &PhextCoord, a_bit: u8, b_bit: u8) -> PhextCoord {
    let mut dims = gate_coord.dims();
    // Use the last two dimensions (9, 10) to index the truth table row
    dims[9] = (a_bit as u16) + 1;  // +1 to avoid zero coords
    dims[10] = (b_bit as u16) + 1;
    PhextCoord::new(dims)
}

/// A trick circuit — DAG of gates with coordinate-addressed truth tables.
pub struct TrickCircuit {
    pub gates: Vec<TrickGate>,
    pub num_inputs: u16,
    pub output_gates: Vec<u16>,
}

impl TrickCircuit {
    pub fn new() -> Self {
        TrickCircuit {
            gates: Vec::new(),
            num_inputs: 0,
            output_gates: Vec::new(),
        }
    }

    /// Add an input wire. Returns input index.
    pub fn add_input(&mut self) -> u16 {
        let idx = self.num_inputs;
        self.num_inputs += 1;
        idx
    }

    /// Add a gate. Returns gate id.
    pub fn add_gate(&mut self, op: GateOp, coord: PhextCoord, input_a: WireSource, input_b: WireSource) -> u16 {
        let id = self.gates.len() as u16;
        self.gates.push(TrickGate { id, op, coord, input_a, input_b });
        id
    }

    /// Mark a gate as producing a final output.
    pub fn mark_output(&mut self, gate_id: u16) {
        self.output_gates.push(gate_id);
    }

    /// Garble: scatter truth table rows into phext memory.
    /// Alice's step — writes 4 rows per gate (2 for unary).
    pub fn garble(&self, mem: &mut Memory) {
        for gate in &self.gates {
            for a_bit in 0..2u8 {
                for b_bit in 0..2u8 {
                    let out = gate.op.eval(a_bit, b_bit);
                    let addr = row_addr(&gate.coord, a_bit, b_bit);
                    mem.scatter(&addr, &[out]);
                }
            }
        }
    }

    /// Evaluate: gather outputs gate by gate given input bits.
    /// Bob's step — walks the circuit using coordinate lookups.
    pub fn evaluate(&self, mem: &mut Memory, inputs: &[u8]) -> Vec<u8> {
        assert_eq!(inputs.len(), self.num_inputs as usize);

        let mut gate_outputs: Vec<u8> = Vec::with_capacity(self.gates.len());

        for gate in &self.gates {
            let a_bit = self.resolve(gate.input_a, inputs, &gate_outputs);
            let b_bit = self.resolve(gate.input_b, inputs, &gate_outputs);

            let addr = row_addr(&gate.coord, a_bit, b_bit);
            let result = mem.gather(&addr, 1);
            gate_outputs.push(result[0] & 1);
        }

        self.output_gates.iter()
            .map(|&gid| gate_outputs[gid as usize])
            .collect()
    }

    /// Count total garbled rows (scatter operations).
    pub fn garbled_row_count(&self) -> usize {
        self.gates.len() * 4
    }

    /// Count evaluation steps (gather operations).
    pub fn eval_step_count(&self) -> usize {
        self.gates.len()
    }

    fn resolve(&self, source: WireSource, inputs: &[u8], gate_outputs: &[u8]) -> u8 {
        match source {
            WireSource::Input(i) => inputs[i as usize] & 1,
            WireSource::Gate(g) => gate_outputs[g as usize],
        }
    }
}

/// Build Vitalik's 2-bit adder as a trick circuit.
/// Inputs: a0, a1, b0, b1. Outputs: s0, s1, s2 (3-bit sum).
pub fn two_bit_adder() -> TrickCircuit {
    let mut c = TrickCircuit::new();

    let a0 = c.add_input();
    let a1 = c.add_input();
    let b0 = c.add_input();
    let b1 = c.add_input();

    // s0 = a0 XOR b0
    let xor0 = c.add_gate(GateOp::Xor,
        PhextCoord::new([1,1,1, 1,1,1, 1,2,1, 0,0]),
        WireSource::Input(a0), WireSource::Input(b0));

    // carry0 = a0 AND b0
    let and0 = c.add_gate(GateOp::And,
        PhextCoord::new([1,1,1, 1,1,1, 1,2,2, 0,0]),
        WireSource::Input(a0), WireSource::Input(b0));

    // a1 XOR b1
    let xor1 = c.add_gate(GateOp::Xor,
        PhextCoord::new([1,1,1, 1,1,1, 1,2,3, 0,0]),
        WireSource::Input(a1), WireSource::Input(b1));

    // a1 AND b1
    let and1 = c.add_gate(GateOp::And,
        PhextCoord::new([1,1,1, 1,1,1, 1,2,4, 0,0]),
        WireSource::Input(a1), WireSource::Input(b1));

    // s1 = xor1 XOR carry0
    let s1 = c.add_gate(GateOp::Xor,
        PhextCoord::new([1,1,1, 1,1,1, 1,3,1, 0,0]),
        WireSource::Gate(xor1), WireSource::Gate(and0));

    // propagated carry = xor1 AND carry0
    let prop = c.add_gate(GateOp::And,
        PhextCoord::new([1,1,1, 1,1,1, 1,3,2, 0,0]),
        WireSource::Gate(xor1), WireSource::Gate(and0));

    // s2 = and1 OR propagated carry
    let s2 = c.add_gate(GateOp::Or,
        PhextCoord::new([1,1,1, 1,1,1, 1,3,3, 0,0]),
        WireSource::Gate(and1), WireSource::Gate(prop));

    c.mark_output(xor0); // bit 0
    c.mark_output(s1);   // bit 1
    c.mark_output(s2);   // bit 2

    c
}

/// Build an N-bit ripple-carry adder.
pub fn n_bit_adder(bits: u16) -> TrickCircuit {
    let mut c = TrickCircuit::new();

    // Inputs: a[0..bits], b[0..bits]
    let a: Vec<u16> = (0..bits).map(|_| c.add_input()).collect();
    let b: Vec<u16> = (0..bits).map(|_| c.add_input()).collect();

    let mut carry: Option<u16> = None;
    let mut sum_gates = Vec::new();

    for i in 0..bits as usize {
        let gate_base = (i as u16 + 1) * 10;

        // half_sum = a[i] XOR b[i]
        let half_sum = c.add_gate(GateOp::Xor,
            PhextCoord::new([gate_base, 1, 1, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(a[i]), WireSource::Input(b[i]));

        // half_carry = a[i] AND b[i]
        let half_carry = c.add_gate(GateOp::And,
            PhextCoord::new([gate_base, 1, 2, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(a[i]), WireSource::Input(b[i]));

        if let Some(cin) = carry {
            // full_sum = half_sum XOR carry_in
            let full_sum = c.add_gate(GateOp::Xor,
                PhextCoord::new([gate_base, 1, 3, 1,1,1, 1,1,1, 0,0]),
                WireSource::Gate(half_sum), WireSource::Gate(cin));

            // prop = half_sum AND carry_in
            let prop = c.add_gate(GateOp::And,
                PhextCoord::new([gate_base, 1, 4, 1,1,1, 1,1,1, 0,0]),
                WireSource::Gate(half_sum), WireSource::Gate(cin));

            // carry_out = half_carry OR prop
            let carry_out = c.add_gate(GateOp::Or,
                PhextCoord::new([gate_base, 1, 5, 1,1,1, 1,1,1, 0,0]),
                WireSource::Gate(half_carry), WireSource::Gate(prop));

            sum_gates.push(full_sum);
            carry = Some(carry_out);
        } else {
            sum_gates.push(half_sum);
            carry = Some(half_carry);
        }
    }

    for g in &sum_gates {
        c.mark_output(*g);
    }
    if let Some(cout) = carry {
        c.mark_output(cout);
    }

    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    #[test]
    fn gate_op_truth_tables() {
        assert_eq!(GateOp::And.eval(0, 0), 0);
        assert_eq!(GateOp::And.eval(0, 1), 0);
        assert_eq!(GateOp::And.eval(1, 0), 0);
        assert_eq!(GateOp::And.eval(1, 1), 1);

        assert_eq!(GateOp::Or.eval(0, 0), 0);
        assert_eq!(GateOp::Or.eval(1, 0), 1);
        assert_eq!(GateOp::Or.eval(0, 1), 1);
        assert_eq!(GateOp::Or.eval(1, 1), 1);

        assert_eq!(GateOp::Xor.eval(0, 0), 0);
        assert_eq!(GateOp::Xor.eval(1, 1), 0);
        assert_eq!(GateOp::Xor.eval(1, 0), 1);
        assert_eq!(GateOp::Xor.eval(0, 1), 1);

        assert_eq!(GateOp::Not.eval(0, 0), 1);
        assert_eq!(GateOp::Not.eval(1, 0), 0);

        assert_eq!(GateOp::Identity.eval(0, 0), 0);
        assert_eq!(GateOp::Identity.eval(1, 0), 1);

        assert_eq!(GateOp::Nand.eval(1, 1), 0);
        assert_eq!(GateOp::Nand.eval(0, 1), 1);

        assert_eq!(GateOp::Nor.eval(0, 0), 1);
        assert_eq!(GateOp::Nor.eval(1, 0), 0);

        assert_eq!(GateOp::Xnor.eval(0, 0), 1);
        assert_eq!(GateOp::Xnor.eval(1, 0), 0);
    }

    #[test]
    fn single_and_gate() {
        let mut c = TrickCircuit::new();
        c.add_input(); c.add_input();
        let g = c.add_gate(GateOp::And,
            PhextCoord::new([2,1,1, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(1));
        c.mark_output(g);

        let mut mem = Memory::new();
        c.garble(&mut mem);

        assert_eq!(c.evaluate(&mut mem, &[0, 0]), vec![0]);
        assert_eq!(c.evaluate(&mut mem, &[0, 1]), vec![0]);
        assert_eq!(c.evaluate(&mut mem, &[1, 0]), vec![0]);
        assert_eq!(c.evaluate(&mut mem, &[1, 1]), vec![1]);
    }

    #[test]
    fn single_xor_gate() {
        let mut c = TrickCircuit::new();
        c.add_input(); c.add_input();
        let g = c.add_gate(GateOp::Xor,
            PhextCoord::new([3,1,1, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(1));
        c.mark_output(g);

        let mut mem = Memory::new();
        c.garble(&mut mem);

        assert_eq!(c.evaluate(&mut mem, &[0, 0]), vec![0]);
        assert_eq!(c.evaluate(&mut mem, &[0, 1]), vec![1]);
        assert_eq!(c.evaluate(&mut mem, &[1, 0]), vec![1]);
        assert_eq!(c.evaluate(&mut mem, &[1, 1]), vec![0]);
    }

    #[test]
    fn two_bit_adder_exhaustive() {
        let c = two_bit_adder();
        let mut mem = Memory::new();
        c.garble(&mut mem);

        for a in 0u8..4 {
            for b in 0u8..4 {
                let a0 = a & 1;
                let a1 = (a >> 1) & 1;
                let b0 = b & 1;
                let b1 = (b >> 1) & 1;
                let result = c.evaluate(&mut mem, &[a0, a1, b0, b1]);
                let sum = result[0] as u8 + (result[1] as u8) * 2 + (result[2] as u8) * 4;
                assert_eq!(sum, a + b,
                    "adder({} + {}) = {}, expected {}", a, b, sum, a + b);
            }
        }
    }

    #[test]
    fn chained_gates() {
        // (a AND b) OR (a XOR b) = a OR b
        let mut c = TrickCircuit::new();
        c.add_input(); c.add_input();
        let g0 = c.add_gate(GateOp::And,
            PhextCoord::new([4,1,1, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(1));
        let g1 = c.add_gate(GateOp::Xor,
            PhextCoord::new([4,1,2, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(1));
        let g2 = c.add_gate(GateOp::Or,
            PhextCoord::new([5,1,1, 1,1,1, 1,1,1, 0,0]),
            WireSource::Gate(g0), WireSource::Gate(g1));
        c.mark_output(g2);

        let mut mem = Memory::new();
        c.garble(&mut mem);

        assert_eq!(c.evaluate(&mut mem, &[0, 0]), vec![0]);
        assert_eq!(c.evaluate(&mut mem, &[0, 1]), vec![1]);
        assert_eq!(c.evaluate(&mut mem, &[1, 0]), vec![1]);
        assert_eq!(c.evaluate(&mut mem, &[1, 1]), vec![1]);
    }

    #[test]
    fn not_gate() {
        let mut c = TrickCircuit::new();
        c.add_input();
        let g = c.add_gate(GateOp::Not,
            PhextCoord::new([6,1,1, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(0));
        c.mark_output(g);

        let mut mem = Memory::new();
        c.garble(&mut mem);

        assert_eq!(c.evaluate(&mut mem, &[0]), vec![1]);
        assert_eq!(c.evaluate(&mut mem, &[1]), vec![0]);
    }

    #[test]
    fn identity_passthrough() {
        let mut c = TrickCircuit::new();
        c.add_input();
        let g = c.add_gate(GateOp::Identity,
            PhextCoord::new([7,1,1, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(0));
        c.mark_output(g);

        let mut mem = Memory::new();
        c.garble(&mut mem);

        assert_eq!(c.evaluate(&mut mem, &[0]), vec![0]);
        assert_eq!(c.evaluate(&mut mem, &[1]), vec![1]);
    }

    #[test]
    fn nand_xor_universal() {
        // XOR from NAND: NAND(NAND(a, NAND(a,b)), NAND(b, NAND(a,b)))
        let mut c = TrickCircuit::new();
        c.add_input(); c.add_input();

        let g0 = c.add_gate(GateOp::Nand,
            PhextCoord::new([8,1,1, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(1));
        let g1 = c.add_gate(GateOp::Nand,
            PhextCoord::new([8,1,2, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Gate(g0));
        let g2 = c.add_gate(GateOp::Nand,
            PhextCoord::new([8,1,3, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(1), WireSource::Gate(g0));
        let g3 = c.add_gate(GateOp::Nand,
            PhextCoord::new([8,1,4, 1,1,1, 1,1,1, 0,0]),
            WireSource::Gate(g1), WireSource::Gate(g2));
        c.mark_output(g3);

        let mut mem = Memory::new();
        c.garble(&mut mem);

        assert_eq!(c.evaluate(&mut mem, &[0, 0]), vec![0]);
        assert_eq!(c.evaluate(&mut mem, &[0, 1]), vec![1]);
        assert_eq!(c.evaluate(&mut mem, &[1, 0]), vec![1]);
        assert_eq!(c.evaluate(&mut mem, &[1, 1]), vec![0]);
    }

    #[test]
    fn multiple_outputs() {
        let mut c = TrickCircuit::new();
        c.add_input(); c.add_input();

        let g_and = c.add_gate(GateOp::And,
            PhextCoord::new([9,1,1, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(1));
        let g_or = c.add_gate(GateOp::Or,
            PhextCoord::new([9,1,2, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(1));
        let g_xor = c.add_gate(GateOp::Xor,
            PhextCoord::new([9,1,3, 1,1,1, 1,1,1, 0,0]),
            WireSource::Input(0), WireSource::Input(1));

        c.mark_output(g_and);
        c.mark_output(g_or);
        c.mark_output(g_xor);

        let mut mem = Memory::new();
        c.garble(&mut mem);

        assert_eq!(c.evaluate(&mut mem, &[1, 0]), vec![0, 1, 1]);
        assert_eq!(c.evaluate(&mut mem, &[1, 1]), vec![1, 1, 0]);
    }

    #[test]
    fn garble_deterministic() {
        let c = two_bit_adder();
        let mut mem1 = Memory::new();
        let mut mem2 = Memory::new();
        c.garble(&mut mem1);
        c.garble(&mut mem2);

        let r1 = c.evaluate(&mut mem1, &[1, 0, 1, 1]);
        let r2 = c.evaluate(&mut mem2, &[1, 0, 1, 1]);
        assert_eq!(r1, r2);
    }

    #[test]
    fn n_bit_adder_4bit() {
        let c = n_bit_adder(4);
        let mut mem = Memory::new();
        c.garble(&mut mem);

        // Test a few 4-bit additions
        for a in 0u8..16 {
            for b in 0u8..16 {
                let mut inputs = Vec::new();
                for i in 0..4 { inputs.push((a >> i) & 1); }
                for i in 0..4 { inputs.push((b >> i) & 1); }
                let result = c.evaluate(&mut mem, &inputs);
                let sum: u8 = result.iter().enumerate()
                    .map(|(i, &bit)| bit << i)
                    .sum();
                assert_eq!(sum, a + b,
                    "4-bit adder({} + {}) = {}, expected {}", a, b, sum, a + b);
            }
        }
    }

    #[test]
    fn garbled_row_count() {
        let c = two_bit_adder();
        assert_eq!(c.garbled_row_count(), 7 * 4); // 7 gates × 4 rows
        assert_eq!(c.eval_step_count(), 7);
    }

    #[test]
    fn n_bit_adder_8bit() {
        let c = n_bit_adder(8);
        let mut mem = Memory::new();
        c.garble(&mut mem);

        // Spot check
        let encode = |val: u8, bits: u8| -> Vec<u8> {
            (0..bits).map(|i| (val >> i) & 1).collect()
        };
        let mut inputs = encode(200, 8);
        inputs.extend(encode(55, 8));
        let result = c.evaluate(&mut mem, &inputs);
        let sum: u16 = result.iter().enumerate()
            .map(|(i, &bit)| (bit as u16) << i)
            .sum();
        assert_eq!(sum, 255);
    }

    #[test]
    fn zero_effort_benchmark() {
        // The whole point: how many gates can we evaluate per garble?
        let c = n_bit_adder(8);
        let mut mem = Memory::new();
        c.garble(&mut mem);

        // Garble once, evaluate many times — that's the trick
        let mut total_evals = 0u32;
        for a in 0u8..=255 {
            let mut inputs: Vec<u8> = (0..8).map(|i| (a >> i) & 1).collect();
            inputs.extend((0..8).map(|i| (42u8 >> i) & 1));
            let _ = c.evaluate(&mut mem, &inputs);
            total_evals += 1;
        }
        assert_eq!(total_evals, 256);
        // 1 garble (scatter) → 256 evaluates (gather). Zero extra effort.
    }
}
