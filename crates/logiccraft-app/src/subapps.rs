//! Hierarchical `SubApp` sidebar registry for LogicCraft's 6 workbenches.

use crate::app::{AppLocale, Page};

#[derive(Debug, Clone, Copy)]
pub struct SubPage {
    pub page: Page,
    pub sub: usize,
    pub en: &'static str,
    pub de: &'static str,
}

impl SubPage {
    pub fn label(&self, locale: AppLocale) -> &'static str {
        match locale {
            AppLocale::En => self.en,
            AppLocale::De => self.de,
        }
    }
}

pub trait SubApp: Sync {
    fn id(&self) -> &'static str;
    fn title(&self, locale: AppLocale) -> &'static str;
    fn subpages(&self) -> &'static [SubPage];
}

struct CraftWorkbench {
    id: &'static str,
    en: &'static str,
    de: &'static str,
    subpages: &'static [SubPage],
}

impl SubApp for CraftWorkbench {
    fn id(&self) -> &'static str {
        self.id
    }

    fn title(&self, locale: AppLocale) -> &'static str {
        match locale {
            AppLocale::En => self.en,
            AppLocale::De => self.de,
        }
    }

    fn subpages(&self) -> &'static [SubPage] {
        self.subpages
    }
}

const BREADBOARD_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::Breadboard,
        sub: 0,
        en: "Gate Canvas & Wires",
        de: "Gatter-Leinwand & Leitungen",
    },
    SubPage {
        page: Page::Breadboard,
        sub: 1,
        en: "Presets & Gate Builder",
        de: "Vorlagen & Gatter-Editor",
    },
];

const TRUTHTABLE_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::TruthTable,
        sub: 0,
        en: "Truth Table Sweep",
        de: "Wahrheitstabelle",
    },
    SubPage {
        page: Page::TruthTable,
        sub: 1,
        en: "K-Map & Quine-McCluskey",
        de: "KV-Diagramm & Minimierung",
    },
];

const TIMING_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::Timing,
        sub: 0,
        en: "Digital Oscilloscope",
        de: "Digital-Oszilloskop",
    },
    SubPage {
        page: Page::Timing,
        sub: 1,
        en: "Critical Path & MHz Budget",
        de: "Kritischer Pfad & Taktbudget",
    },
];

const ALU_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::AluLab,
        sub: 0,
        en: "8-Bit ALU & Status Flags",
        de: "8-Bit ALU & Status-Flags",
    },
    SubPage {
        page: Page::AluLab,
        sub: 1,
        en: "CPU Registers & Micro-Stepper",
        de: "CPU-Register & Mikroprogramm",
    },
];

const HDL_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::HdlExport,
        sub: 0,
        en: "Verilog-2001 & VHDL",
        de: "Verilog-2001 & VHDL",
    },
    SubPage {
        page: Page::HdlExport,
        sub: 1,
        en: "Graphviz DOT & CMOS BOM",
        de: "Graphviz DOT & CMOS-Stueckliste",
    },
];

const ASSISTANT_SUBS: &[SubPage] = &[
    SubPage {
        page: Page::Chat,
        sub: 0,
        en: "Chat",
        de: "Chat",
    },
    SubPage {
        page: Page::Help,
        sub: 0,
        en: "Help (F1)",
        de: "Hilfe (F1)",
    },
];

static WORKBENCHES: [CraftWorkbench; 6] = [
    CraftWorkbench {
        id: "breadboard",
        en: "1. Breadboard Studio",
        de: "1. Steckbrett-Studio",
        subpages: BREADBOARD_SUBS,
    },
    CraftWorkbench {
        id: "truthtable",
        en: "2. Truth Table & K-Map",
        de: "2. Wahrheitstabelle & KV",
        subpages: TRUTHTABLE_SUBS,
    },
    CraftWorkbench {
        id: "timing",
        en: "3. Oscilloscope & Timing",
        de: "3. Oszilloskop & Timing",
        subpages: TIMING_SUBS,
    },
    CraftWorkbench {
        id: "alulab",
        en: "4. 8-Bit ALU & CPU Lab",
        de: "4. 8-Bit ALU & CPU-Labor",
        subpages: ALU_SUBS,
    },
    CraftWorkbench {
        id: "hdlexport",
        en: "5. HDL & Silicon Forge",
        de: "5. HDL & Silizium-Schmiede",
        subpages: HDL_SUBS,
    },
    CraftWorkbench {
        id: "assistant",
        en: "6. Assistant & Help",
        de: "6. Assistent & Hilfe",
        subpages: ASSISTANT_SUBS,
    },
];

pub fn registry() -> Vec<&'static dyn SubApp> {
    WORKBENCHES.iter().map(|w| w as &dyn SubApp).collect()
}
