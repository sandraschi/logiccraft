//! LogicCraft L3 Native Desktop Studio (`eframe` / `egui` 0.37).

use crate::subapps;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use logiccraft_core::{
    default_db_path, execute_alu8, quine_mccluskey_minimize, AluOp, CircuitPreset, Command,
    GateKind, LogicEngine, APP_VERSION,
};
use std::path::PathBuf;

const HELP_MD: &str = include_str!("../../../HELP.md");
const ATTRIBUTION_MD: &str = include_str!("../../../ATTRIBUTION.md");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLocale {
    En,
    De,
}

impl AppLocale {
    pub fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::De => "Deutsch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Breadboard,
    TruthTable,
    Timing,
    AluLab,
    HdlExport,
    Chat,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    GetStarted,
    BreadboardGates,
    TruthTablesKMaps,
    AluTiming,
    CliMcp,
    TechnicalInfo,
}

pub struct LogicCraftApp {
    pub engine: LogicEngine,
    pub db_path: PathBuf,
    pub page: Page,
    pub subpage: usize,
    pub locale: AppLocale,
    pub dark_mode: bool,
    pub zoom_factor: f32,
    pub status_line: String,
    // Breadboard builder state
    pub new_gate_id: String,
    pub new_gate_label: String,
    pub new_gate_kind: String,
    pub wire_from: String,
    pub wire_to: String,
    // K-Map custom minimizer state
    pub custom_vars: String,
    pub custom_minterms: String,
    pub custom_min_result: String,
    // Timing state
    pub timing_ticks: usize,
    // 8-Bit ALU state
    pub alu_op: AluOp,
    pub alu_a: u8,
    pub alu_b: u8,
    // HDL state
    pub hdl_format: String,
    // Assistant & modals
    pub chat_input: String,
    pub chat_history: Vec<(String, String)>,
    pub help_tab: HelpTab,
    pub show_palette: bool,
    pub palette_query: String,
    pub show_shortcuts: bool,
    pub show_licenses: bool,
    pub show_about: bool,
}

impl Default for LogicCraftApp {
    fn default() -> Self {
        Self::with_db_path(default_db_path())
    }
}

impl LogicCraftApp {
    pub fn with_db_path(db_path: PathBuf) -> Self {
        let engine = LogicEngine::open_or_seed(&db_path).unwrap_or_default();
        Self {
            engine,
            db_path,
            page: Page::Breadboard,
            subpage: 0,
            locale: AppLocale::En,
            dark_mode: true,
            zoom_factor: 1.0,
            status_line: format!("LogicCraft v{APP_VERSION} ready -- preset: full_adder"),
            new_gate_id: "g_custom".into(),
            new_gate_label: "CUSTOM".into(),
            new_gate_kind: "nand".into(),
            wire_from: "in_a".into(),
            wire_to: "out_sum".into(),
            custom_vars: "A,B,CIN".into(),
            custom_minterms: "3,5,6,7".into(),
            custom_min_result: "(A & B) | (A & CIN) | (B & CIN)".into(),
            timing_ticks: 16,
            alu_op: AluOp::Add,
            alu_a: 0x2A,
            alu_b: 0x15,
            hdl_format: "verilog".into(),
            chat_input: String::new(),
            chat_history: vec![(
                "LogicCraft".into(),
                "no model selected -- configure a local Ollama or MCP model to co-design logic circuits.".into(),
            )],
            help_tab: HelpTab::GetStarted,
            show_palette: false,
            palette_query: String::new(),
            show_shortcuts: false,
            show_licenses: false,
            show_about: false,
        }
    }

    pub fn goto(&mut self, page: Page, sub: usize) {
        self.page = page;
        self.subpage = sub;
    }

    fn persist(&mut self) {
        let _ = self.engine.save_db(&self.db_path);
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let theme = if self.dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        ctx.set_theme(theme);
        let mut style = (*ctx.style_of(theme)).clone();
        if self.dark_mode {
            style.visuals.override_text_color = Some(Color32::from_rgb(241, 245, 249));
            style.visuals.panel_fill = Color32::from_rgb(15, 23, 42);
        } else {
            style.visuals.override_text_color = Some(Color32::from_rgb(15, 23, 42));
            style.visuals.panel_fill = Color32::from_rgb(248, 250, 252);
        }
        ctx.set_style_of(theme, style);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let zoom_delta = ctx.input(|i| {
            if i.modifiers.ctrl {
                i.smooth_scroll_delta.y
            } else {
                0.0
            }
        });
        if zoom_delta.abs() > 0.1 {
            let step = if zoom_delta > 0.0 { 0.1 } else { -0.1 };
            self.zoom_factor = (self.zoom_factor + step).clamp(0.75, 1.75);
            ctx.set_zoom_factor(self.zoom_factor);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.goto(Page::Help, 0);
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::K)) {
            self.show_palette = !self.show_palette;
        }
    }

    pub fn render_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save Circuit to SQLite").clicked() {
                    self.persist();
                    self.status_line = format!("Saved to {}", self.db_path.display());
                    ui.close();
                }
                if ui.button("Reset Full Adder Seed").clicked() {
                    self.engine.load_preset(CircuitPreset::FullAdder);
                    self.persist();
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Step Sequential Clock").clicked() {
                    self.engine.step_clock();
                    self.persist();
                    ui.close();
                }
                if ui.button("Evaluate Combinational Signals").clicked() {
                    let _ = self.engine.evaluate_circuit();
                    self.persist();
                    ui.close();
                }
            });
            ui.menu_button("View", |ui| {
                if ui.button("Dark").clicked() {
                    self.dark_mode = true;
                    ui.close();
                }
                if ui.button("Light").clicked() {
                    self.dark_mode = false;
                    ui.close();
                }
                ui.separator();
                if ui.button("English").clicked() {
                    self.locale = AppLocale::En;
                    ui.close();
                }
                if ui.button("Deutsch").clicked() {
                    self.locale = AppLocale::De;
                    ui.close();
                }
            });
            ui.menu_button("Insert", |ui| {
                for (label, preset) in [
                    ("Preset: Half Adder", CircuitPreset::HalfAdder),
                    ("Preset: Full Adder", CircuitPreset::FullAdder),
                    ("Preset: SR NOR Latch", CircuitPreset::SrLatch),
                    ("Preset: 2-to-1 Multiplexer", CircuitPreset::Mux2To1),
                    (
                        "Preset: 4-Bit Parity Checker",
                        CircuitPreset::ParityChecker4Bit,
                    ),
                    (
                        "Preset: 2-Bit Synchronous Counter",
                        CircuitPreset::Counter2Bit,
                    ),
                ] {
                    if ui.button(label).clicked() {
                        self.engine.load_preset(preset);
                        self.persist();
                        ui.close();
                    }
                }
            });
            ui.menu_button("Modify", |ui| {
                if ui.button("Toggle All Input Switches").clicked() {
                    let ids: Vec<String> = self
                        .engine
                        .gates
                        .iter()
                        .filter(|g| g.kind == GateKind::Input)
                        .map(|g| g.id.clone())
                        .collect();
                    for id in ids {
                        let _ = self.engine.toggle_input(&id);
                    }
                    self.persist();
                    ui.close();
                }
            });
            ui.menu_button("Commands", |ui| {
                if ui.button("Generate Truth Table").clicked() {
                    self.goto(Page::TruthTable, 0);
                    ui.close();
                }
                if ui.button("Run Quine-McCluskey Minimizer").clicked() {
                    self.goto(Page::TruthTable, 1);
                    ui.close();
                }
                if ui.button("Export Verilog-2001").clicked() {
                    self.hdl_format = "verilog".into();
                    self.goto(Page::HdlExport, 0);
                    ui.close();
                }
            });
            ui.menu_button("Window", |ui| {
                for (label, p, s) in [
                    ("Breadboard Studio", Page::Breadboard, 0),
                    ("Truth Table & K-Map", Page::TruthTable, 0),
                    ("Digital Oscilloscope", Page::Timing, 0),
                    ("8-Bit ALU & CPU Lab", Page::AluLab, 0),
                    ("HDL & Silicon Forge", Page::HdlExport, 0),
                    ("Chat", Page::Chat, 0),
                    ("Help (F1)", Page::Help, 0),
                ] {
                    if ui.button(label).clicked() {
                        self.goto(p, s);
                        ui.close();
                    }
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Command Palette (Ctrl+K)").clicked() {
                    self.show_palette = true;
                    ui.close();
                }
                if ui.button("Keyboard Shortcuts").clicked() {
                    self.show_shortcuts = true;
                    ui.close();
                }
                if ui.button("Documentation (F1)").clicked() {
                    self.goto(Page::Help, 0);
                    ui.close();
                }
                if ui.button("Licenses").clicked() {
                    self.show_licenses = true;
                    ui.close();
                }
                if ui.button("About LogicCraft").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let theme_lbl = if self.dark_mode { "Light" } else { "Dark" };
                if ui.button(theme_lbl).clicked() {
                    self.dark_mode = !self.dark_mode;
                }
                let loc_lbl = self.locale.label();
                if ui.button(loc_lbl).clicked() {
                    self.locale = match self.locale {
                        AppLocale::En => AppLocale::De,
                        AppLocale::De => AppLocale::En,
                    };
                }
                ui.label(format!(
                    "Preset: {} | Gates: {} | Wires: {} | Tick: {}",
                    self.engine.active_preset,
                    self.engine.gates.len(),
                    self.engine.wires.len(),
                    self.engine.tick_count
                ));
            });
        });
        let _ = ctx;
    }

    pub fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("LogicCraft");
        ui.small("Digital Logic & ALU Studio");
        ui.separator();

        for wb in subapps::registry() {
            let subs = wb.subpages();
            let has_active = subs
                .iter()
                .any(|sp| sp.page == self.page && sp.sub == self.subpage);
            egui::CollapsingHeader::new(RichText::new(wb.title(self.locale)).strong())
                .id_salt(wb.id())
                .default_open(has_active || wb.id() == "breadboard")
                .show(ui, |ui| {
                    for sp in subs {
                        let active = self.page == sp.page && self.subpage == sp.sub;
                        if ui.selectable_label(active, sp.label(self.locale)).clicked() {
                            self.goto(sp.page, sp.sub);
                        }
                    }
                });
        }
    }

    pub fn render_page(&mut self, ui: &mut egui::Ui) {
        match self.page {
            Page::Breadboard => self.render_breadboard(ui),
            Page::TruthTable => self.render_truth_table(ui),
            Page::Timing => self.render_timing(ui),
            Page::AluLab => self.render_alu_lab(ui),
            Page::HdlExport => self.render_hdl_export(ui),
            Page::Chat => self.render_chat(ui),
            Page::Help => self.render_help_view(ui),
        }
    }

    fn render_breadboard(&mut self, ui: &mut egui::Ui) {
        let green = Color32::from_rgb(34, 197, 94);
        let slate = Color32::from_rgb(100, 116, 139);
        let cyan = Color32::from_rgb(56, 189, 248);

        ui.horizontal_wrapped(|ui| {
            ui.heading("1. Breadboard & Digital Gate Studio");
            if ui.button("Step Clock (+1 Tick)").clicked() {
                self.engine.step_clock();
                self.persist();
            }
            for (lbl, p) in [
                ("Half Adder", CircuitPreset::HalfAdder),
                ("Full Adder", CircuitPreset::FullAdder),
                ("SR Latch", CircuitPreset::SrLatch),
                ("2:1 MUX", CircuitPreset::Mux2To1),
                ("4-Bit Parity", CircuitPreset::ParityChecker4Bit),
                ("2-Bit Counter", CircuitPreset::Counter2Bit),
            ] {
                if ui
                    .selectable_label(self.engine.active_preset == p.as_str(), lbl)
                    .clicked()
                {
                    self.engine.load_preset(p);
                    self.persist();
                }
            }
        });
        ui.separator();

        if self.subpage == 1 {
            ui.group(|ui| {
                ui.strong("Add Logic Gate to Breadboard");
                ui.horizontal_wrapped(|ui| {
                    ui.label("ID:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_gate_id)
                            .hint_text("Gate ID")
                            .desired_width(90.0),
                    );
                    ui.label("Label:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_gate_label)
                            .hint_text("Gate Label")
                            .desired_width(90.0),
                    );
                    ui.label("Kind:");
                    egui::ComboBox::from_label("Gate Kind")
                        .selected_text(&self.new_gate_kind)
                        .show_ui(ui, |ui| {
                            for k in [
                                "input", "output", "not", "and", "or", "nand", "nor", "xor",
                                "xnor", "dff", "clock", "seg7",
                            ] {
                                ui.selectable_value(&mut self.new_gate_kind, k.to_string(), k);
                            }
                        });
                    if ui.button("Add Gate").clicked() {
                        if let Some(gk) = GateKind::parse(&self.new_gate_kind) {
                            if self
                                .engine
                                .add_gate(&self.new_gate_id, &self.new_gate_label, gk, 360.0, 180.0)
                                .is_ok()
                            {
                                self.persist();
                            }
                        }
                    }
                });
            });
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("Connect Directed Wire");
                ui.horizontal_wrapped(|ui| {
                    ui.label("From Gate ID:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.wire_from)
                            .hint_text("Source Gate ID")
                            .desired_width(100.0),
                    );
                    ui.label("To Gate ID:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.wire_to)
                            .hint_text("Target Gate ID")
                            .desired_width(100.0),
                    );
                    if ui.button("Connect Wire").clicked()
                        && self
                            .engine
                            .connect_wire(&self.wire_from, &self.wire_to)
                            .is_ok()
                    {
                        self.persist();
                    }
                });
            });
            ui.add_space(6.0);
        }

        // Input Toggle Bar
        ui.horizontal_wrapped(|ui| {
            ui.strong("Inputs (Click to Toggle 0/1):");
            let inputs: Vec<(String, String, bool)> = self
                .engine
                .gates
                .iter()
                .filter(|g| g.kind == GateKind::Input)
                .map(|g| (g.id.clone(), g.label.clone(), g.state))
                .collect();
            for (id, label, state) in inputs {
                let text = format!("{label} = {}", i32::from(state));
                let btn = egui::Button::new(RichText::new(text).strong().color(Color32::WHITE))
                    .fill(if state { green } else { slate });
                if ui.add(btn).clicked() {
                    let _ = self.engine.toggle_input(&id);
                    self.persist();
                }
            }
            ui.separator();
            ui.strong("Outputs:");
            for g in self
                .engine
                .gates
                .iter()
                .filter(|g| matches!(g.kind, GateKind::Output | GateKind::Seg7))
            {
                let col = if g.state { green } else { slate };
                ui.colored_label(col, format!("[LED] {} = {}", g.label, i32::from(g.state)));
            }
        });

        ui.add_space(6.0);
        let canvas_size = Vec2::new(ui.available_width().max(640.0), 360.0);
        let (rect, response) = ui.allocate_exact_size(canvas_size, Sense::click());
        let painter = ui.painter_at(rect);
        let bg = if self.dark_mode {
            Color32::from_rgb(9, 14, 28)
        } else {
            Color32::from_rgb(241, 245, 249)
        };
        painter.rect_filled(rect, 8.0, bg);

        let pos_of = |x: f32, y: f32| -> Pos2 { Pos2::new(rect.left() + x, rect.top() + y) };

        // Draw wires
        for w in &self.engine.wires {
            if let (Some(fg), Some(tg)) = (
                self.engine.gates.iter().find(|g| g.id == w.from_gate),
                self.engine.gates.iter().find(|g| g.id == w.to_gate),
            ) {
                let p1 = pos_of(fg.x + 42.0, fg.y);
                let p2 = pos_of(tg.x - 42.0, tg.y);
                let mid_x = (p1.x + p2.x) * 0.5;
                let col = if w.signal { green } else { slate };
                let width = if w.signal { 3.0 } else { 1.8 };
                painter.line_segment([p1, Pos2::new(mid_x, p1.y)], Stroke::new(width, col));
                painter.line_segment(
                    [Pos2::new(mid_x, p1.y), Pos2::new(mid_x, p2.y)],
                    Stroke::new(width, col),
                );
                painter.line_segment([Pos2::new(mid_x, p2.y), p2], Stroke::new(width, col));
            }
        }

        // Draw gate nodes & handle click-to-toggle on Input nodes
        let mut clicked_gate: Option<String> = None;
        for g in &self.engine.gates {
            let center = pos_of(g.x, g.y);
            let node_rect = Rect::from_center_size(center, Vec2::new(84.0, 44.0));
            let fill = if g.state {
                Color32::from_rgb(20, 83, 45)
            } else if self.dark_mode {
                Color32::from_rgb(30, 41, 59)
            } else {
                Color32::from_rgb(226, 232, 240)
            };
            let border = if g.state { green } else { cyan };
            painter.rect_filled(node_rect, 6.0, fill);
            painter.rect_stroke(
                node_rect,
                6.0,
                Stroke::new(2.0, border),
                egui::StrokeKind::Outside,
            );
            let txt_col = if g.state || self.dark_mode {
                Color32::WHITE
            } else {
                Color32::from_rgb(15, 23, 42)
            };
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                format!(
                    "{}\n[{}] = {}",
                    g.label,
                    g.kind.as_str().to_uppercase(),
                    i32::from(g.state)
                ),
                egui::FontId::monospace(11.0),
                txt_col,
            );

            if response.clicked() {
                if let Some(ptr) = response.interact_pointer_pos() {
                    if node_rect.contains(ptr) && g.kind == GateKind::Input {
                        clicked_gate = Some(g.id.clone());
                    }
                }
            }
        }
        if let Some(cid) = clicked_gate {
            let _ = self.engine.toggle_input(&cid);
            self.persist();
        }
    }

    fn render_truth_table(&mut self, ui: &mut egui::Ui) {
        ui.heading("2. Truth Table & Quine-McCluskey K-Map Synthesizer");
        ui.separator();

        if self.subpage == 1 {
            ui.group(|ui| {
                ui.strong("Custom Quine-McCluskey Prime-Implicant Minimizer");
                ui.horizontal_wrapped(|ui| {
                    ui.label("Variables (comma-separated):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.custom_vars)
                            .hint_text("Variables e.g. A,B,CIN")
                            .desired_width(140.0),
                    );
                    ui.label("Active Minterms (e.g. 1,2,4,7):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.custom_minterms)
                            .hint_text("Minterms e.g. 3,5,6,7")
                            .desired_width(160.0),
                    );
                    if ui.button("Synthesize Minimal SOP").clicked() {
                        let vars: Vec<String> = self
                            .custom_vars
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        let mterms: Vec<usize> = self
                            .custom_minterms
                            .split(',')
                            .filter_map(|s| s.trim().parse::<usize>().ok())
                            .collect();
                        let (expr, pis) = quine_mccluskey_minimize(&vars, &mterms);
                        self.custom_min_result =
                            format!("F = {expr}   |   Prime Implicants: {}", pis.join(", "));
                    }
                });
                ui.monospace(&self.custom_min_result);
            });
            ui.add_space(8.0);
        }

        match self.engine.generate_truth_table() {
            Ok(tt) => {
                ui.group(|ui| {
                    ui.strong("Synthesized Boolean Equations (Active Breadboard Circuit)");
                    for eq in &tt.minimized_sop {
                        ui.monospace(format!("Minimized SOP: {eq}"));
                    }
                    for can in &tt.canonical_sop {
                        ui.small(format!("Canonical: {can}"));
                    }
                });
                ui.add_space(6.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("truth_table_grid")
                        .striped(true)
                        .min_col_width(52.0)
                        .show(ui, |ui| {
                            ui.strong("Row #");
                            for il in &tt.input_labels {
                                ui.strong(il);
                            }
                            for ol in &tt.output_labels {
                                ui.strong(format!("OUT: {ol}"));
                            }
                            ui.end_row();

                            for (idx, row) in tt.rows.iter().enumerate() {
                                ui.monospace(idx.to_string());
                                for &b in &row.inputs {
                                    ui.monospace(if b { "1" } else { "0" });
                                }
                                for &b in &row.outputs {
                                    let col = if b {
                                        Color32::from_rgb(34, 197, 94)
                                    } else {
                                        ui.visuals().text_color()
                                    };
                                    ui.colored_label(col, if b { "1" } else { "0" });
                                }
                                ui.end_row();
                            }
                        });
                });
            }
            Err(e) => {
                ui.label(format!("Truth table note: {e}"));
            }
        }
    }

    fn render_timing(&mut self, ui: &mut egui::Ui) {
        ui.heading("3. Multi-Channel Digital Oscilloscope & Timing Budget");
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.timing_ticks, 8..=32).text("Simulation Ticks"));
        });
        ui.separator();

        let trace = self.engine.simulate_timing(self.timing_ticks);
        ui.horizontal_wrapped(|ui| {
            ui.group(|ui| {
                ui.strong("Critical Path Delay");
                ui.monospace(format!("{} ps (45nm CMOS)", trace.critical_path_ps));
            });
            ui.group(|ui| {
                ui.strong("Max Clock Frequency");
                ui.monospace(format!("{:.1} MHz", trace.max_clock_mhz));
            });
            ui.group(|ui| {
                ui.strong("Total CMOS Transistors");
                ui.monospace(format!("{} FETs", trace.total_transistors));
            });
        });

        ui.add_space(8.0);
        if self.subpage == 1 {
            egui::Grid::new("timing_bom_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Gate ID");
                    ui.strong("Label");
                    ui.strong("Kind");
                    ui.strong("Stage Delay (ps)");
                    ui.strong("CMOS Transistors");
                    ui.end_row();
                    for g in &self.engine.gates {
                        ui.monospace(&g.id);
                        ui.label(&g.label);
                        ui.monospace(g.kind.as_str());
                        ui.monospace(format!("{} ps", g.kind.delay_ps()));
                        ui.monospace(format!("{}", g.kind.cmos_transistors()));
                        ui.end_row();
                    }
                });
            return;
        }

        // Draw multi-channel logic analyzer waveforms
        let row_h = 34.0;
        let height = (trace.channels.len() as f32 * row_h + 20.0).max(160.0);
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width().max(600.0), height),
            Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        let bg = if self.dark_mode {
            Color32::from_rgb(9, 14, 28)
        } else {
            Color32::from_rgb(241, 245, 249)
        };
        painter.rect_filled(rect, 6.0, bg);

        let label_w = 110.0;
        let trace_w = (rect.width() - label_w - 20.0).max(200.0);
        let step_w = trace_w / (trace.ticks.max(1) as f32);
        let green = Color32::from_rgb(34, 197, 94);

        for (cidx, ch) in trace.channels.iter().enumerate() {
            let y_top = rect.top() + 10.0 + (cidx as f32) * row_h;
            let y_high = y_top + 6.0;
            let y_low = y_top + 24.0;
            painter.text(
                Pos2::new(rect.left() + 8.0, y_top + 14.0),
                egui::Align2::LEFT_CENTER,
                &ch.label,
                egui::FontId::monospace(11.5),
                ui.visuals().text_color(),
            );

            for (tidx, &sample) in ch.samples.iter().enumerate() {
                let x0 = rect.left() + label_w + (tidx as f32) * step_w;
                let x1 = x0 + step_w;
                let y = if sample { y_high } else { y_low };
                painter.line_segment(
                    [Pos2::new(x0, y), Pos2::new(x1, y)],
                    Stroke::new(2.0, green),
                );
                if tidx > 0 && ch.samples[tidx - 1] != sample {
                    painter.line_segment(
                        [Pos2::new(x0, y_high), Pos2::new(x0, y_low)],
                        Stroke::new(1.5, green),
                    );
                }
            }
        }
    }

    fn render_alu_lab(&mut self, ui: &mut egui::Ui) {
        ui.heading("4. 8-Bit ALU & CPU Register Datapath Lab");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("ALU Operation:");
            for op in [
                AluOp::Add,
                AluOp::Sub,
                AluOp::And,
                AluOp::Or,
                AluOp::Xor,
                AluOp::NotA,
                AluOp::Shl,
                AluOp::Shr,
                AluOp::Cmp,
                AluOp::IncA,
            ] {
                if ui
                    .selectable_label(self.alu_op == op, op.mnemonic())
                    .clicked()
                {
                    self.alu_op = op;
                }
            }
        });

        ui.add_space(6.0);
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.strong(format!(
                    "Operand A = {} (0x{:02X} | {:08b} | signed {})",
                    self.alu_a, self.alu_a, self.alu_a, self.alu_a as i8
                ));
                ui.add(egui::Slider::new(&mut self.alu_a, 0..=255).text("Operand A"));
                ui.horizontal(|ui| {
                    for bit in (0..8).rev() {
                        let is_set = ((self.alu_a >> bit) & 1) != 0;
                        if ui
                            .selectable_label(is_set, format!("b{bit}:{}", i32::from(is_set)))
                            .clicked()
                        {
                            self.alu_a ^= 1 << bit;
                        }
                    }
                });
            });

            cols[1].group(|ui| {
                ui.strong(format!(
                    "Operand B = {} (0x{:02X} | {:08b} | signed {})",
                    self.alu_b, self.alu_b, self.alu_b, self.alu_b as i8
                ));
                ui.add(egui::Slider::new(&mut self.alu_b, 0..=255).text("Operand B"));
                ui.horizontal(|ui| {
                    for bit in (0..8).rev() {
                        let is_set = ((self.alu_b >> bit) & 1) != 0;
                        if ui
                            .selectable_label(is_set, format!("b{bit}:{}", i32::from(is_set)))
                            .clicked()
                        {
                            self.alu_b ^= 1 << bit;
                        }
                    }
                });
            });
        });

        let res = execute_alu8(self.alu_op, self.alu_a, self.alu_b);
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.strong(format!(
                "ALU Output ({}) -> {} | {} | Binary {} | Two's Complement Signed {}",
                res.op, res.result, res.hex_result, res.binary_result, res.signed_result
            ));
            ui.horizontal_wrapped(|ui| {
                for (flag_name, active) in [
                    ("Z (Zero)", res.zero_flag),
                    ("C (Carry/Borrow)", res.carry_flag),
                    ("N (Negative)", res.negative_flag),
                    ("V (Signed Overflow)", res.overflow_flag),
                ] {
                    let col = if active {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_rgb(100, 116, 139)
                    };
                    ui.colored_label(col, format!("[{flag_name} = {}]", i32::from(active)));
                }
                if ui.button("Latch Result into CPU ACC").clicked() {
                    let _ = Command::AluExecute {
                        op: self.alu_op.mnemonic().to_string(),
                        a: self.alu_a,
                        b: self.alu_b,
                    }
                    .dispatch(&mut self.engine);
                    self.engine.registers.pc = self.engine.registers.pc.wrapping_add(1);
                    self.persist();
                }
            });
        });

        if self.subpage == 1 {
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.strong("CPU 4-Register File (R0..R3, ACC, PC)");
                let regs = &self.engine.registers;
                ui.monospace(format!(
                    "R0: 0x{:02X} ({:3}) | R1: 0x{:02X} ({:3}) | R2: 0x{:02X} ({:3}) | R3: 0x{:02X} ({:3})",
                    regs.r0, regs.r0, regs.r1, regs.r1, regs.r2, regs.r2, regs.r3, regs.r3
                ));
                ui.monospace(format!(
                    "ACC: 0x{:02X} ({:08b}) | PC: {} | Last Micro-Op: {}",
                    regs.acc, regs.acc, regs.pc, regs.last_op
                ));
            });
        }
    }

    fn render_hdl_export(&mut self, ui: &mut egui::Ui) {
        ui.heading("5. HDL & Silicon Netlist Forge");
        ui.horizontal(|ui| {
            for (lbl, fmt) in [
                ("Verilog-2001", "verilog"),
                ("VHDL (IEEE 1164)", "vhdl"),
                ("Graphviz DOT", "dot"),
                ("JSON Netlist", "json"),
            ] {
                if ui.selectable_label(self.hdl_format == fmt, lbl).clicked() {
                    self.hdl_format = fmt.to_string();
                }
            }
        });
        ui.separator();

        let code = self
            .engine
            .export_hdl(&self.hdl_format)
            .unwrap_or_else(|e| e.to_string());
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut code.as_str())
                    .hint_text("Synthesized HDL Output")
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(20),
            );
        });
    }

    fn render_chat(&mut self, ui: &mut egui::Ui) {
        ui.heading("LogicCraft Co-Designer Chat");
        ui.label("no model selected -- connect an MCP client or local Ollama endpoint to synthesize circuits from natural language.");
        ui.separator();
        for (who, msg) in &self.chat_history {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!("{who}:"));
                ui.label(msg);
            });
        }
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.chat_input)
                    .hint_text("Ask about truth tables, K-maps, ALU flags, or Verilog...")
                    .desired_width(420.0),
            );
            if ui.button("Send").clicked() && !self.chat_input.trim().is_empty() {
                let q = self.chat_input.trim().to_string();
                self.chat_input.clear();
                self.chat_history.push(("You".into(), q));
                self.chat_history.push((
                    "LogicCraft".into(),
                    "no model selected -- use the 14 MCP tools (`generate_truth_table`, `minimize_expression`, `alu_execute`, `export_hdl`) from your agent IDE.".into(),
                ));
            }
        });
    }

    fn render_help_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("LogicCraft Documentation & Reference (F1)");
        ui.horizontal_wrapped(|ui| {
            for (tab, label) in [
                (HelpTab::GetStarted, "Get Started"),
                (HelpTab::BreadboardGates, "Breadboard & Gates"),
                (HelpTab::TruthTablesKMaps, "Truth Tables & K-Maps"),
                (HelpTab::AluTiming, "ALU & Timing"),
                (HelpTab::CliMcp, "CLI & MCP"),
                (HelpTab::TechnicalInfo, "Technical Info"),
            ] {
                if ui.selectable_label(self.help_tab == tab, label).clicked() {
                    self.help_tab = tab;
                }
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| match self.help_tab {
            HelpTab::GetStarted => {
                ui.label(HELP_MD);
            }
            HelpTab::BreadboardGates => {
                ui.strong("Supported Digital Logic Primitives (45nm CMOS Characterization)");
                ui.label("INPUT, OUTPUT, NOT (2T, 25ps), NAND/NOR (4T, 35ps), AND/OR (6T, 55ps), XOR/XNOR (12T, 85ps), DFF (20T, 120ps), CLOCK, SEG7.");
            }
            HelpTab::TruthTablesKMaps => {
                ui.strong("Quine-McCluskey Prime-Implicant Minimization");
                ui.label("Sweeps all 2^N input vectors (up to 8 variables = 256 minterms), combines adjacent Hamming-distance-1 implicants, and extracts an essential prime-implicant cover.");
            }
            HelpTab::AluTiming => {
                ui.strong("8-Bit Two's-Complement ALU & Status Flags");
                ui.label("Computes ADD, SUB, AND, OR, XOR, NOT A, SHL, SHR, CMP, and INC A with exact hardware Zero (Z), Carry/Borrow (C), Negative (N), and Signed Overflow (V) flags.");
            }
            HelpTab::CliMcp => {
                ui.strong("14 MCP 2024-11-05 Tools & CLI Subcommands");
                ui.monospace("logic_status, logic_summary, load_preset, add_gate, remove_gate, toggle_input, connect_wire, evaluate_circuit, step_clock, generate_truth_table, minimize_expression, simulate_timing, alu_execute, export_hdl");
            }
            HelpTab::TechnicalInfo => {
                ui.strong("3-Tier Pure-Rust ArtCraft Architecture");
                ui.label(format!("SQLite Database Path: {}", self.db_path.display()));
                ui.label("L1: logiccraft-core | L2: logiccraft-mcp | L3: logiccraft-app (egui 0.37)");
            }
        });
    }

    fn render_modals(&mut self, ctx: &egui::Context) {
        if self.show_palette {
            let mut open = self.show_palette;
            let mut nav_target: Option<(Page, usize)> = None;
            egui::Window::new("Command Palette")
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.palette_query)
                            .hint_text("Filter commands..."),
                    );
                    for (label, page, sub) in [
                        ("Breadboard: Gate Canvas", Page::Breadboard, 0),
                        ("Breadboard: Gate & Wire Builder", Page::Breadboard, 1),
                        ("Truth Table Sweep", Page::TruthTable, 0),
                        ("Quine-McCluskey K-Map Minimizer", Page::TruthTable, 1),
                        ("Digital Oscilloscope", Page::Timing, 0),
                        ("Critical Path & CMOS Transistor BOM", Page::Timing, 1),
                        ("8-Bit ALU & Status Flags", Page::AluLab, 0),
                        ("CPU Register File", Page::AluLab, 1),
                        ("Export Verilog-2001 / VHDL", Page::HdlExport, 0),
                        ("Help Documentation (F1)", Page::Help, 0),
                    ] {
                        if (self.palette_query.is_empty()
                            || label
                                .to_ascii_lowercase()
                                .contains(&self.palette_query.to_ascii_lowercase()))
                            && ui.button(label).clicked()
                        {
                            nav_target = Some((page, sub));
                        }
                    }
                });
            self.show_palette = open;
            if let Some((page, sub)) = nav_target {
                self.goto(page, sub);
                self.show_palette = false;
            }
        }
        if self.show_shortcuts {
            egui::Window::new("Keyboard Shortcuts")
                .open(&mut self.show_shortcuts)
                .show(ctx, |ui| {
                    ui.label("F1: Open 6-Tab Help");
                    ui.label("Ctrl+K: Toggle Command Palette");
                    ui.label("Ctrl+MouseWheel: Zoom UI (0.75x .. 1.75x)");
                });
        }
        if self.show_licenses {
            egui::Window::new("Licenses & Attribution")
                .open(&mut self.show_licenses)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(ATTRIBUTION_MD);
                    });
                });
        }
        if self.show_about {
            egui::Window::new("About LogicCraft")
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.heading(format!("LogicCraft v{APP_VERSION}"));
                    ui.label("Pure-Rust Digital Logic Simulator, K-Map Synthesizer, Oscilloscope, 8-Bit ALU Lab & Verilog Forge.");
                });
        }
    }
}

impl eframe::App for LogicCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);
        self.handle_shortcuts(&ctx);

        egui::Panel::top("logiccraft_top_menu").show(ui, |ui| {
            self.render_menu_bar(&ctx, ui);
        });
        egui::Panel::bottom("logiccraft_status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_line);
            });
        });
        egui::Panel::left("logiccraft_sidebar")
            .default_size(225.0)
            .show(ui, |ui| {
                self.render_sidebar(ui);
            });
        egui::CentralPanel::default().show(ui, |ui| {
            self.render_page(ui);
        });

        self.render_modals(&ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    fn temp_app() -> LogicCraftApp {
        let mut p = std::env::temp_dir();
        p.push(format!("logiccraft_app_test_{}.db", std::process::id()));
        LogicCraftApp::with_db_path(p)
    }

    #[test]
    fn subapp_registry_exposes_all_six_workbenches() {
        let reg = subapps::registry();
        assert_eq!(reg.len(), 6);
        let total_subpages: usize = reg.iter().map(|w| w.subpages().len()).sum();
        assert_eq!(total_subpages, 12);
    }

    #[test]
    fn headless_egui_kittest_renders_all_twelve_workbench_subpages() {
        let mut app = temp_app();
        for wb in subapps::registry() {
            for sp in wb.subpages() {
                app.goto(sp.page, sp.sub);
                let mut harness = Harness::new_ui(|ui| {
                    let ctx = ui.ctx().clone();
                    app.render_menu_bar(&ctx, ui);
                    app.render_sidebar(ui);
                    app.render_page(ui);
                });
                harness.run();
            }
        }
        let _ = std::fs::remove_file(&app.db_path);
    }

    #[test]
    fn headless_egui_kittest_renders_all_six_help_tabs_and_both_themes() {
        let mut app = temp_app();
        app.goto(Page::Help, 0);
        for tab in [
            HelpTab::GetStarted,
            HelpTab::BreadboardGates,
            HelpTab::TruthTablesKMaps,
            HelpTab::AluTiming,
            HelpTab::CliMcp,
            HelpTab::TechnicalInfo,
        ] {
            app.help_tab = tab;
            app.dark_mode = !app.dark_mode;
            let mut harness = Harness::new_ui(|ui| {
                let ctx = ui.ctx().clone();
                app.apply_theme(&ctx);
                app.render_page(ui);
            });
            harness.run();
        }
        let _ = std::fs::remove_file(&app.db_path);
    }
}
