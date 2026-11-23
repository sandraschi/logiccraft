//! Comprehensive 6-Tab In-App Documentation for LogicCraft.

use crate::app::{HelpTab, LogicCraftApp};
use eframe::egui;
use logiccraft_core::APP_VERSION;

const HELP_MD: &str = include_str!("../../../HELP.md");

pub fn render_help_tab_content(app: &LogicCraftApp, ui: &mut egui::Ui) {
    match app.help_tab {
        HelpTab::GetStarted => {
            ui.label(HELP_MD);
        }
        HelpTab::BreadboardGates => {
            ui.heading("Workbench 1: Digital Logic Primitives & 45nm CMOS Characterization");
            ui.label(
                "Every gate on the LogicCraft breadboard models both exact Boolean logic and physical 45nm static CMOS transistor count and propagation delay.",
            );
            ui.add_space(6.0);

            ui.group(|ui| {
                ui.strong("1. Complete Gate Library & Physical Metrics");
                egui::Grid::new("help_logic_gates_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Gate Kind");
                        ui.strong("Boolean Expression");
                        ui.strong("Stage Delay (ps)");
                        ui.strong("Static CMOS FETs");
                        ui.strong("Notes");
                        ui.end_row();

                        for (kind, expr, delay, fets, notes) in [
                            (
                                "INPUT",
                                "Driven by user / test vector",
                                "10 ps",
                                "0 T",
                                "Click on canvas or toolbar to toggle 0 / 1",
                            ),
                            (
                                "OUTPUT",
                                "Y = A",
                                "10 ps",
                                "0 T",
                                "Primary probe / LED terminal",
                            ),
                            (
                                "NOT",
                                "Y = ~A",
                                "25 ps",
                                "2 T",
                                "1 PMOS + 1 NMOS inverter stage",
                            ),
                            (
                                "NAND",
                                "Y = ~(A & B & ...)",
                                "35 ps",
                                "4 T",
                                "Universal single-stage CMOS gate",
                            ),
                            (
                                "NOR",
                                "Y = ~(A | B | ...)",
                                "35 ps",
                                "4 T",
                                "Universal single-stage CMOS gate (SR Latch primitive)",
                            ),
                            (
                                "AND",
                                "Y = A & B & ...",
                                "55 ps",
                                "6 T",
                                "NAND followed by CMOS inverter",
                            ),
                            (
                                "OR",
                                "Y = A | B | ...",
                                "55 ps",
                                "6 T",
                                "NOR followed by CMOS inverter",
                            ),
                            (
                                "XOR",
                                "Y = A ^ B ^ ...",
                                "85 ps",
                                "12 T",
                                "Odd-parity / half-adder sum gate",
                            ),
                            (
                                "XNOR",
                                "Y = ~(A ^ B ^ ...)",
                                "85 ps",
                                "12 T",
                                "Equivalence / equality comparator gate",
                            ),
                            (
                                "DFF",
                                "Q(t+1) = D(t) on rising CLK",
                                "120 ps",
                                "20 T",
                                "Edge-triggered master-slave D flip-flop",
                            ),
                            (
                                "CLOCK",
                                "Toggles each Step Clock tick",
                                "10 ps",
                                "8 T",
                                "Synchronous system clock driver",
                            ),
                            (
                                "SEG7",
                                "7-Segment LED indicator",
                                "40 ps",
                                "28 T",
                                "Visual display sink",
                            ),
                        ] {
                            ui.monospace(kind);
                            ui.monospace(expr);
                            ui.monospace(delay);
                            ui.monospace(fets);
                            ui.label(notes);
                            ui.end_row();
                        }
                    });
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. Built-In Circuit Presets");
                ui.label("- half_adder: 2 inputs (A, B), XOR sum + AND carry-out (18 CMOS transistors).");
                ui.label("- full_adder: 3 inputs (A, B, CIN), 2 XOR + 2 AND + 1 OR carry chain (42 CMOS transistors).");
                ui.label("- sr_latch: Cross-coupled NOR bistable memory cell with Set (S) and Reset (R).");
                ui.label("- mux_2to1: 2-input data multiplexer (D0, D1) controlled by select line SEL.");
                ui.label("- parity_4bit: 3-XOR binary tree computing odd parity across 4 input bits (B0..B3).");
                ui.label("- counter_2bit: Synchronous sequential 2-bit modulo-4 binary counter using DFFs.");
            });
        }
        HelpTab::TruthTablesKMaps => {
            ui.heading("Workbench 2: Exhaustive Truth Tables & Quine-McCluskey K-Map Minimization");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("1. Exhaustive 2^N Truth-Table Sweep (Subpage 1)");
                ui.label("LogicCraft clones the active circuit in memory and sweeps all 2^N binary input vectors (up to 8 inputs = 256 rows), relaxing combinational logic to steady state for every row.");
                ui.label("For each output terminal, it extracts:");
                ui.monospace(
                    "Canonical Sum-of-Products : Sum_m(m_0, m_1, ...) listing all active minterm indices\nMinimized SOP Equation    : Exact minimal two-level AND-OR expression",
                );
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. Quine-McCluskey Prime-Implicant Algorithm (Subpage 2)");
                ui.label("Unlike visual 4-variable Karnaugh maps that stop working above 4 inputs, LogicCraft implements the tabular Quine-McCluskey method:");
                ui.label("1. Groups active minterms by Hamming weight (number of 1-bits) and iteratively merges pairs differing in exactly 1 bit position into dash-masked implicants (bits, mask).");
                ui.label("2. Collects all unmergeable Prime Implicants (PIs).");
                ui.label("3. Identifies Essential Prime Implicants (any PI that is the sole cover for at least one minterm) and completes minimal coverage across remaining minterms.");
                ui.add_space(4.0);
                ui.strong("Example -- Full Adder Carry-Out (Variables: A, B, CIN | Minterms: 3, 5, 6, 7):");
                ui.monospace(
                    "Minterm 3 (011) + Minterm 7 (111) -> -11 (B & CIN)\nMinterm 5 (101) + Minterm 7 (111) -> 1-1 (A & CIN)\nMinterm 6 (110) + Minterm 7 (111) -> 11- (A & B)\nMinimal SOP: COUT = (B & CIN) | (A & CIN) | (A & B)",
                );
            });
        }
        HelpTab::AluTiming => {
            ui.heading(
                "Workbenches 3 & 4: Digital Oscilloscope Timing Budget & 8-Bit ALU / CPU Lab",
            );
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("1. Digital Oscilloscope & Maximum Clock Frequency (Workbench 3)");
                ui.label("Simulates 8..32 discrete clock ticks, driving periodic stimulus patterns across input pins and latching DFF registers on each tick.");
                ui.monospace(
                    "Critical Path Budget (ps) = Sum of active stage delays along longest path\nMax Clock Frequency (MHz) = 1,000,000 / Critical_Path_ps",
                );
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. 8-Bit Two's-Complement ALU & Status Flags (Workbench 4)");
                ui.label("Interactive 8-bit datapath supporting 10 hardware opcodes (ADD, SUB, AND, OR, XOR, NOT A, SHL, SHR, CMP, INC A) with individual bit toggles (b7..b0) and four status flags:");
                egui::Grid::new("help_alu_flags_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Flag");
                        ui.strong("Name");
                        ui.strong("Hardware Condition");
                        ui.end_row();

                        ui.monospace("Z");
                        ui.label("Zero Flag");
                        ui.label("Set to 1 when the 8-bit result equals 0x00");
                        ui.end_row();

                        ui.monospace("C");
                        ui.label("Carry / Borrow Flag");
                        ui.label("Unsigned carry out of bit 7 (ADD/SHL) or unsigned borrow (SUB/CMP)");
                        ui.end_row();

                        ui.monospace("N");
                        ui.label("Negative (Sign) Flag");
                        ui.label("Set to 1 when bit 7 (MSB) of the result is 1 (signed value < 0)");
                        ui.end_row();

                        ui.monospace("V");
                        ui.label("Signed Overflow Flag");
                        ui.label("Set when two's-complement signed arithmetic overflows [-128, +127] (e.g. 127 + 1 = -128)");
                        ui.end_row();
                    });
            });
        }
        HelpTab::CliMcp => {
            ui.heading("L2 CLI Subcommands & 14 MCP 2026-07-28 Tools Reference");
            ui.label("Every GUI button, CLI command, and MCP tool call routes through L1 'Command::dispatch'.");
            ui.add_space(6.0);

            ui.group(|ui| {
                ui.strong("1. Complete 14 MCP Tools Table");
                egui::Grid::new("help_logic_mcp_table")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("MCP Tool Name");
                        ui.strong("CLI Subcommand");
                        ui.strong("Description");
                        ui.end_row();

                        for (tool, cli, desc) in [
                            ("logic_status", "status", "Engine version, active preset, gate/wire counts, and clock tick"),
                            ("logic_summary", "summary", "Full breadboard state (gates, wires, and 8-bit CPU register file)"),
                            ("load_preset", "load-preset", "Load half_adder, full_adder, sr_latch, mux_2to1, parity_4bit, or counter_2bit"),
                            ("add_gate", "add-gate", "Add a gate node (id, label, kind, x, y) to the breadboard"),
                            ("remove_gate", "remove-gate", "Delete a gate and its attached wires"),
                            ("toggle_input", "toggle-input", "Flip an input switch 0 <-> 1 and propagate signals"),
                            ("connect_wire", "connect-wire", "Add a directed wire between from_gate and to_gate"),
                            ("evaluate_circuit", "evaluate", "Run combinational signal relaxation across the breadboard"),
                            ("step_clock", "step-clock", "Advance sequential clock and DFF registers by 1 tick"),
                            ("generate_truth_table", "truth-table", "Sweep all 2^N input vectors and synthesize minimal SOP equations"),
                            ("minimize_expression", "minimize", "Run Quine-McCluskey minimization on custom variables and minterms"),
                            ("simulate_timing", "timing", "Run multi-tick oscilloscope trace and compute max clock MHz"),
                            ("alu_execute", "alu", "Execute 8-bit ALU operation with Z/C/N/V flags and latch into ACC"),
                            ("export_hdl", "export-hdl", "Synthesize Verilog-2001, VHDL, Graphviz DOT, or JSON netlist"),
                        ] {
                            ui.monospace(tool);
                            ui.monospace(cli);
                            ui.label(desc);
                            ui.end_row();
                        }
                    });
            });
        }
        HelpTab::TechnicalInfo => {
            ui.heading("Technical Architecture, SQLite Schema & HDL Synthesis");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.strong("1. 3-Tier Pure-Rust ArtCraft Architecture");
                ui.label(format!("Application Version: v{APP_VERSION}"));
                ui.label(format!("Active SQLite Database Path: {}", app.db_path.display()));
                ui.monospace(
                    "L1: crates/logiccraft-core  (Pure-Rust solver, Quine-McCluskey, ALU8, rusqlite 0.40 bundled)\nL2: crates/logiccraft-mcp   (clap 4.5 CLI + MCP 2026-07-28 JSON-RPC 2.0 stdio server)\nL3: crates/logiccraft-app   (eframe/egui 0.37 native desktop studio + egui_kittest harness)",
                );
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("2. SQLite Persistence Schema (gates, wires, cpu_meta)");
                ui.monospace(
                    "CREATE TABLE gates (id TEXT PRIMARY KEY, label TEXT, kind TEXT, state INTEGER, x REAL, y REAL);\nCREATE TABLE wires (id TEXT PRIMARY KEY, from_gate TEXT, to_gate TEXT, signal INTEGER);\nCREATE TABLE cpu_meta (id INTEGER PRIMARY KEY CHECK (id = 1), preset_name TEXT, tick_count INTEGER, regs_json TEXT);",
                );
            });

            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("3. Safety & Quality Containment Invariants");
                ui.label("- #![forbid(unsafe_code)] enforced across all workspace crates.");
                ui.label("- clippy::deny(unwrap_used, expect_used, panic, todo, unimplemented) enforced at workspace level.");
                ui.label("- Headless egui_kittest accessibility and rendering verification across all 12 workbench subpages and all 6 help tabs.");
            });
        }
    }
}
