# LogicCraft

**Pure-Rust Digital Logic Breadboard, Quine-McCluskey / K-Map Synthesizer, Multi-Channel Logic Oscilloscope, 8-Bit ALU/CPU Lab & Verilog-2001 Forge** (`L1 logiccraft-core` + `L2 logiccraft-mcp` + `L3 logiccraft-app`).

## Overview

LogicCraft replaces heavyweight Java/Electron circuit simulators with a zero-dependency native Rust desktop studio and MCP `2026-07-28` server. It bundles 6 hierarchical workbenches in one unified workspace:

1. **Breadboard Studio** -- interactive gate & wire canvas (`INPUT`, `OUTPUT`, `NOT`, `AND`, `OR`, `NAND`, `NOR`, `XOR`, `XNOR`, `DFF`, `CLOCK`, `SEG7`) with live glowing signal wires and 6 built-in presets (`half_adder`, `full_adder`, `sr_latch`, `mux_2to1`, `parity_4bit`, `counter_2bit`).
2. **Truth Table & K-Map Lab** -- exhaustive truth-table sweep up to 8 inputs (256 rows) with automatic **Quine-McCluskey prime-implicant minimization**.
3. **Oscilloscope & Timing Analyzer** -- multi-tick cycle-accurate waveform analyzer, 45nm CMOS propagation delay (`ps`), transistor count estimator, and maximum clock frequency (`MHz`).
4. **8-Bit ALU & CPU Lab** -- hardware-accurate 8-bit ALU (`ADD`, `SUB`, `AND`, `OR`, `XOR`, `NOT A`, `SHL`, `SHR`, `CMP`, `INC A`) with bit-toggle switches, status flags (`Zero`, `Carry`, `Negative`, `Overflow`), and a 4-register CPU datapath (`R0..R3`, `ACC`, `PC`).
5. **HDL & Silicon Forge** -- structural **Verilog-2001**, **VHDL**, **Graphviz DOT**, and **JSON Netlist** exporter.
6. **Assistant & 6-Tab Help** -- local-first Chat and interactive documentation.

## Feature Matrix

| Capability | Status | Crate / Module |
|---|---|---|
| Interactive Breadboard Gate & Wire Simulator (`12` gate types, `6` presets) | `Implemented` | `logiccraft-core`, `logiccraft-app` |
| Exhaustive Truth Table Sweep + Quine-McCluskey Prime-Implicant Minimizer | `Implemented` | `logiccraft-core::model` |
| Multi-Channel Digital Oscilloscope & Critical-Path `MHz` Timing Budget | `Implemented` | `logiccraft-core`, `logiccraft-app` |
| 8-Bit ALU (`Z/C/N/V` flags) & 4-Register CPU Datapath Workbench | `Implemented` | `logiccraft-core::model` |
| Structural Verilog-2001, VHDL, Graphviz DOT & JSON Netlist Exporter | `Implemented` | `logiccraft-core` |
| Bundled SQLite Persistence (`%LOCALAPPDATA%\logiccraft\logiccraft.db`) | `Implemented` | `logiccraft-core::db` |
| CLI (`--json`) + MCP `2026-07-28` Stdio Server (`14` tools) | `Implemented` | `logiccraft-mcp` |
| Native `eframe`/`egui` 0.37 Studio (6 Workbenches, WCAG AAA Dark/Light) | `Implemented` | `logiccraft-app` |

## Quick Start

```powershell
# Launch desktop studio
.\start.ps1

# Run headless smoke test
.\start.ps1 -Smoke

# Run CLI commands
cargo run -p logiccraft-mcp -- status --json
cargo run -p logiccraft-mcp -- truth-table
cargo run -p logiccraft-mcp -- alu --op add --a 127 --b 1
cargo run -p logiccraft-mcp -- export-hdl --format verilog

# Run MCP 2026-07-28 stdio server
cargo run -p logiccraft-mcp -- --mcp
```

## License

Dual-licensed under `MIT OR Apache-2.0`.
