use crate::state::LineEnding;
use dioxus::prelude::*;

#[component]
pub fn LineEndSelector(
    label: &'static str,
    selected: LineEnding,
    onselect: EventHandler<LineEnding>,
    active_class: &'static str,
    is_rx: bool,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-2",
            span { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest",
                "{label}"
            }
            div { class: "flex bg-[#0d0f10] p-0.5 rounded-lg border border-[#2a2e33]",
                for ending in [LineEnding::None, LineEnding::NL, LineEnding::CR, LineEnding::NLCR] {
                    button {
                        class: "px-2 py-1 rounded text-[10px] font-bold transition-all duration-200",
                        class: if selected == ending { "{active_class} border shadow-sm" } else { "text-gray-500 hover:text-white" },
                        onclick: move |_| onselect.call(ending),
                        match ending {
                            LineEnding::None => if is_rx { "RAW" } else { "NONE" },
                            LineEnding::NL => "LF",
                            LineEnding::CR => "CR",
                            LineEnding::NLCR => "CRLF",
                        }
                    }
                }
            }
        }
    }
}

/// A reusable custom select component with premium styling.
#[component]
pub fn CustomSelect(
    options: Vec<&'static str>,
    selected: String,
    onchange: EventHandler<String>,
    #[props(default = "w-full")] class: &'static str,
    #[props(default = false)] disabled: bool,
) -> Element {
    let mut is_open = use_signal(|| false);

    rsx! {
        div { class: "relative {class} group/select",
            button {
                class: if disabled { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg text-xs font-bold text-gray-500 py-2 px-3 opacity-50 cursor-not-allowed" } else { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg text-xs font-bold text-gray-300 py-2 px-3 hover:bg-[#16181a] hover:border-primary/50 transition-all duration-200 outline-none focus:border-primary/50" },
                disabled: "{disabled}",
                onclick: move |e| {
                    if !disabled {
                        e.stop_propagation();
                        is_open.set(!is_open());
                    }
                },
                span { "{selected}" }
                span {
                    class: "material-symbols-outlined text-[18px] text-gray-500 group-hover/select:text-primary transition-all duration-300",
                    class: if is_open() { "rotate-180 text-primary" } else { "" },
                    "expand_more"
                }
            }

            if is_open() {
                div {
                    class: "fixed inset-0 z-40 cursor-default",
                    onclick: move |_| is_open.set(false),
                }

                div { class: "absolute top-full left-0 right-0 mt-1 z-50 bg-[#16181a] border border-white/10 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200",
                    for opt in options {
                        button {
                            class: "w-full text-left px-3 py-2 text-[11px] font-bold transition-all duration-150",
                            class: if opt == selected { "bg-primary/20 text-primary" } else { "text-gray-400 hover:bg-white/5 hover:text-white" },
                            onclick: move |_| {
                                onchange.call(opt.to_string());
                                is_open.set(false);
                            },
                            "{opt}"
                        }
                    }
                }
            }
        }
    }
}

/// A select component that also allows custom user input
#[component]
pub fn CustomInputSelect(
    options: Vec<&'static str>,
    selected: String,
    onchange: EventHandler<String>,
    #[props(default = "w-full")] class: &'static str,
    #[props(default = false)] disabled: bool,
) -> Element {
    let mut is_open = use_signal(|| false);

    rsx! {
        div { class: "relative {class} group/select",
            div {
                class: if disabled { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg py-0 px-0 opacity-50 cursor-not-allowed" } else { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg py-0 px-0 hover:bg-[#16181a] hover:border-primary/50 transition-all duration-200 focus-within:border-primary/50" },

                // Input field for custom value
                input {
                    class: if disabled { "w-full bg-transparent border-none text-xs font-bold text-gray-300 py-2 pl-3 outline-none placeholder-gray-600 cursor-not-allowed" } else { "w-full bg-transparent border-none text-xs font-bold text-gray-300 py-2 pl-3 outline-none placeholder-gray-600" },
                    value: "{selected}",
                    disabled: "{disabled}",
                    oninput: move |evt| {
                        onchange.call(evt.value());
                    },
                    onfocus: move |_| is_open.set(false), // Close dropdown when typing
                }

                // Dropdown trigger button
                button {
                    class: if disabled { "flex items-center justify-center px-2 py-2 outline-none border-l border-[#2a2e33] cursor-not-allowed" } else { "flex items-center justify-center px-2 py-2 outline-none border-l border-[#2a2e33] cursor-pointer" },
                    disabled: "{disabled}",
                    onclick: move |e| {
                        if !disabled {
                            e.stop_propagation();
                            is_open.set(!is_open());
                        }
                    },
                    span {
                        class: "material-symbols-outlined text-[18px] text-gray-500 group-hover/select:text-primary transition-all duration-300",
                        class: if is_open() { "rotate-180 text-primary" } else { "" },
                        "expand_more"
                    }
                }
            }

            if is_open() {
                div {
                    class: "fixed inset-0 z-40 cursor-default",
                    onclick: move |_| is_open.set(false),
                }

                div { class: "absolute top-full left-0 right-0 mt-1 z-50 bg-[#16181a] border border-white/10 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200 max-h-60 overflow-y-auto custom-scrollbar",
                    for opt in options {
                        button {
                            class: "w-full text-left px-3 py-2 text-[11px] font-bold transition-all duration-150",
                            class: if opt == selected { "bg-primary/20 text-primary" } else { "text-gray-400 hover:bg-white/5 hover:text-white" },
                            onclick: move |_| {
                                onchange.call(opt.to_string());
                                is_open.set(false);
                            },
                            "{opt}"
                        }
                    }
                }
            }
        }
    }
}

/// A toggle switch for boolean settings.
#[component]
pub fn ToggleSwitch(
    label: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            class: "flex items-center cursor-pointer group gap-2",
            onclick: move |evt| onclick.call(evt),
            div { class: "relative flex items-center",
                div {
                    class: "w-7 h-3.5 rounded-full transition-all duration-200 border border-white/5",
                    class: if active { "bg-primary border-primary shadow-[0_0_8px_rgba(0,191,255,0.4)]" } else { "bg-[#2a2e33] group-hover:bg-[#34393e]" },
                }
                div {
                    class: "absolute left-0 w-3.5 h-3.5 rounded-full transition-all duration-200",
                    class: if active { "translate-x-3.5 bg-white" } else { "bg-gray-500" },
                }
            }
            span {
                class: "text-[10px] font-bold uppercase tracking-widest transition-colors leading-none",
                class: if active { "text-primary" } else { "text-gray-500 group-hover:text-gray-300" },
                "{label}"
            }
        }
    }
}
