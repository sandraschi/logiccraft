# LogicCraft Help & User Guide

## Get Started

**What is LogicCraft?**
LogicCraft is a 3-tier Pure-Rust ArtCraft Digital Logic Simulator, Boolean & Karnaugh-Map Synthesizer, Multi-Channel Logic Oscilloscope, 8-Bit ALU/CPU Datapath Lab, and Verilog-2001/VHDL Netlist Forge.

It provides 6 integrated sub-app workbenches in the left sidebar:
1. **Breadboard Studio** -- interactive gate & wire canvas (`INPUT`, `OUTPUT`, `NOT`, `AND`, `OR`, `NAND`, `NOR`, `XOR`, `XNOR`, `DFF`, `CLOCK`, `SEG7`) with live glowing high/low signal propagation.
2. **Truth Table & K-Map** -- exhaustive truth-table sweep up to 8 inputs (256 rows) with automatic **Quine-McCluskey prime-implicant minimization**.
3. **Oscilloscope & Timing** -- cycle-accurate multi-tick logic analyzer waveforms, CMOS transistor counts, critical-path delay in picoseconds (`ps`), and maximum clock frequency (`MHz`).
4. **8-Bit ALU & CPU Lab** -- interactive 8-bit Arithmetic Logic Unit (`ADD`, `SUB`, `AND`, `OR`, `XOR`, `NOT A`, `SHL`, `SHR`, `CMP`, `INC A`) with bit-toggle switches, hardware status flags (`Zero`, `Carry`, `Negative`, `Overflow`), and a 4-register CPU datapath (`R0..R3`, `ACC`, `PC`).
5. **HDL & Silicon Forge** -- instant structural **Verilog-2001**, **VHDL**, **Graphviz DOT**, and **JSON Netlist** synthesis.
6. **Assistant & Help** -- local-first Chat and 6-tab interactive documentation.

## Functions

- **Circuit Presets**: `half_adder`, `full_adder`, `sr_latch`, `mux_2to1`, `parity_4bit`, `counter_2bit`.
- **SQLite Persistence**: Auto-saves gates, wires, and CPU register state to `%LOCALAPPDATA%\logiccraft\logiccraft.db` (or `--db <PATH>`).

## Usage

### CLI
```powershell
logiccraft-mcp status --json
logiccraft-mcp load-preset --preset full_adder
logiccraft-mcp truth-table
logiccraft-mcp minimize --variables A,B,CIN --minterms 3,5,6,7
logiccraft-mcp timing --ticks 16
logiccraft-mcp alu --op add --a 127 --b 1
logiccraft-mcp export-hdl --format verilog
```

### MCP Server (`2024-11-05` stdio)
```powershell
logiccraft-mcp --mcp
```
Exposes 14 MCP tools: `logic_status`, `logic_summary`, `load_preset`, `add_gate`, `remove_gate`, `toggle_input`, `connect_wire`, `evaluate_circuit`, `step_clock`, `generate_truth_table`, `minimize_expression`, `simulate_timing`, `alu_execute`, and `export_hdl`.

## Tips

- Click any **INPUT** node on the Breadboard canvas (or in the sidebar table) to toggle its logic state (`0` / `1`) and watch downstream gates recompute immediately.
- Use `Step Clock` on sequential presets (`counter_2bit`, `sr_latch`) to advance flip-flops.
- Press `Ctrl+K` for the Command Palette, `F1` for Help, and `Ctrl+MouseWheel` to zoom the UI.

## Other Crafts

LogicCraft is part of the `sandraschi` Pure-Rust ArtCraft ecosystem alongside `graphcraft`, `mathcraft`, `robotcraft`, `smbcraft`, `gamecraft`, and `lenscraft`.
