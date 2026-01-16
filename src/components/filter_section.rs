use crate::components::common::{IconButton, PanelHeader, ToggleSwitch};
use crate::state::{AppState, Highlight, HIGHLIGHT_COLORS};
use dioxus::prelude::*;

#[component]
pub fn FilterSection() -> Element {
    let state = use_context::<AppState>();
    let show_highlights = (state.show_highlights)();

    rsx! {
        div {
            class: "shrink-0 px-5 py-3 z-10 flex flex-col gap-3 filter-section",
            div { class: "flex gap-2 w-full items-stretch",
                FilterInput {}
                IconButton {
                    icon: "ink_highlighter",
                    active: show_highlights,
                    class: "w-12 h-10 rounded-xl border border-[#2a2e33] bg-[#0d0f10] shadow-inset-input",
                    onclick: move |_| {
                        let mut state = use_context::<AppState>();
                        let current = (state.show_highlights)();
                        state.show_highlights.set(!current);
                    },
                    title: "Toggle Highlights"
                }
            }
            HighlightPanel { visible: show_highlights }
            DisplayOptions {}
        }
    }
}

#[component]
fn FilterInput() -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        div { class: "relative flex-1 group",
            span { class: "material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-gray-600 text-[20px] group-focus-within:text-primary transition-colors",
                "search"
            }
            input {
                class: "w-full bg-[#0d0f10] text-sm font-medium text-white placeholder-gray-600 pl-10 pr-30 py-2.5 rounded-xl border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none shadow-inset-input transition-all",
                placeholder: "Filter output...",
                "type": "text",
                value: "{state.filter_query}",
                oninput: move |evt| state.filter_query.set(evt.value()),
            }
            div { class: "absolute right-1.5 top-1/2 -translate-y-1/2 flex items-center gap-1",
                FilterOptionButton {
                    title: "Match Case",
                    label: "Aa",
                    active: (state.match_case)(),
                    onclick: move |_| {
                        let mut state = use_context::<AppState>();
                        let current = (state.match_case)();
                        state.match_case.set(!current);
                    }
                }
                FilterOptionButton {
                    title: "Use Regex",
                    label: ".*",
                    active: (state.use_regex)(),
                    onclick: move |_| {
                        let mut state = use_context::<AppState>();
                        let current = (state.use_regex)();
                        state.use_regex.set(!current);
                    }
                }
                FilterOptionButton {
                    title: "Invert Filter",
                    label: "!",
                    active: (state.invert_filter)(),
                    onclick: move |_| {
                        let mut state = use_context::<AppState>();
                        let current = (state.invert_filter)();
                        state.invert_filter.set(!current);
                    }
                }
            }
        }
    }
}

#[component]
fn FilterOptionButton(
    title: &'static str,
    label: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let state_class = if active {
        "bg-primary/10 border border-primary/20 text-primary shadow-[0_0_10px_rgba(0,191,255,0.15)]"
    } else {
        "text-gray-500 hover:text-white hover:bg-[#2a2e33]"
    };

    rsx! {
        button {
            class: "w-8 h-7 flex items-center justify-center rounded-md transition-all focus:outline-none {state_class}",
            title: "{title}",
            "aria-label": "{title}",
            onclick: move |evt| onclick.call(evt),
            span { class: "text-[11px] font-bold font-mono", "{label}" }
        }
    }
}

#[component]
fn HighlightPanel(visible: bool) -> Element {
    let state = use_context::<AppState>();
    let highlights = (state.highlights)();

    rsx! {
        div {
            class: "highlight-panel overflow-hidden transition-all duration-300 bg-surface rounded-xl border border-white/10 shadow-lg",
            class: if visible { "max-h-[400px] opacity-100 visible p-4 mt-2" } else { "max-h-0 opacity-0 invisible" },
            div { class: "flex flex-col gap-3",
                PanelHeader {
                    title: "Active Highlights",
                    subtitle: Some(format!("{} active rules", highlights.len()))
                }
                div { class: "flex flex-wrap gap-2",
                    for h in highlights {
                        HighlightTag {
                            color: h.color,
                            label: h.text.clone(),
                            onremove: move |_| {
                                let mut state = use_context::<AppState>();
                                let mut list = state.highlights.read().clone();
                                list.retain(|item| item.id != h.id);
                                state.highlights.set(list);
                            }
                        }
                    }
                }
                HighlightInput {}
            }
        }
    }
}

#[component]
fn HighlightTag(color: &'static str, label: String, onremove: EventHandler<MouseEvent>) -> Element {
    let (border_class, text_class) = match color {
        "red" => ("border-red-500/30 hover:border-red-500/60", "text-red-400"),
        "blue" => (
            "border-blue-500/30 hover:border-blue-500/60",
            "text-blue-400",
        ),
        "yellow" => (
            "border-yellow-500/30 hover:border-yellow-500/60",
            "text-yellow-400",
        ),
        "green" => (
            "border-green-500/30 hover:border-green-500/60",
            "text-green-400",
        ),
        "purple" => (
            "border-purple-500/30 hover:border-purple-500/60",
            "text-purple-400",
        ),
        "orange" => (
            "border-orange-500/30 hover:border-orange-500/60",
            "text-orange-400",
        ),
        "teal" => (
            "border-teal-500/30 hover:border-teal-500/60",
            "text-teal-400",
        ),
        "pink" => (
            "border-pink-500/30 hover:border-pink-500/60",
            "text-pink-400",
        ),
        "indigo" => (
            "border-indigo-500/30 hover:border-indigo-500/60",
            "text-indigo-400",
        ),
        "lime" => (
            "border-lime-500/30 hover:border-lime-500/60",
            "text-lime-400",
        ),
        "cyan" => (
            "border-cyan-500/30 hover:border-cyan-500/60",
            "text-cyan-400",
        ),
        "rose" => (
            "border-rose-500/30 hover:border-rose-500/60",
            "text-rose-400",
        ),
        "fuchsia" => (
            "border-fuchsia-500/30 hover:border-fuchsia-500/60",
            "text-fuchsia-400",
        ),
        "amber" => (
            "border-amber-500/30 hover:border-amber-500/60",
            "text-amber-400",
        ),
        "emerald" => (
            "border-emerald-500/30 hover:border-emerald-500/60",
            "text-emerald-400",
        ),
        "sky" => ("border-sky-500/30 hover:border-sky-500/60", "text-sky-400"),
        "violet" => (
            "border-violet-500/30 hover:border-violet-500/60",
            "text-violet-400",
        ),
        _ => ("border-primary/30 hover:border-primary/60", "text-primary"),
    };

    rsx! {
        div { class: "flex items-center gap-2 pl-3 pr-2 py-1.5 bg-[#0d0f10] border {border_class} rounded-full group transition-colors",
            span { class: "text-xs font-bold {text_class}", "{label}" }
            IconButton {
                icon: "close",
                icon_class: "text-[14px]",
                class: "ml-1 w-4 h-4 rounded-full",
                onclick: move |evt| onremove.call(evt)
            }
        }
    }
}

#[component]
fn HighlightInput() -> Element {
    let mut new_text = use_signal(|| String::new());

    let mut add_highlight_logic = move || {
        let text = new_text.read().trim().to_string();
        if !text.is_empty() {
            let mut state = use_context::<AppState>();
            let mut list = state.highlights.read().clone();

            let used_colors: std::collections::HashSet<&str> =
                list.iter().map(|h| h.color).collect();
            let color = HIGHLIGHT_COLORS
                .iter()
                .find(|&&c| !used_colors.contains(c))
                .copied()
                .unwrap_or_else(|| HIGHLIGHT_COLORS[list.len() % HIGHLIGHT_COLORS.len()]);

            let next_id = list.iter().map(|h| h.id).max().unwrap_or(0) + 1;
            list.push(Highlight {
                id: next_id,
                text,
                color,
            });
            state.highlights.set(list);
            new_text.set(String::new());
        }
    };

    rsx! {
        div { class: "pt-2 border-t border-white/5 flex gap-2",
            input {
                class: "flex-1 bg-[#0d0f10] text-xs font-medium text-white placeholder-gray-600 px-3 py-2 rounded-lg border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none transition-all",
                placeholder: "Enter keyword to highlight...",
                "type": "text",
                value: "{new_text}",
                oninput: move |evt| new_text.set(evt.value()),
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter {
                        add_highlight_logic();
                    }
                }
            }
            button {
                class: "px-4 rounded-lg bg-primary text-surface font-bold hover:bg-white transition-all active:scale-95 flex items-center gap-2",
                onclick: move |_| add_highlight_logic(),
                span { class: "material-symbols-outlined text-[18px]", "add" }
                span { class: "text-[10px] uppercase tracking-wider", "Add" }
            }
        }
    }
}

#[component]
fn DisplayOptions() -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        div { class: "flex items-center gap-6",
            ToggleSwitch {
                label: "Timestamp",
                active: (state.show_timestamps)(),
                onclick: move |_| {
                    let current = (state.show_timestamps)();
                    state.show_timestamps.set(!current);
                }
            }
            ToggleSwitch {
                label: "Auto-scroll",
                active: (state.autoscroll)(),
                onclick: move |_| {
                    let current = (state.autoscroll)();
                    state.autoscroll.set(!current);
                }
            }
            div { class: "ml-auto text-[10px] font-bold text-gray-500 uppercase tracking-widest flex items-center gap-2",
                span { class: "w-1.5 h-1.5 rounded-full bg-primary animate-pulse" }
                "Live"
            }
        }
    }
}
