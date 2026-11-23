//! LogicCraft L1 Core Engine & Universal Command Bus.
//!
//! Pure-Rust digital logic circuit simulator, Quine-McCluskey / Karnaugh Map
//! Boolean synthesizer, multi-tick oscilloscope timing analyzer, 8-bit ALU & CPU
//! register workbench, and structural Verilog-2001 / VHDL / DOT netlist exporter.
#![forbid(unsafe_code)]

pub mod db;
pub mod error;
pub mod model;

pub use db::{default_db_path, load_from_db, save_to_db};
pub use error::LogicError;
pub use model::{
    execute_alu8, quine_mccluskey_minimize, Alu8Result, AluOp, CircuitPreset, CpuRegisterFile,
    GateKind, GateNode, TimingTraceResult, TruthTableResult, TruthTableRow, WaveformChannel,
    WireEdge,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicEngine {
    pub active_preset: String,
    pub gates: Vec<GateNode>,
    pub wires: Vec<WireEdge>,
    pub tick_count: u64,
    pub registers: CpuRegisterFile,
}

impl Default for LogicEngine {
    fn default() -> Self {
        Self::with_preset(CircuitPreset::FullAdder)
    }
}

impl LogicEngine {
    pub fn open_or_seed(path: &Path) -> Result<Self, LogicError> {
        if let Some(loaded) = load_from_db(path)? {
            return Ok(loaded);
        }
        let seeded = Self::with_preset(CircuitPreset::FullAdder);
        save_to_db(&seeded, path)?;
        Ok(seeded)
    }

    pub fn save_db(&self, path: &Path) -> Result<(), LogicError> {
        save_to_db(self, path)
    }

    pub fn with_preset(preset: CircuitPreset) -> Self {
        let mut eng = Self {
            active_preset: preset.as_str().to_string(),
            gates: Vec::new(),
            wires: Vec::new(),
            tick_count: 0,
            registers: CpuRegisterFile::default(),
        };
        eng.load_preset(preset);
        eng
    }

    pub fn load_preset(&mut self, preset: CircuitPreset) {
        self.gates.clear();
        self.wires.clear();
        self.active_preset = preset.as_str().to_string();

        match preset {
            CircuitPreset::HalfAdder => {
                self.gates = vec![
                    GateNode {
                        id: "in_a".into(),
                        label: "A".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 90.0,
                        y: 120.0,
                    },
                    GateNode {
                        id: "in_b".into(),
                        label: "B".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 90.0,
                        y: 240.0,
                    },
                    GateNode {
                        id: "g_xor".into(),
                        label: "XOR_SUM".into(),
                        kind: GateKind::Xor,
                        state: false,
                        x: 310.0,
                        y: 120.0,
                    },
                    GateNode {
                        id: "g_and".into(),
                        label: "AND_CARRY".into(),
                        kind: GateKind::And,
                        state: false,
                        x: 310.0,
                        y: 240.0,
                    },
                    GateNode {
                        id: "out_sum".into(),
                        label: "SUM".into(),
                        kind: GateKind::Output,
                        state: false,
                        x: 530.0,
                        y: 120.0,
                    },
                    GateNode {
                        id: "out_cout".into(),
                        label: "COUT".into(),
                        kind: GateKind::Output,
                        state: false,
                        x: 530.0,
                        y: 240.0,
                    },
                ];
                for (id, f, t) in [
                    ("w1", "in_a", "g_xor"),
                    ("w2", "in_b", "g_xor"),
                    ("w3", "in_a", "g_and"),
                    ("w4", "in_b", "g_and"),
                    ("w5", "g_xor", "out_sum"),
                    ("w6", "g_and", "out_cout"),
                ] {
                    self.wires.push(WireEdge {
                        id: id.into(),
                        from_gate: f.into(),
                        to_gate: t.into(),
                        signal: false,
                    });
                }
            }
            CircuitPreset::FullAdder => {
                self.gates = vec![
                    GateNode {
                        id: "in_a".into(),
                        label: "A".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 80.0,
                        y: 90.0,
                    },
                    GateNode {
                        id: "in_b".into(),
                        label: "B".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 80.0,
                        y: 190.0,
                    },
                    GateNode {
                        id: "in_cin".into(),
                        label: "CIN".into(),
                        kind: GateKind::Input,
                        state: false,
                        x: 80.0,
                        y: 290.0,
                    },
                    GateNode {
                        id: "xor1".into(),
                        label: "XOR1".into(),
                        kind: GateKind::Xor,
                        state: false,
                        x: 250.0,
                        y: 110.0,
                    },
                    GateNode {
                        id: "and1".into(),
                        label: "AND1".into(),
                        kind: GateKind::And,
                        state: false,
                        x: 250.0,
                        y: 220.0,
                    },
                    GateNode {
                        id: "xor2".into(),
                        label: "XOR2".into(),
                        kind: GateKind::Xor,
                        state: false,
                        x: 430.0,
                        y: 130.0,
                    },
                    GateNode {
                        id: "and2".into(),
                        label: "AND2".into(),
                        kind: GateKind::And,
                        state: false,
                        x: 430.0,
                        y: 250.0,
                    },
                    GateNode {
                        id: "or_c".into(),
                        label: "OR_CARRY".into(),
                        kind: GateKind::Or,
                        state: false,
                        x: 570.0,
                        y: 250.0,
                    },
                    GateNode {
                        id: "out_sum".into(),
                        label: "SUM".into(),
                        kind: GateKind::Output,
                        state: false,
                        x: 700.0,
                        y: 130.0,
                    },
                    GateNode {
                        id: "out_cout".into(),
                        label: "COUT".into(),
                        kind: GateKind::Output,
                        state: false,
                        x: 700.0,
                        y: 250.0,
                    },
                ];
                for (id, f, t) in [
                    ("w1", "in_a", "xor1"),
                    ("w2", "in_b", "xor1"),
                    ("w3", "in_a", "and1"),
                    ("w4", "in_b", "and1"),
                    ("w5", "xor1", "xor2"),
                    ("w6", "in_cin", "xor2"),
                    ("w7", "xor1", "and2"),
                    ("w8", "in_cin", "and2"),
                    ("w9", "and1", "or_c"),
                    ("w10", "and2", "or_c"),
                    ("w11", "xor2", "out_sum"),
                    ("w12", "or_c", "out_cout"),
                ] {
                    self.wires.push(WireEdge {
                        id: id.into(),
                        from_gate: f.into(),
                        to_gate: t.into(),
                        signal: false,
                    });
                }
            }
            CircuitPreset::SrLatch => {
                self.gates = vec![
                    GateNode {
                        id: "in_r".into(),
                        label: "R".into(),
                        kind: GateKind::Input,
                        state: false,
                        x: 90.0,
                        y: 110.0,
                    },
                    GateNode {
                        id: "in_s".into(),
                        label: "S".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 90.0,
                        y: 250.0,
                    },
                    GateNode {
                        id: "nor_q".into(),
                        label: "NOR_Q".into(),
                        kind: GateKind::Nor,
                        state: true,
                        x: 320.0,
                        y: 110.0,
                    },
                    GateNode {
                        id: "nor_nq".into(),
                        label: "NOR_NQ".into(),
                        kind: GateKind::Nor,
                        state: false,
                        x: 320.0,
                        y: 250.0,
                    },
                    GateNode {
                        id: "out_q".into(),
                        label: "Q".into(),
                        kind: GateKind::Output,
                        state: true,
                        x: 540.0,
                        y: 110.0,
                    },
                    GateNode {
                        id: "out_nq".into(),
                        label: "NQ".into(),
                        kind: GateKind::Output,
                        state: false,
                        x: 540.0,
                        y: 250.0,
                    },
                ];
                for (id, f, t) in [
                    ("w1", "in_r", "nor_q"),
                    ("w2", "nor_nq", "nor_q"),
                    ("w3", "in_s", "nor_nq"),
                    ("w4", "nor_q", "nor_nq"),
                    ("w5", "nor_q", "out_q"),
                    ("w6", "nor_nq", "out_nq"),
                ] {
                    self.wires.push(WireEdge {
                        id: id.into(),
                        from_gate: f.into(),
                        to_gate: t.into(),
                        signal: false,
                    });
                }
            }
            CircuitPreset::Mux2To1 => {
                self.gates = vec![
                    GateNode {
                        id: "in_d0".into(),
                        label: "D0".into(),
                        kind: GateKind::Input,
                        state: false,
                        x: 80.0,
                        y: 90.0,
                    },
                    GateNode {
                        id: "in_d1".into(),
                        label: "D1".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 80.0,
                        y: 190.0,
                    },
                    GateNode {
                        id: "in_sel".into(),
                        label: "SEL".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 80.0,
                        y: 290.0,
                    },
                    GateNode {
                        id: "not_s".into(),
                        label: "NOT_SEL".into(),
                        kind: GateKind::Not,
                        state: false,
                        x: 240.0,
                        y: 290.0,
                    },
                    GateNode {
                        id: "and0".into(),
                        label: "AND0".into(),
                        kind: GateKind::And,
                        state: false,
                        x: 400.0,
                        y: 110.0,
                    },
                    GateNode {
                        id: "and1".into(),
                        label: "AND1".into(),
                        kind: GateKind::And,
                        state: false,
                        x: 400.0,
                        y: 230.0,
                    },
                    GateNode {
                        id: "or_y".into(),
                        label: "OR_MUX".into(),
                        kind: GateKind::Or,
                        state: false,
                        x: 550.0,
                        y: 170.0,
                    },
                    GateNode {
                        id: "out_y".into(),
                        label: "Y".into(),
                        kind: GateKind::Output,
                        state: false,
                        x: 690.0,
                        y: 170.0,
                    },
                ];
                for (id, f, t) in [
                    ("w1", "in_sel", "not_s"),
                    ("w2", "in_d0", "and0"),
                    ("w3", "not_s", "and0"),
                    ("w4", "in_d1", "and1"),
                    ("w5", "in_sel", "and1"),
                    ("w6", "and0", "or_y"),
                    ("w7", "and1", "or_y"),
                    ("w8", "or_y", "out_y"),
                ] {
                    self.wires.push(WireEdge {
                        id: id.into(),
                        from_gate: f.into(),
                        to_gate: t.into(),
                        signal: false,
                    });
                }
            }
            CircuitPreset::ParityChecker4Bit => {
                self.gates = vec![
                    GateNode {
                        id: "b0".into(),
                        label: "B0".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 80.0,
                        y: 70.0,
                    },
                    GateNode {
                        id: "b1".into(),
                        label: "B1".into(),
                        kind: GateKind::Input,
                        state: false,
                        x: 80.0,
                        y: 150.0,
                    },
                    GateNode {
                        id: "b2".into(),
                        label: "B2".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 80.0,
                        y: 230.0,
                    },
                    GateNode {
                        id: "b3".into(),
                        label: "B3".into(),
                        kind: GateKind::Input,
                        state: true,
                        x: 80.0,
                        y: 310.0,
                    },
                    GateNode {
                        id: "x01".into(),
                        label: "XOR_01".into(),
                        kind: GateKind::Xor,
                        state: false,
                        x: 270.0,
                        y: 110.0,
                    },
                    GateNode {
                        id: "x23".into(),
                        label: "XOR_23".into(),
                        kind: GateKind::Xor,
                        state: false,
                        x: 270.0,
                        y: 270.0,
                    },
                    GateNode {
                        id: "xp".into(),
                        label: "XOR_PARITY".into(),
                        kind: GateKind::Xor,
                        state: false,
                        x: 460.0,
                        y: 190.0,
                    },
                    GateNode {
                        id: "out_p".into(),
                        label: "ODD_PARITY".into(),
                        kind: GateKind::Output,
                        state: false,
                        x: 640.0,
                        y: 190.0,
                    },
                ];
                for (id, f, t) in [
                    ("w1", "b0", "x01"),
                    ("w2", "b1", "x01"),
                    ("w3", "b2", "x23"),
                    ("w4", "b3", "x23"),
                    ("w5", "x01", "xp"),
                    ("w6", "x23", "xp"),
                    ("w7", "xp", "out_p"),
                ] {
                    self.wires.push(WireEdge {
                        id: id.into(),
                        from_gate: f.into(),
                        to_gate: t.into(),
                        signal: false,
                    });
                }
            }
            CircuitPreset::Counter2Bit => {
                self.gates = vec![
                    GateNode {
                        id: "clk".into(),
                        label: "CLK".into(),
                        kind: GateKind::Clock,
                        state: false,
                        x: 80.0,
                        y: 180.0,
                    },
                    GateNode {
                        id: "dff0".into(),
                        label: "Q0_DFF".into(),
                        kind: GateKind::Dff,
                        state: false,
                        x: 260.0,
                        y: 120.0,
                    },
                    GateNode {
                        id: "not0".into(),
                        label: "NOT_Q0".into(),
                        kind: GateKind::Not,
                        state: true,
                        x: 260.0,
                        y: 240.0,
                    },
                    GateNode {
                        id: "dff1".into(),
                        label: "Q1_DFF".into(),
                        kind: GateKind::Dff,
                        state: false,
                        x: 460.0,
                        y: 120.0,
                    },
                    GateNode {
                        id: "xor1".into(),
                        label: "XOR_D1".into(),
                        kind: GateKind::Xor,
                        state: false,
                        x: 460.0,
                        y: 240.0,
                    },
                    GateNode {
                        id: "out_q0".into(),
                        label: "Q0".into(),
                        kind: GateKind::Output,
                        state: false,
                        x: 650.0,
                        y: 120.0,
                    },
                    GateNode {
                        id: "out_q1".into(),
                        label: "Q1".into(),
                        kind: GateKind::Output,
                        state: false,
                        x: 650.0,
                        y: 240.0,
                    },
                ];
                for (id, f, t) in [
                    ("w1", "dff0", "not0"),
                    ("w2", "not0", "dff0"),
                    ("w3", "dff0", "xor1"),
                    ("w4", "dff1", "xor1"),
                    ("w5", "xor1", "dff1"),
                    ("w6", "dff0", "out_q0"),
                    ("w7", "dff1", "out_q1"),
                ] {
                    self.wires.push(WireEdge {
                        id: id.into(),
                        from_gate: f.into(),
                        to_gate: t.into(),
                        signal: false,
                    });
                }
            }
        }
        let _ = self.evaluate_circuit();
    }

    /// Evaluate combinational relaxation loop up to 16 passes.
    pub fn evaluate_circuit(&mut self) -> usize {
        let mut passes = 0;
        for _ in 0..16 {
            passes += 1;
            let mut changed = false;
            let state_map: HashMap<String, bool> =
                self.gates.iter().map(|g| (g.id.clone(), g.state)).collect();

            for w in &mut self.wires {
                let s = state_map.get(&w.from_gate).copied().unwrap_or(false);
                w.signal = s;
            }

            let mut next_states = Vec::with_capacity(self.gates.len());
            for g in &self.gates {
                let inputs: Vec<bool> = self
                    .wires
                    .iter()
                    .filter(|w| w.to_gate == g.id)
                    .map(|w| w.signal)
                    .collect();
                let next = g.kind.eval(&inputs, g.state);
                next_states.push(next);
            }

            for (g, next) in self.gates.iter_mut().zip(next_states) {
                if !matches!(g.kind, GateKind::Input | GateKind::Clock | GateKind::Dff)
                    && g.state != next
                {
                    g.state = next;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        // Final wire sync
        let state_map: HashMap<String, bool> =
            self.gates.iter().map(|g| (g.id.clone(), g.state)).collect();
        for w in &mut self.wires {
            w.signal = state_map.get(&w.from_gate).copied().unwrap_or(false);
        }
        passes
    }

    /// Advance sequential clock & DFF registers by 1 tick, then relax combinational logic.
    pub fn step_clock(&mut self) {
        self.tick_count += 1;
        // Sample DFF data inputs before latching
        let dff_next: Vec<(String, bool)> = self
            .gates
            .iter()
            .filter(|g| g.kind == GateKind::Dff)
            .map(|g| {
                let d_in = self
                    .wires
                    .iter()
                    .find(|w| w.to_gate == g.id)
                    .map(|w| w.signal)
                    .unwrap_or(!g.state);
                (g.id.clone(), d_in)
            })
            .collect();

        for g in &mut self.gates {
            if g.kind == GateKind::Clock {
                g.state = !g.state;
            }
            if let Some((_, val)) = dff_next.iter().find(|(id, _)| *id == g.id) {
                g.state = *val;
            }
        }
        let _ = self.evaluate_circuit();
    }

    pub fn toggle_input(&mut self, gate_id: &str) -> Result<bool, LogicError> {
        let g = self
            .gates
            .iter_mut()
            .find(|g| g.id == gate_id || g.label.eq_ignore_ascii_case(gate_id))
            .ok_or_else(|| LogicError::NotFound(gate_id.to_string()))?;
        g.state = !g.state;
        let new_state = g.state;
        let _ = self.evaluate_circuit();
        Ok(new_state)
    }

    pub fn add_gate(
        &mut self,
        id: &str,
        label: &str,
        kind: GateKind,
        x: f32,
        y: f32,
    ) -> Result<GateNode, LogicError> {
        let clean_id = id.trim();
        if clean_id.is_empty() {
            return Err(LogicError::Validation("Gate ID cannot be empty".into()));
        }
        if self.gates.iter().any(|g| g.id == clean_id) {
            return Err(LogicError::Validation(format!(
                "Gate '{clean_id}' already exists"
            )));
        }
        let node = GateNode {
            id: clean_id.to_string(),
            label: if label.trim().is_empty() {
                clean_id.to_uppercase()
            } else {
                label.trim().to_string()
            },
            kind,
            state: false,
            x,
            y,
        };
        self.gates.push(node.clone());
        let _ = self.evaluate_circuit();
        Ok(node)
    }

    pub fn remove_gate(&mut self, gate_id: &str) -> Result<(), LogicError> {
        let before = self.gates.len();
        self.gates.retain(|g| g.id != gate_id);
        if self.gates.len() == before {
            return Err(LogicError::NotFound(gate_id.to_string()));
        }
        self.wires
            .retain(|w| w.from_gate != gate_id && w.to_gate != gate_id);
        let _ = self.evaluate_circuit();
        Ok(())
    }

    pub fn connect_wire(&mut self, from_gate: &str, to_gate: &str) -> Result<WireEdge, LogicError> {
        if !self.gates.iter().any(|g| g.id == from_gate) {
            return Err(LogicError::NotFound(from_gate.to_string()));
        }
        if !self.gates.iter().any(|g| g.id == to_gate) {
            return Err(LogicError::NotFound(to_gate.to_string()));
        }
        let id = format!("w_{from_gate}_{to_gate}");
        if !self.wires.iter().any(|w| w.id == id) {
            self.wires.push(WireEdge {
                id: id.clone(),
                from_gate: from_gate.to_string(),
                to_gate: to_gate.to_string(),
                signal: false,
            });
        }
        let _ = self.evaluate_circuit();
        self.wires
            .iter()
            .find(|w| w.id == id)
            .cloned()
            .ok_or(LogicError::NotFound(id))
    }

    pub fn generate_truth_table(&self) -> Result<TruthTableResult, LogicError> {
        let mut clone = self.clone();
        let input_ids: Vec<(String, String)> = clone
            .gates
            .iter()
            .filter(|g| g.kind == GateKind::Input)
            .map(|g| (g.id.clone(), g.label.clone()))
            .collect();
        let output_ids: Vec<(String, String)> = clone
            .gates
            .iter()
            .filter(|g| matches!(g.kind, GateKind::Output | GateKind::Seg7))
            .map(|g| (g.id.clone(), g.label.clone()))
            .collect();

        if input_ids.is_empty() {
            return Err(LogicError::Validation(
                "Circuit has no Input gates to sweep for a truth table".into(),
            ));
        }
        if input_ids.len() > 8 {
            return Err(LogicError::Validation(
                "Truth table exhaustive sweep supports at most 8 input variables (256 rows)".into(),
            ));
        }

        let n = input_ids.len();
        let num_rows = 1usize << n;
        let mut rows = Vec::with_capacity(num_rows);
        let mut minterms_per_output: Vec<Vec<usize>> = vec![Vec::new(); output_ids.len()];

        for mask in 0..num_rows {
            let mut in_vec = Vec::with_capacity(n);
            for (idx, (gid, _)) in input_ids.iter().enumerate() {
                let bit = ((mask >> (n - 1 - idx)) & 1) != 0;
                in_vec.push(bit);
                if let Some(g) = clone.gates.iter_mut().find(|g| g.id == *gid) {
                    g.state = bit;
                }
            }
            let _ = clone.evaluate_circuit();

            let mut out_vec = Vec::with_capacity(output_ids.len());
            for (oidx, (oid, _)) in output_ids.iter().enumerate() {
                let val = clone
                    .gates
                    .iter()
                    .find(|g| g.id == *oid)
                    .map(|g| g.state)
                    .unwrap_or(false);
                out_vec.push(val);
                if val {
                    minterms_per_output[oidx].push(mask);
                }
            }
            rows.push(TruthTableRow {
                inputs: in_vec,
                outputs: out_vec,
            });
        }

        let input_labels: Vec<String> = input_ids.iter().map(|(_, l)| l.clone()).collect();
        let output_labels: Vec<String> = output_ids.iter().map(|(_, l)| l.clone()).collect();

        let mut canonical_sop = Vec::new();
        let mut minimized_sop = Vec::new();
        let mut all_pis = Vec::new();

        for (oidx, out_label) in output_labels.iter().enumerate() {
            let mterms = &minterms_per_output[oidx];
            let m_list = mterms
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            canonical_sop.push(format!("{out_label} = Sum_m({m_list})"));
            let (min_expr, pis) = quine_mccluskey_minimize(&input_labels, mterms);
            minimized_sop.push(format!("{out_label} = {min_expr}"));
            for pi in pis {
                if !all_pis.contains(&pi) {
                    all_pis.push(pi);
                }
            }
        }

        Ok(TruthTableResult {
            input_labels,
            output_labels,
            rows,
            canonical_sop,
            minimized_sop,
            prime_implicants: all_pis,
        })
    }

    pub fn simulate_timing(&self, ticks: usize) -> TimingTraceResult {
        let clamped_ticks = ticks.clamp(4, 64);
        let mut sim = self.clone();
        let mut channels: Vec<WaveformChannel> = sim
            .gates
            .iter()
            .map(|g| WaveformChannel {
                gate_id: g.id.clone(),
                label: g.label.clone(),
                kind: g.kind,
                samples: Vec::with_capacity(clamped_ticks),
            })
            .collect();

        let input_ids: Vec<String> = sim
            .gates
            .iter()
            .filter(|g| g.kind == GateKind::Input)
            .map(|g| g.id.clone())
            .collect();

        for t in 0..clamped_ticks {
            // Cycle input stimuli automatically if no clock/DFF exists, or toggle inputs periodically
            for (idx, iid) in input_ids.iter().enumerate() {
                let period = 1usize << (idx + 1);
                if t > 0 && t % period == 0 {
                    if let Some(g) = sim.gates.iter_mut().find(|g| g.id == *iid) {
                        g.state = !g.state;
                    }
                }
            }
            sim.step_clock();
            for (ch, g) in channels.iter_mut().zip(sim.gates.iter()) {
                ch.samples.push(g.state);
            }
        }

        let total_transistors: u32 = self.gates.iter().map(|g| g.kind.cmos_transistors()).sum();
        let critical_path_ps: u32 = self
            .gates
            .iter()
            .map(|g| g.kind.delay_ps())
            .sum::<u32>()
            .max(50);
        let max_clock_mhz = ((1_000_000.0 / f64::from(critical_path_ps)) * 10.0).round() / 10.0;

        TimingTraceResult {
            ticks: clamped_ticks,
            critical_path_ps,
            max_clock_mhz,
            total_transistors,
            channels,
        }
    }

    pub fn export_hdl(&self, format: &str) -> Result<String, LogicError> {
        match format.trim().to_ascii_lowercase().as_str() {
            "verilog" | "v" => Ok(self.to_verilog()),
            "vhdl" | "vhd" => Ok(self.to_vhdl()),
            "dot" | "graphviz" => Ok(self.to_dot()),
            "json" | "netlist" => serde_json::to_string_pretty(&json!({
                "preset": self.active_preset,
                "gates": self.gates,
                "wires": self.wires,
            }))
            .map_err(|e| LogicError::Serialization(e.to_string())),
            other => Err(LogicError::Validation(format!(
                "Unsupported HDL format '{other}'. Expected verilog, vhdl, dot, or json."
            ))),
        }
    }

    fn to_verilog(&self) -> String {
        let inputs: Vec<String> = self
            .gates
            .iter()
            .filter(|g| matches!(g.kind, GateKind::Input | GateKind::Clock))
            .map(|g| g.id.clone())
            .collect();
        let outputs: Vec<String> = self
            .gates
            .iter()
            .filter(|g| matches!(g.kind, GateKind::Output | GateKind::Seg7))
            .map(|g| g.id.clone())
            .collect();
        let mut ports = inputs.clone();
        ports.extend(outputs.clone());

        let mut out = String::new();
        out.push_str(&format!(
            "// Generated by LogicCraft v{APP_VERSION} (preset: {})\n",
            self.active_preset
        ));
        out.push_str(&format!(
            "module logiccraft_circuit (\n    {}\n);\n",
            ports.join(", ")
        ));
        for i in &inputs {
            out.push_str(&format!("    input wire {i};\n"));
        }
        for o in &outputs {
            out.push_str(&format!("    output wire {o};\n"));
        }
        for g in &self.gates {
            if !matches!(
                g.kind,
                GateKind::Input | GateKind::Clock | GateKind::Output | GateKind::Seg7
            ) {
                out.push_str(&format!("    wire {};\n", g.id));
            }
        }
        out.push('\n');
        for g in &self.gates {
            let drivers: Vec<String> = self
                .wires
                .iter()
                .filter(|w| w.to_gate == g.id)
                .map(|w| w.from_gate.clone())
                .collect();
            let rhs = match g.kind {
                GateKind::Input | GateKind::Clock => continue,
                GateKind::Output | GateKind::Seg7 | GateKind::Dff => drivers
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "1'b0".to_string()),
                GateKind::Not => format!(
                    "~{}",
                    drivers
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "1'b0".to_string())
                ),
                GateKind::And => drivers.join(" & "),
                GateKind::Or => drivers.join(" | "),
                GateKind::Nand => format!("~({})", drivers.join(" & ")),
                GateKind::Nor => format!("~({})", drivers.join(" | ")),
                GateKind::Xor => drivers.join(" ^ "),
                GateKind::Xnor => format!("~({})", drivers.join(" ^ ")),
            };
            let expr = if rhs.is_empty() {
                "1'b0".to_string()
            } else {
                rhs
            };
            out.push_str(&format!("    assign {} = {};\n", g.id, expr));
        }
        out.push_str("endmodule\n");
        out
    }

    fn to_vhdl(&self) -> String {
        let mut out = format!(
            "-- Generated by LogicCraft v{APP_VERSION}\nlibrary IEEE;\nuse IEEE.STD_LOGIC_1164.ALL;\n\nentity logiccraft_circuit is\nend logiccraft_circuit;\n\narchitecture Behavioral of logiccraft_circuit is\nbegin\n"
        );
        for w in &self.wires {
            out.push_str(&format!("    {} <= {};\n", w.to_gate, w.from_gate));
        }
        out.push_str("end Behavioral;\n");
        out
    }

    fn to_dot(&self) -> String {
        let mut out = String::from("digraph LogicCraft {\n    rankdir=LR;\n");
        for g in &self.gates {
            out.push_str(&format!(
                "    \"{}\" [label=\"{} ({}) = {}\"];\n",
                g.id,
                g.label,
                g.kind.as_str(),
                i32::from(g.state)
            ));
        }
        for w in &self.wires {
            out.push_str(&format!(
                "    \"{}\" -> \"{}\" [label=\"{}\"];\n",
                w.from_gate,
                w.to_gate,
                i32::from(w.signal)
            ));
        }
        out.push_str("}\n");
        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "params", rename_all = "snake_case")]
pub enum Command {
    Status,
    Summary,
    LoadPreset {
        preset: String,
    },
    AddGate {
        id: String,
        label: String,
        kind: String,
        x: f32,
        y: f32,
    },
    RemoveGate {
        gate_id: String,
    },
    ToggleInput {
        gate_id: String,
    },
    ConnectWire {
        from_gate: String,
        to_gate: String,
    },
    EvaluateCircuit,
    StepClock,
    GenerateTruthTable,
    MinimizeExpression {
        variables: Vec<String>,
        minterms: Vec<usize>,
    },
    SimulateTiming {
        ticks: usize,
    },
    AluExecute {
        op: String,
        a: u8,
        b: u8,
    },
    ExportHdl {
        format: String,
    },
}

impl Command {
    pub fn dispatch(&self, engine: &mut LogicEngine) -> Result<Value, LogicError> {
        match self {
            Self::Status => Ok(json!({
                "app": "logiccraft",
                "version": APP_VERSION,
                "status": "ready",
                "active_preset": engine.active_preset,
                "gate_count": engine.gates.len(),
                "wire_count": engine.wires.len(),
                "tick_count": engine.tick_count,
            })),
            Self::Summary => Ok(json!({
                "active_preset": engine.active_preset,
                "gates": engine.gates,
                "wires": engine.wires,
                "tick_count": engine.tick_count,
                "registers": engine.registers,
            })),
            Self::LoadPreset { preset } => {
                let p = CircuitPreset::parse(preset).ok_or_else(|| {
                    LogicError::Validation(format!(
                        "Unknown preset '{preset}'. Expected half_adder, full_adder, sr_latch, mux_2to1, parity_4bit, or counter_2bit."
                    ))
                })?;
                engine.load_preset(p);
                Ok(json!({
                    "loaded": p.as_str(),
                    "gate_count": engine.gates.len(),
                    "wire_count": engine.wires.len(),
                }))
            }
            Self::AddGate {
                id,
                label,
                kind,
                x,
                y,
            } => {
                let gk = GateKind::parse(kind)
                    .ok_or_else(|| LogicError::Validation(format!("Unknown gate kind '{kind}'")))?;
                let node = engine.add_gate(id, label, gk, *x, *y)?;
                Ok(json!({ "added": node }))
            }
            Self::RemoveGate { gate_id } => {
                engine.remove_gate(gate_id)?;
                Ok(json!({ "removed": gate_id }))
            }
            Self::ToggleInput { gate_id } => {
                let state = engine.toggle_input(gate_id)?;
                Ok(json!({ "gate_id": gate_id, "state": state }))
            }
            Self::ConnectWire { from_gate, to_gate } => {
                let wire = engine.connect_wire(from_gate, to_gate)?;
                Ok(json!({ "wire": wire }))
            }
            Self::EvaluateCircuit => {
                let passes = engine.evaluate_circuit();
                Ok(json!({
                    "passes": passes,
                    "gates": engine.gates,
                    "wires": engine.wires,
                }))
            }
            Self::StepClock => {
                engine.step_clock();
                Ok(json!({
                    "tick_count": engine.tick_count,
                    "gates": engine.gates,
                }))
            }
            Self::GenerateTruthTable => {
                let tt = engine.generate_truth_table()?;
                serde_json::to_value(tt).map_err(|e| LogicError::Serialization(e.to_string()))
            }
            Self::MinimizeExpression {
                variables,
                minterms,
            } => {
                let (expr, pis) = quine_mccluskey_minimize(variables, minterms);
                Ok(json!({
                    "variables": variables,
                    "minterms": minterms,
                    "minimized_sop": expr,
                    "prime_implicants": pis,
                }))
            }
            Self::SimulateTiming { ticks } => {
                let trace = engine.simulate_timing(*ticks);
                serde_json::to_value(trace).map_err(|e| LogicError::Serialization(e.to_string()))
            }
            Self::AluExecute { op, a, b } => {
                let alu_op = AluOp::parse(op)
                    .ok_or_else(|| LogicError::Validation(format!("Unknown ALU op '{op}'")))?;
                let res = execute_alu8(alu_op, *a, *b);
                engine.registers.acc = res.result;
                engine.registers.zero_flag = res.zero_flag;
                engine.registers.carry_flag = res.carry_flag;
                engine.registers.negative_flag = res.negative_flag;
                engine.registers.overflow_flag = res.overflow_flag;
                engine.registers.last_op = format!(
                    "{} 0x{:02X}, 0x{:02X} -> 0x{:02X}",
                    res.op, a, b, res.result
                );
                serde_json::to_value(res).map_err(|e| LogicError::Serialization(e.to_string()))
            }
            Self::ExportHdl { format } => {
                let code = engine.export_hdl(format)?;
                Ok(json!({ "format": format, "code": code }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_adder_evaluates_all_eight_input_combinations() {
        let eng = LogicEngine::with_preset(CircuitPreset::FullAdder);
        let res = eng.generate_truth_table();
        assert!(res.is_ok());
        if let Ok(tt) = res {
            assert_eq!(tt.rows.len(), 8);
            // Row 7: A=1, B=1, CIN=1 => SUM=1, COUT=1
            assert_eq!(tt.rows[7].outputs, vec![true, true]);
            // Row 3: A=0, B=1, CIN=1 => SUM=0, COUT=1
            assert_eq!(tt.rows[3].outputs, vec![false, true]);
        }
    }

    #[test]
    fn quine_mccluskey_minimizes_carry_out_to_three_prime_implicants() {
        let vars = vec!["A".to_string(), "B".to_string(), "CIN".to_string()];
        let (expr, pis) = quine_mccluskey_minimize(&vars, &[3, 5, 6, 7]);
        assert_eq!(pis.len(), 3);
        assert!(expr.contains("A & B"));
        assert!(expr.contains("B & CIN"));
    }

    #[test]
    fn alu8_computes_signed_overflow_and_carry_accurately() {
        let add_ovf = execute_alu8(AluOp::Add, 127, 1);
        assert_eq!(add_ovf.result, 128);
        assert_eq!(add_ovf.signed_result, -128);
        assert!(add_ovf.overflow_flag);
        assert!(add_ovf.negative_flag);
        assert!(!add_ovf.carry_flag);

        let add_carry = execute_alu8(AluOp::Add, 255, 1);
        assert_eq!(add_carry.result, 0);
        assert!(add_carry.zero_flag);
        assert!(add_carry.carry_flag);
    }

    #[test]
    fn timing_analyzer_and_verilog_exporter_produce_valid_traces() {
        let eng = LogicEngine::with_preset(CircuitPreset::Counter2Bit);
        let trace = eng.simulate_timing(12);
        assert_eq!(trace.ticks, 12);
        assert!(trace.max_clock_mhz > 100.0);

        let verilog = eng.export_hdl("verilog").unwrap_or_default();
        assert!(verilog.contains("module logiccraft_circuit"));
        assert!(verilog.contains("endmodule"));
    }

    #[test]
    fn sqlite_roundtrip_persists_gates_wires_and_registers() {
        let mut path = std::env::temp_dir();
        path.push(format!("logiccraft_test_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);

        let mut eng = LogicEngine::with_preset(CircuitPreset::Mux2To1);
        let _ = Command::AluExecute {
            op: "xor".into(),
            a: 0xAA,
            b: 0x55,
        }
        .dispatch(&mut eng);
        assert!(eng.save_db(&path).is_ok());

        let loaded = load_from_db(&path).ok().flatten();
        let _ = std::fs::remove_file(&path);
        assert!(loaded.is_some());
        if let Some(l) = loaded {
            assert_eq!(l.active_preset, "mux_2to1");
            assert_eq!(l.registers.acc, 0xFF);
        }
    }
}
