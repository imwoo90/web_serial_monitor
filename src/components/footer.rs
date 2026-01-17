use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "shrink-0 py-4 px-6 bg-[#0d0f10] border-t border-[#2a2e33] flex items-center justify-between text-[11px] text-gray-500 font-mono z-20",
            div { class: "flex items-center gap-2",
                span { "Web Serial Monitor" }
                span { class: "w-1 h-1 rounded-full bg-gray-700" }
                span { "v1.0.0" }
            }
            div { class: "flex items-center gap-4",
                a {
                    class: "hover:text-primary transition-colors flex items-center gap-1.5 group",
                    href: "https://github.com/imwoo90/web_serial_monitor", // Assuming this from repo path, can be placeholder
                    target: "_blank",
                    span { class: "material-symbols-outlined text-[14px]", "code" }
                    span { "GitHub" }
                }
            }
        }
    }
}
