//! Integration & preset verification suite for LogicCraft L1 core.

use logiccraft_core::{
    execute_alu8, quine_mccluskey_minimize, AluOp, CircuitPreset, Command, GateKind, LogicEngine,
};

#[test]
fn preset_half_adder_computes_sum_and_carry() {
    let eng = LogicEngine::with_preset(CircuitPreset::HalfAdder);
    let tt = eng.generate_truth_table();
    assert!(tt.is_ok());
    if let Ok(t) = tt {
        assert_eq!(t.rows.len(), 4);
        assert_eq!(t.rows[0].outputs, vec![false, false]);
        assert_eq!(t.rows[1].outputs, vec![true, false]);
        assert_eq!(t.rows[2].outputs, vec![true, false]);
        assert_eq!(t.rows[3].outputs, vec![false, true]);
    }
}

#[test]
fn preset_mux_2to1_selects_d0_or_d1() {
    let eng = LogicEngine::with_preset(CircuitPreset::Mux2To1);
    let tt = eng.generate_truth_table();
    assert!(tt.is_ok());
    if let Ok(t) = tt {
        assert_eq!(t.rows.len(), 8);
        // Inputs are [D0, D1, SEL]
        // Row 2: D0=0, D1=1, SEL=0 => Y=0
        assert_eq!(t.rows[2].outputs, vec![false]);
        // Row 3: D0=0, D1=1, SEL=1 => Y=1
        assert_eq!(t.rows[3].outputs, vec![true]);
        // Row 4: D0=1, D1=0, SEL=0 => Y=1
        assert_eq!(t.rows[4].outputs, vec![true]);
    }
}

#[test]
fn preset_parity_4bit_detects_odd_ones() {
    let eng = LogicEngine::with_preset(CircuitPreset::ParityChecker4Bit);
    let tt = eng.generate_truth_table();
    assert!(tt.is_ok());
    if let Ok(t) = tt {
        assert_eq!(t.rows.len(), 16);
        for (mask, row) in t.rows.iter().enumerate() {
            let expected_odd = mask.count_ones() % 2 == 1;
            assert_eq!(row.outputs, vec![expected_odd]);
        }
    }
}

#[test]
fn preset_counter_2bit_advances_on_step_clock() {
    let mut eng = LogicEngine::with_preset(CircuitPreset::Counter2Bit);
    assert_eq!(eng.tick_count, 0);
    eng.step_clock();
    assert_eq!(eng.tick_count, 1);
    let q0 = eng.gates.iter().find(|g| g.id == "out_q0").map(|g| g.state);
    assert_eq!(q0, Some(true));
}

#[test]
fn preset_sr_latch_sets_and_resets_q() {
    let mut eng = LogicEngine::with_preset(CircuitPreset::SrLatch);
    let q = eng.gates.iter().find(|g| g.id == "out_q").map(|g| g.state);
    assert_eq!(q, Some(true));
    // Toggle S off and R on
    let _ = eng.toggle_input("in_s");
    let _ = eng.toggle_input("in_r");
    let q_after = eng.gates.iter().find(|g| g.id == "out_q").map(|g| g.state);
    assert_eq!(q_after, Some(false));
}

#[test]
fn command_add_connect_toggle_and_remove_gate_workflow() {
    let mut eng = LogicEngine::with_preset(CircuitPreset::HalfAdder);
    let add = Command::AddGate {
        id: "g_inv".into(),
        label: "INV_SUM".into(),
        kind: "not".into(),
        x: 600.0,
        y: 120.0,
    }
    .dispatch(&mut eng);
    assert!(add.is_ok());

    let wire = Command::ConnectWire {
        from_gate: "g_xor".into(),
        to_gate: "g_inv".into(),
    }
    .dispatch(&mut eng);
    assert!(wire.is_ok());

    let rem = Command::RemoveGate {
        gate_id: "g_inv".into(),
    }
    .dispatch(&mut eng);
    assert!(rem.is_ok());
}

#[test]
fn hdl_exporter_supports_verilog_vhdl_dot_and_json() {
    let eng = LogicEngine::with_preset(CircuitPreset::FullAdder);
    for fmt in ["verilog", "vhdl", "dot", "json"] {
        let res = eng.export_hdl(fmt);
        assert!(res.is_ok());
        assert!(!res.unwrap_or_default().is_empty());
    }
    assert!(eng.export_hdl("invalid_format").is_err());
}

#[test]
fn alu_executes_all_ten_operations_deterministically() {
    for (op, a, b, expected) in [
        (AluOp::Add, 10, 20, 30),
        (AluOp::Sub, 30, 12, 18),
        (AluOp::And, 0b1100, 0b1010, 0b1000),
        (AluOp::Or, 0b1100, 0b1010, 0b1110),
        (AluOp::Xor, 0b1100, 0b1010, 0b0110),
        (AluOp::NotA, 0x0F, 0, 0xF0),
        (AluOp::Shl, 0b0001_0001, 0, 0b0010_0010),
        (AluOp::Shr, 0b0010_0010, 0, 0b0001_0001),
        (AluOp::Cmp, 42, 42, 0),
        (AluOp::IncA, 99, 0, 100),
    ] {
        let r = execute_alu8(op, a, b);
        assert_eq!(r.result, expected);
    }
}

#[test]
fn quine_mccluskey_handles_tautology_and_contradiction() {
    let vars = vec!["X".to_string(), "Y".to_string()];
    let (zero_expr, _) = quine_mccluskey_minimize(&vars, &[]);
    assert_eq!(zero_expr, "0");

    let (one_expr, _) = quine_mccluskey_minimize(&vars, &[0, 1, 2, 3]);
    assert_eq!(one_expr, "1");
}

#[test]
fn gate_kind_characterization_returns_positive_delays() {
    for gk in [
        GateKind::Input,
        GateKind::Output,
        GateKind::Not,
        GateKind::And,
        GateKind::Or,
        GateKind::Nand,
        GateKind::Nor,
        GateKind::Xor,
        GateKind::Xnor,
        GateKind::Dff,
        GateKind::Clock,
        GateKind::Seg7,
    ] {
        assert!(gk.delay_ps() >= 10);
        assert_eq!(GateKind::parse(gk.as_str()), Some(gk));
    }
}
