//! LogicCraft L2 CLI & MCP `2026-07-28` stdio server.
#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use logiccraft_core::{default_db_path, Command, LogicEngine, APP_VERSION};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "logiccraft-mcp",
    version,
    about = "LogicCraft Digital Logic Simulator, K-Map Synthesizer, Timing Analyzer, 8-Bit ALU & HDL Exporter"
)]
pub struct Cli {
    /// Emit structured JSON output
    #[arg(long, global = true, default_value_t = false)]
    pub json: bool,

    /// Optional SQLite database file path override
    #[arg(long, global = true)]
    pub db: Option<PathBuf>,

    /// Launch MCP 2026-07-28 JSON-RPC server on stdio
    #[arg(long, global = true, default_value_t = false)]
    pub mcp: bool,

    #[command(subcommand)]
    pub subcommand: Option<CliSub>,
}

#[derive(Debug, Subcommand)]
pub enum CliSub {
    /// Run MCP 2026-07-28 JSON-RPC server on stdio
    Mcp,
    /// Show LogicCraft engine status
    Status,
    /// Show full circuit summary (gates, wires, registers)
    Summary,
    /// Load a built-in circuit preset (half_adder, full_adder, sr_latch, mux_2to1, parity_4bit, counter_2bit)
    LoadPreset {
        #[arg(long, default_value = "full_adder")]
        preset: String,
    },
    /// Add a gate to the active breadboard
    AddGate {
        #[arg(long)]
        id: String,
        #[arg(long, default_value = "")]
        label: String,
        #[arg(long)]
        kind: String,
        #[arg(long, default_value_t = 200.0)]
        x: f32,
        #[arg(long, default_value_t = 150.0)]
        y: f32,
    },
    /// Remove a gate and its attached wires
    RemoveGate {
        #[arg(long)]
        gate_id: String,
    },
    /// Toggle an input switch by gate id or label
    ToggleInput {
        #[arg(long)]
        gate_id: String,
    },
    /// Connect a directed wire from one gate to another
    ConnectWire {
        #[arg(long)]
        from_gate: String,
        #[arg(long)]
        to_gate: String,
    },
    /// Propagate combinational logic signals
    Evaluate,
    /// Step sequential clock and DFF registers by 1 tick
    StepClock,
    /// Generate exhaustive truth table and Quine-McCluskey minimized SOP expressions
    TruthTable,
    /// Minimize a Boolean function from variable names and active minterm indices
    Minimize {
        #[arg(long, value_delimiter = ',')]
        variables: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        minterms: Vec<usize>,
    },
    /// Simulate multi-tick oscilloscope timing waveforms and critical-path max MHz
    Timing {
        #[arg(long, default_value_t = 16)]
        ticks: usize,
    },
    /// Execute an 8-bit ALU operation (add, sub, and, or, xor, not_a, shl, shr, cmp, inc_a)
    Alu {
        #[arg(long, default_value = "add")]
        op: String,
        #[arg(long, default_value_t = 42)]
        a: u8,
        #[arg(long, default_value_t = 21)]
        b: u8,
    },
    /// Synthesize and export HDL or netlist (verilog, vhdl, dot, json)
    ExportHdl {
        #[arg(long, default_value = "verilog")]
        format: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_path);
    let mut engine = LogicEngine::open_or_seed(&db_path).unwrap_or_default();

    if cli.mcp || matches!(cli.subcommand, Some(CliSub::Mcp)) {
        run_mcp_stdio(&mut engine, &db_path);
        return;
    }

    let cmd = match cli.subcommand.unwrap_or(CliSub::Status) {
        CliSub::Mcp | CliSub::Status => Command::Status,
        CliSub::Summary => Command::Summary,
        CliSub::LoadPreset { preset } => Command::LoadPreset { preset },
        CliSub::AddGate {
            id,
            label,
            kind,
            x,
            y,
        } => Command::AddGate {
            id,
            label,
            kind,
            x,
            y,
        },
        CliSub::RemoveGate { gate_id } => Command::RemoveGate { gate_id },
        CliSub::ToggleInput { gate_id } => Command::ToggleInput { gate_id },
        CliSub::ConnectWire { from_gate, to_gate } => Command::ConnectWire { from_gate, to_gate },
        CliSub::Evaluate => Command::EvaluateCircuit,
        CliSub::StepClock => Command::StepClock,
        CliSub::TruthTable => Command::GenerateTruthTable,
        CliSub::Minimize {
            variables,
            minterms,
        } => Command::MinimizeExpression {
            variables,
            minterms,
        },
        CliSub::Timing { ticks } => Command::SimulateTiming { ticks },
        CliSub::Alu { op, a, b } => Command::AluExecute { op, a, b },
        CliSub::ExportHdl { format } => Command::ExportHdl { format },
    };

    match cmd.dispatch(&mut engine) {
        Ok(val) => {
            let _ = engine.save_db(&db_path);
            let out = serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
            println!("{out}");
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

pub fn handle_mcp_request(req: &Value, engine: &mut LogicEngine, db_path: &Path) -> Option<Value> {
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let id = req.get("id").cloned();

    if method.starts_with("notifications/") {
        return None;
    }
    let req_id = id?;

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2026-07-28",
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": {
                    "name": "logiccraft-mcp",
                    "version": APP_VERSION
                }
            }
        })),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": mcp_tools_schema()
            }
        })),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            let cmd_res = match name {
                "logic_status" => Command::Status.dispatch(engine),
                "logic_summary" => Command::Summary.dispatch(engine),
                "load_preset" => {
                    let preset = args
                        .get("preset")
                        .and_then(Value::as_str)
                        .unwrap_or("full_adder")
                        .to_string();
                    Command::LoadPreset { preset }.dispatch(engine)
                }
                "add_gate" => {
                    let id = args
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let label = args
                        .get("label")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let kind = args
                        .get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or("and")
                        .to_string();
                    let x = args.get("x").and_then(Value::as_f64).unwrap_or(200.0) as f32;
                    let y = args.get("y").and_then(Value::as_f64).unwrap_or(150.0) as f32;
                    Command::AddGate {
                        id,
                        label,
                        kind,
                        x,
                        y,
                    }
                    .dispatch(engine)
                }
                "remove_gate" => {
                    let gate_id = args
                        .get("gate_id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    Command::RemoveGate { gate_id }.dispatch(engine)
                }
                "toggle_input" => {
                    let gate_id = args
                        .get("gate_id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    Command::ToggleInput { gate_id }.dispatch(engine)
                }
                "connect_wire" => {
                    let from_gate = args
                        .get("from_gate")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let to_gate = args
                        .get("to_gate")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    Command::ConnectWire { from_gate, to_gate }.dispatch(engine)
                }
                "evaluate_circuit" => Command::EvaluateCircuit.dispatch(engine),
                "step_clock" => Command::StepClock.dispatch(engine),
                "generate_truth_table" => Command::GenerateTruthTable.dispatch(engine),
                "minimize_expression" => {
                    let variables: Vec<String> = args
                        .get("variables")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default();
                    let minterms: Vec<usize> = args
                        .get("minterms")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(Value::as_u64)
                                .map(|n| n as usize)
                                .collect()
                        })
                        .unwrap_or_default();
                    Command::MinimizeExpression {
                        variables,
                        minterms,
                    }
                    .dispatch(engine)
                }
                "simulate_timing" => {
                    let ticks = args.get("ticks").and_then(Value::as_u64).unwrap_or(16) as usize;
                    Command::SimulateTiming { ticks }.dispatch(engine)
                }
                "alu_execute" => {
                    let op = args
                        .get("op")
                        .and_then(Value::as_str)
                        .unwrap_or("add")
                        .to_string();
                    let a = args.get("a").and_then(Value::as_u64).unwrap_or(0) as u8;
                    let b = args.get("b").and_then(Value::as_u64).unwrap_or(0) as u8;
                    Command::AluExecute { op, a, b }.dispatch(engine)
                }
                "export_hdl" => {
                    let format = args
                        .get("format")
                        .and_then(Value::as_str)
                        .unwrap_or("verilog")
                        .to_string();
                    Command::ExportHdl { format }.dispatch(engine)
                }
                other => Err(logiccraft_core::LogicError::Validation(format!(
                    "Unknown tool '{other}'"
                ))),
            };

            match cmd_res {
                Ok(val) => {
                    let _ = engine.save_db(db_path);
                    let text = serde_json::to_string_pretty(&val).unwrap_or_default();
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": req_id,
                        "result": {
                            "content": [{ "type": "text", "text": text }],
                            "isError": false
                        }
                    }))
                }
                Err(e) => Some(json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "content": [{ "type": "text", "text": e.to_string() }],
                        "isError": true
                    }
                })),
            }
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "error": { "code": -32601, "message": format!("Method not found: {method}") }
        })),
    }
}

fn mcp_tools_schema() -> Value {
    json!([
        { "name": "logic_status", "description": "Get LogicCraft engine status, active preset, gate/wire counts, and clock tick.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "logic_summary", "description": "Return full breadboard circuit state (gates, wires, and 8-bit CPU register file).", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "load_preset", "description": "Load a circuit preset (half_adder, full_adder, sr_latch, mux_2to1, parity_4bit, counter_2bit).", "inputSchema": { "type": "object", "properties": { "preset": { "type": "string" } }, "required": ["preset"] } },
        { "name": "add_gate", "description": "Add a logic gate (input, output, not, and, or, nand, nor, xor, xnor, dff, clock, seg7).", "inputSchema": { "type": "object", "properties": { "id": { "type": "string" }, "label": { "type": "string" }, "kind": { "type": "string" }, "x": { "type": "number" }, "y": { "type": "number" } }, "required": ["id", "kind"] } },
        { "name": "remove_gate", "description": "Remove a gate and its attached wires.", "inputSchema": { "type": "object", "properties": { "gate_id": { "type": "string" } }, "required": ["gate_id"] } },
        { "name": "toggle_input", "description": "Toggle an input gate high/low and propagate signals.", "inputSchema": { "type": "object", "properties": { "gate_id": { "type": "string" } }, "required": ["gate_id"] } },
        { "name": "connect_wire", "description": "Connect a directed wire between two gates.", "inputSchema": { "type": "object", "properties": { "from_gate": { "type": "string" }, "to_gate": { "type": "string" } }, "required": ["from_gate", "to_gate"] } },
        { "name": "evaluate_circuit", "description": "Run combinational signal relaxation across the active circuit.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "step_clock", "description": "Advance sequential clock and DFF registers by 1 tick.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "generate_truth_table", "description": "Exhaustively sweep all circuit inputs and compute Quine-McCluskey minimized SOP expressions.", "inputSchema": { "type": "object", "properties": {} } },
        { "name": "minimize_expression", "description": "Minimize a Boolean function using Quine-McCluskey prime implicants.", "inputSchema": { "type": "object", "properties": { "variables": { "type": "array", "items": { "type": "string" } }, "minterms": { "type": "array", "items": { "type": "integer" } } }, "required": ["variables", "minterms"] } },
        { "name": "simulate_timing", "description": "Run multi-tick oscilloscope timing analysis and estimate critical-path max clock MHz.", "inputSchema": { "type": "object", "properties": { "ticks": { "type": "integer" } } } },
        { "name": "alu_execute", "description": "Execute an 8-bit ALU operation (add, sub, and, or, xor, not_a, shl, shr, cmp, inc_a) with Z/C/N/V flags.", "inputSchema": { "type": "object", "properties": { "op": { "type": "string" }, "a": { "type": "integer" }, "b": { "type": "integer" } }, "required": ["op", "a", "b"] } },
        { "name": "export_hdl", "description": "Synthesize structural Verilog-2001, VHDL, Graphviz DOT, or JSON netlist from the active circuit.", "inputSchema": { "type": "object", "properties": { "format": { "type": "string" } }, "required": ["format"] } }
    ])
}

fn run_mcp_stdio(engine: &mut LogicEngine, db_path: &Path) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let Ok(raw) = line else { break };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(req) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        if let Some(resp) = handle_mcp_request(&req, engine, db_path) {
            if let Ok(s) = serde_json::to_string(&resp) {
                let _ = writeln!(stdout, "{s}");
                let _ = stdout.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("logiccraft_mcp_test_{}.db", std::process::id()));
        p
    }

    #[test]
    fn mcp_initialize_and_tools_list_return_14_tools() {
        let mut eng = LogicEngine::default();
        let db = temp_db();
        let init = handle_mcp_request(
            &json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
            &mut eng,
            &db,
        );
        assert!(init.is_some());

        let list = handle_mcp_request(
            &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
            &mut eng,
            &db,
        )
        .unwrap_or_default();
        let tools = list["result"]["tools"]
            .as_array()
            .map(Vec::len)
            .unwrap_or(0);
        assert_eq!(tools, 14);
        let _ = std::fs::remove_file(&db);
    }

    #[test]
    fn mcp_tools_call_executes_truth_table_alu_and_verilog_export() {
        let mut eng = LogicEngine::default();
        let db = temp_db();

        let tt = handle_mcp_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": { "name": "generate_truth_table", "arguments": {} }
            }),
            &mut eng,
            &db,
        )
        .unwrap_or_default();
        assert_eq!(tt["result"]["isError"], false);

        let alu = handle_mcp_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": { "name": "alu_execute", "arguments": { "op": "add", "a": 100, "b": 55 } }
            }),
            &mut eng,
            &db,
        )
        .unwrap_or_default();
        assert_eq!(alu["result"]["isError"], false);
        assert_eq!(eng.registers.acc, 155);

        let hdl = handle_mcp_request(
            &json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": { "name": "export_hdl", "arguments": { "format": "verilog" } }
            }),
            &mut eng,
            &db,
        )
        .unwrap_or_default();
        assert_eq!(hdl["result"]["isError"], false);
        let _ = std::fs::remove_file(&db);
    }
}
