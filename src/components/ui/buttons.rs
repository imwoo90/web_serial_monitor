use dioxus::prelude::*;

/// A reusable icon-only button with consistent hover and active states.
#[component]
pub fn IconButton(
    icon: &'static str,
    onclick: EventHandler<MouseEvent>,
    #[props(default = false)] active: bool,
    #[props(default = "")] title: &'static str,
    #[props(default = "")] class: &'static str,
    #[props(default = "text-[20px]")] icon_class: &'static str,
) -> Element {
    rsx! {
        button {
            class: "flex items-center justify-center transition-all active:scale-95 {class}",
            class: if active { "text-primary bg-primary/10 border-primary/50" } else { "text-gray-500 hover:text-white" },
            onclick: move |evt| onclick.call(evt),
            title: "{title}",
            span { class: "material-symbols-outlined {icon_class}", "{icon}" }
        }
    }
}

#[component]
pub fn FilterOptionButton(
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

/// A floating action button (FAB) that appears when auto-scroll is paused.
/// Typically positioned absolute bottom-right relative to its container.
#[component]
pub fn ResumeScrollButton(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "absolute bottom-6 right-6 bg-primary text-surface rounded-full w-10 h-10 shadow-lg shadow-black/50 hover:bg-white active:scale-95 transition-all duration-300 z-20 flex items-center justify-center cursor-pointer group/fab",
            onclick: move |evt| onclick.call(evt),
            span { class: "material-symbols-outlined text-[20px] font-bold", "arrow_downward" }
            span { class: "absolute -top-8 right-0 bg-surface text-[9px] font-bold text-gray-300 px-2 py-1 rounded border border-white/5 opacity-0 group-hover/fab:opacity-100 transition-opacity whitespace-nowrap pointer-events-none uppercase tracking-widest",
                "Resume Scroll"
            }
        }
    }
}
