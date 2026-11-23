//! Domain models for Digital Logic gates, wires, presets, truth tables,
//! Quine-McCluskey minimization, oscilloscope waveforms, 8-bit ALU, and HDL synthesis.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateKind {
    Input,
    Output,
    Not,
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Xnor,
    Dff,
    Clock,
    Seg7,
}

impl GateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
            Self::Not => "not",
            Self::And => "and",
            Self::Or => "or",
            Self::Nand => "nand",
            Self::Nor => "nor",
            Self::Xor => "xor",
            Self::Xnor => "xnor",
            Self::Dff => "dff",
            Self::Clock => "clock",
            Self::Seg7 => "seg7",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "input" | "in" => Some(Self::Input),
            "output" | "out" => Some(Self::Output),
            "not" | "inv" => Some(Self::Not),
            "and" => Some(Self::And),
            "or" => Some(Self::Or),
            "nand" => Some(Self::Nand),
            "nor" => Some(Self::Nor),
            "xor" => Some(Self::Xor),
            "xnor" => Some(Self::Xnor),
            "dff" | "flipflop" => Some(Self::Dff),
            "clock" | "clk" => Some(Self::Clock),
            "seg7" | "display" => Some(Self::Seg7),
            _ => None,
        }
    }

    /// Propagation delay in picoseconds (45nm CMOS node characterization).
    pub fn delay_ps(self) -> u32 {
        match self {
            Self::Input | Self::Output | Self::Clock => 10,
            Self::Not => 25,
            Self::Nand | Self::Nor => 35,
            Self::And | Self::Or => 55,
            Self::Xor | Self::Xnor => 85,
            Self::Dff => 120,
            Self::Seg7 => 40,
        }
    }

    /// Static CMOS transistor count estimate.
    pub fn cmos_transistors(self) -> u32 {
        match self {
            Self::Input | Self::Output => 0,
            Self::Not => 2,
            Self::Nand | Self::Nor => 4,
            Self::And | Self::Or => 6,
            Self::Xor | Self::Xnor => 12,
            Self::Dff => 20,
            Self::Clock => 8,
            Self::Seg7 => 28,
        }
    }

    /// Evaluate gate output given input boolean signals and current internal state.
    pub fn eval(self, inputs: &[bool], current_state: bool) -> bool {
        match self {
            Self::Input | Self::Clock | Self::Dff => current_state,
            Self::Output | Self::Seg7 => inputs.first().copied().unwrap_or(false),
            Self::Not => !inputs.first().copied().unwrap_or(false),
            Self::And => !inputs.is_empty() && inputs.iter().all(|&b| b),
            Self::Or => inputs.iter().any(|&b| b),
            Self::Nand => inputs.is_empty() || !inputs.iter().all(|&b| b),
            Self::Nor => !inputs.iter().any(|&b| b),
            Self::Xor => inputs.iter().filter(|&&b| b).count() % 2 == 1,
            Self::Xnor => inputs.iter().filter(|&&b| b).count() % 2 == 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateNode {
    pub id: String,
    pub label: String,
    pub kind: GateKind,
    pub state: bool,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireEdge {
    pub id: String,
    pub from_gate: String,
    pub to_gate: String,
    pub signal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitPreset {
    HalfAdder,
    FullAdder,
    SrLatch,
    Mux2To1,
    ParityChecker4Bit,
    Counter2Bit,
}

impl CircuitPreset {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "half_adder" | "halfadder" => Some(Self::HalfAdder),
            "full_adder" | "fulladder" => Some(Self::FullAdder),
            "sr_latch" | "srlatch" => Some(Self::SrLatch),
            "mux_2to1" | "mux2to1" | "mux" => Some(Self::Mux2To1),
            "parity_4bit" | "parity" => Some(Self::ParityChecker4Bit),
            "counter_2bit" | "counter" => Some(Self::Counter2Bit),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::HalfAdder => "half_adder",
            Self::FullAdder => "full_adder",
            Self::SrLatch => "sr_latch",
            Self::Mux2To1 => "mux_2to1",
            Self::ParityChecker4Bit => "parity_4bit",
            Self::Counter2Bit => "counter_2bit",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthTableRow {
    pub inputs: Vec<bool>,
    pub outputs: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthTableResult {
    pub input_labels: Vec<String>,
    pub output_labels: Vec<String>,
    pub rows: Vec<TruthTableRow>,
    pub canonical_sop: Vec<String>,
    pub minimized_sop: Vec<String>,
    pub prime_implicants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformChannel {
    pub gate_id: String,
    pub label: String,
    pub kind: GateKind,
    pub samples: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingTraceResult {
    pub ticks: usize,
    pub critical_path_ps: u32,
    pub max_clock_mhz: f64,
    pub total_transistors: u32,
    pub channels: Vec<WaveformChannel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AluOp {
    Add,
    Sub,
    And,
    Or,
    Xor,
    NotA,
    Shl,
    Shr,
    Cmp,
    IncA,
}

impl AluOp {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "add" | "+" => Some(Self::Add),
            "sub" | "-" => Some(Self::Sub),
            "and" | "&" => Some(Self::And),
            "or" | "|" => Some(Self::Or),
            "xor" | "^" => Some(Self::Xor),
            "not" | "nota" | "not_a" | "~" => Some(Self::NotA),
            "shl" | "<<" => Some(Self::Shl),
            "shr" | ">>" => Some(Self::Shr),
            "cmp" => Some(Self::Cmp),
            "inc" | "inca" | "inc_a" => Some(Self::IncA),
            _ => None,
        }
    }

    pub fn mnemonic(self) -> &'static str {
        match self {
            Self::Add => "ADD",
            Self::Sub => "SUB",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Xor => "XOR",
            Self::NotA => "NOT A",
            Self::Shl => "SHL",
            Self::Shr => "SHR",
            Self::Cmp => "CMP",
            Self::IncA => "INC A",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alu8Result {
    pub op: String,
    pub operand_a: u8,
    pub operand_b: u8,
    pub result: u8,
    pub binary_result: String,
    pub hex_result: String,
    pub signed_result: i8,
    pub zero_flag: bool,
    pub carry_flag: bool,
    pub negative_flag: bool,
    pub overflow_flag: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuRegisterFile {
    pub r0: u8,
    pub r1: u8,
    pub r2: u8,
    pub r3: u8,
    pub acc: u8,
    pub pc: u8,
    pub zero_flag: bool,
    pub carry_flag: bool,
    pub negative_flag: bool,
    pub overflow_flag: bool,
    pub last_op: String,
}

impl Default for CpuRegisterFile {
    fn default() -> Self {
        Self {
            r0: 0x2A, // 42
            r1: 0x15, // 21
            r2: 0x0F, // 15
            r3: 0x80, // 128 (-128)
            acc: 0x3F,
            pc: 0,
            zero_flag: false,
            carry_flag: false,
            negative_flag: false,
            overflow_flag: false,
            last_op: "ADD 0x2A, 0x15 -> 0x3F".to_string(),
        }
    }
}

/// Execute an 8-bit ALU operation with exact hardware status flags (Z, C, N, V).
pub fn execute_alu8(op: AluOp, a: u8, b: u8) -> Alu8Result {
    let (res, carry, overflow) = match op {
        AluOp::Add => {
            let wide = (a as u16) + (b as u16);
            let r = (wide & 0xFF) as u8;
            let c = wide > 0xFF;
            let v = ((!(a ^ b)) & (a ^ r) & 0x80) != 0;
            (r, c, v)
        }
        AluOp::Sub | AluOp::Cmp => {
            let (r, borrow) = a.overflowing_sub(b);
            let v = (((a ^ b) & (a ^ r)) & 0x80) != 0;
            (r, borrow, v)
        }
        AluOp::And => (a & b, false, false),
        AluOp::Or => (a | b, false, false),
        AluOp::Xor => (a ^ b, false, false),
        AluOp::NotA => (!a, false, false),
        AluOp::Shl => {
            let c = (a & 0x80) != 0;
            (a.wrapping_shl(1), c, false)
        }
        AluOp::Shr => {
            let c = (a & 0x01) != 0;
            (a.wrapping_shr(1), c, false)
        }
        AluOp::IncA => {
            let (r, c) = a.overflowing_add(1);
            let v = a == 0x7F;
            (r, c, v)
        }
    };

    Alu8Result {
        op: op.mnemonic().to_string(),
        operand_a: a,
        operand_b: b,
        result: res,
        binary_result: format!("{res:08b}"),
        hex_result: format!("0x{res:02X}"),
        signed_result: res as i8,
        zero_flag: res == 0,
        carry_flag: carry,
        negative_flag: (res & 0x80) != 0,
        overflow_flag: overflow,
    }
}

/// Quine-McCluskey Boolean minimizer over a list of active minterms (`0..2^num_vars`).
pub fn quine_mccluskey_minimize(var_names: &[String], minterms: &[usize]) -> (String, Vec<String>) {
    let n = var_names.len();
    if n == 0 || minterms.is_empty() {
        return ("0".to_string(), Vec::new());
    }
    let max_terms = 1usize << n;
    let mut active: Vec<usize> = minterms
        .iter()
        .copied()
        .filter(|&m| m < max_terms)
        .collect();
    active.sort_unstable();
    active.dedup();
    if active.len() == max_terms {
        return ("1".to_string(), vec!["1".to_string()]);
    }

    // Represent implicants as (bits, mask_of_dashes)
    let mut current: Vec<(usize, usize)> = active.iter().map(|&m| (m, 0usize)).collect();
    let mut prime_implicants: Vec<(usize, usize)> = Vec::new();

    loop {
        let mut next: Vec<(usize, usize)> = Vec::new();
        let mut used = vec![false; current.len()];

        for i in 0..current.len() {
            for j in (i + 1)..current.len() {
                if current[i].1 == current[j].1 {
                    let diff = (current[i].0 ^ current[j].0) & !current[i].1;
                    if diff.count_ones() == 1 {
                        used[i] = true;
                        used[j] = true;
                        let combined = (current[i].0 & !diff, current[i].1 | diff);
                        if !next.contains(&combined) {
                            next.push(combined);
                        }
                    }
                }
            }
        }

        for (idx, &u) in used.iter().enumerate() {
            if !u && !prime_implicants.contains(&current[idx]) {
                prime_implicants.push(current[idx]);
            }
        }

        if next.is_empty() {
            break;
        }
        current = next;
    }

    // Greedy prime implicant cover over original minterms
    let covers = |imp: (usize, usize), m: usize| -> bool { (m & !imp.1) == (imp.0 & !imp.1) };
    let mut uncovered = active;
    let mut chosen: Vec<(usize, usize)> = Vec::new();

    // Essential prime implicants first
    for &m in &uncovered.clone() {
        let covering: Vec<(usize, usize)> = prime_implicants
            .iter()
            .copied()
            .filter(|&pi| covers(pi, m))
            .collect();
        if covering.len() == 1 && !chosen.contains(&covering[0]) {
            chosen.push(covering[0]);
        }
    }
    uncovered.retain(|&m| !chosen.iter().any(|&c| covers(c, m)));

    while !uncovered.is_empty() {
        let best = prime_implicants
            .iter()
            .copied()
            .max_by_key(|&pi| uncovered.iter().filter(|&&m| covers(pi, m)).count());
        match best {
            Some(pi) => {
                let count = uncovered.iter().filter(|&&m| covers(pi, m)).count();
                if count == 0 {
                    break;
                }
                chosen.push(pi);
                uncovered.retain(|&m| !covers(pi, m));
            }
            None => break,
        }
    }

    let format_imp = |(bits, mask): (usize, usize)| -> String {
        let mut literals = Vec::new();
        for (i, name) in var_names.iter().enumerate() {
            let bit_pos = n - 1 - i;
            if (mask & (1 << bit_pos)) == 0 {
                if (bits & (1 << bit_pos)) != 0 {
                    literals.push(name.clone());
                } else {
                    literals.push(format!("~{name}"));
                }
            }
        }
        if literals.is_empty() {
            "1".to_string()
        } else {
            literals.join(" & ")
        }
    };

    let pi_strings: Vec<String> = prime_implicants.into_iter().map(format_imp).collect();
    let chosen_strings: Vec<String> = chosen.into_iter().map(format_imp).collect();
    let expr = if chosen_strings.len() <= 1 {
        chosen_strings
            .first()
            .cloned()
            .unwrap_or_else(|| "0".to_string())
    } else {
        chosen_strings
            .iter()
            .map(|s| format!("({s})"))
            .collect::<Vec<_>>()
            .join(" | ")
    };

    (expr, pi_strings)
}
