use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn SettingsPanel() -> Element {
    let state = use_context::<AppState>();
    let is_open = (state.show_settings)();

    rsx! {
        div {
            class: "mx-5 bg-surface rounded-xl border border-white/10 shadow-2xl transition-all duration-300 overflow-hidden z-50 absolute top-full left-0 right-0",
            class: if is_open { "max-h-[500px] opacity-100 visible p-4 mt-3" } else { "max-h-0 opacity-0 invisible mt-2" },
            id: "settings-panel",
            style: "width: calc(100% - 2.5rem);",
            div { class: "grid grid-cols-2 gap-x-4 gap-y-3",
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Data Bits"
                    }
                    div { class: "relative group",
                        select { class: "w-full appearance-none bg-[#0d0f10]! border border-[#2a2e33] rounded-lg text-xs font-medium text-gray-300 py-2 pl-3 pr-8 focus:border-primary/50 outline-none transition-colors cursor-pointer hover:bg-[#16181a]",
                            option { "5" }
                            option { "6" }
                            option { "7" }
                            option { selected: true, "8" }
                        }
                        div { class: "pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-500 group-hover:text-primary transition-colors",
                            span { class: "material-symbols-outlined text-[18px]", "expand_more" }
                        }
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Stop Bits"
                    }
                    div { class: "relative group",
                        select { class: "w-full appearance-none bg-[#0d0f10]! border border-[#2a2e33] rounded-lg text-xs font-medium text-gray-300 py-2 pl-3 pr-8 focus:border-primary/50 outline-none transition-colors cursor-pointer hover:bg-[#16181a]",
                            option { selected: true, "1" }
                            option { "1.5" }
                            option { "2" }
                        }
                        div { class: "pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-500 group-hover:text-primary transition-colors",
                            span { class: "material-symbols-outlined text-[18px]", "expand_more" }
                        }
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Parity"
                    }
                    div { class: "relative group",
                        select { class: "w-full appearance-none bg-[#0d0f10]! border border-[#2a2e33] rounded-lg text-xs font-medium text-gray-300 py-2 pl-3 pr-8 focus:border-primary/50 outline-none transition-colors cursor-pointer hover:bg-[#16181a]",
                            option { selected: true, "None" }
                            option { "Even" }
                            option { "Odd" }
                            option { "Mark" }
                            option { "Space" }
                        }
                        div { class: "pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-500 group-hover:text-primary transition-colors",
                            span { class: "material-symbols-outlined text-[18px]", "expand_more" }
                        }
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Flow Control"
                    }
                    div { class: "relative group",
                        select { class: "w-full appearance-none bg-[#0d0f10]! border border-[#2a2e33] rounded-lg text-xs font-medium text-gray-300 py-2 pl-3 pr-8 focus:border-primary/50 outline-none transition-colors cursor-pointer hover:bg-[#16181a]",
                            option { selected: true, "None" }
                            option { "Hardware" }
                            option { "Software" }
                        }
                        div { class: "pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-500 group-hover:text-primary transition-colors",
                            span { class: "material-symbols-outlined text-[18px]", "expand_more" }
                        }
                    }
                }
            }
        }
    }
}
