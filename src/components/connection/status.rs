use dioxus::prelude::*;

#[component]
pub fn PortStatus(connected: bool) -> Element {
    rsx! {
        div { class: "flex items-center gap-2 px-3 py-1.5 bg-[#16181a] rounded-lg border border-[#2a2e33] h-9",
            if connected {
                span { class: "material-symbols-outlined text-emerald-500 text-[18px]",
                    "usb"
                }
                span { class: "text-xs font-bold text-emerald-500 font-mono", "Connected" }
            } else {
                span { class: "material-symbols-outlined text-gray-500 text-[18px]",
                    "usb_off"
                }
                span { class: "text-xs font-bold text-gray-500 font-mono", "No Device" }
            }
        }
    }
}
