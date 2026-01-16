use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Highlight {
    pub id: usize,
    pub text: String,
    pub color: &'static str,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum LineEnding {
    #[default]
    None,
    NL,
    CR,
    NLCR,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub show_settings: Signal<bool>,
    pub show_highlights: Signal<bool>,
    pub show_timestamps: Signal<bool>,
    pub autoscroll: Signal<bool>,
    pub line_ending: Signal<LineEnding>,
    pub highlights: Signal<Vec<Highlight>>,
    pub filter_query: Signal<String>,
    pub match_case: Signal<bool>,
    pub use_regex: Signal<bool>,
    pub invert_filter: Signal<bool>,
}

pub const HIGHLIGHT_COLORS: &[&str] = &[
    "red", "blue", "yellow", "green", "purple", "orange", "teal", "pink", "indigo", "lime", "cyan",
    "rose", "fuchsia", "amber", "emerald", "sky", "violet",
];
