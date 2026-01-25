use crate::components::ui::FilterOptionButton;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn SearchBar() -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        div { class: "flex-[1.3] relative group flex items-center min-w-0",
            span { class: "material-symbols-outlined absolute left-3 text-gray-600 text-[18px] group-focus-within:text-primary transition-colors",
                "search"
            }
            input {
                class: "w-full h-full bg-[#0d0f10] text-xs font-medium text-white placeholder-gray-600 pl-9 pr-24 rounded-lg border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none shadow-inset-input transition-all",
                placeholder: "Filter logs...",
                "type": "text",
                value: "{state.log.filter_query}",
                oninput: move |evt| state.log.filter_query.set(evt.value()),
            }
            div { class: "absolute right-1 flex items-center gap-0.5",
                FilterOptionButton {
                    title: "Match Case",
                    label: "Aa",
                    active: (state.log.match_case)(),
                    onclick: move |_| {
                        let v = (state.log.match_case)();
                        state.log.match_case.set(!v);
                    },
                }
                FilterOptionButton {
                    title: "Regex",
                    label: ".*",
                    active: (state.log.use_regex)(),
                    onclick: move |_| {
                        let v = (state.log.use_regex)();
                        state.log.use_regex.set(!v);
                    },
                }
                FilterOptionButton {
                    title: "Invert",
                    label: "!",
                    active: (state.log.invert_filter)(),
                    onclick: move |_| {
                        let v = (state.log.invert_filter)();
                        state.log.invert_filter.set(!v);
                    },
                }
            }
        }
    }
}
