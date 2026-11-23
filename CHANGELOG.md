# Changelog

## [0.2.0] - 2026-11-23

### Added
- Built 3-tier Pure-Rust ArtCraft workspace (`logiccraft-core`, `logiccraft-mcp`, `logiccraft-app`).
- Added 6 hierarchical workbenches (`Breadboard Studio`, `Truth Table & K-Map`, `Oscilloscope & Timing`, `8-Bit ALU & CPU Lab`, `HDL & Silicon Forge`, `Assistant & Help`).
- Added Quine-McCluskey prime-implicant Boolean minimizer, cycle-accurate waveform analyzer, 8-bit ALU with `Z/C/N/V` status flags, structural Verilog-2001/VHDL/DOT synthesis, and bundled SQLite persistence (`rusqlite` 0.40).
- Added 14 MCP `2026-07-28` tools and 14 CLI subcommands sharing `Command::dispatch`.
