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
