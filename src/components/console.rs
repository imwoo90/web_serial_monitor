use super::serial_monitor::AppState;
use dioxus::prelude::*;

#[component]
pub fn Console() -> Element {
    let mut state = use_context::<AppState>();
    let show_timestamps = (state.show_timestamps)();
    let autoscroll = (state.autoscroll)();

    rsx! {
        main { class: "flex-1 min-h-0 mx-4 mb-0 mt-0 relative group/console",
            div { class: "absolute inset-0 bg-console-bg rounded-t-2xl border-t border-x border-[#222629] shadow-[inset_0_0_20px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col",
                div { class: "absolute inset-0 scanlines opacity-20 pointer-events-none z-10" }
                div { class: "shrink-0 h-6 bg-[#16181a] border-b border-[#222629] flex items-center justify-between px-3",
                    div { class: "flex gap-1.5",
                        div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                        div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                        div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                    }
                    div { class: "flex items-center gap-2",
                        if autoscroll {
                            div { class: "text-[9px] font-mono text-primary/60 uppercase tracking-widest", "Tracking Bottom" }
                        } else {
                            div { class: "text-[9px] font-mono text-yellow-500/60 uppercase tracking-widest", "Scroll Paused" }
                        }
                        div { class: "text-[9px] font-mono text-[#4a555a] uppercase tracking-widest", "/dev/tty.usbserial" }
                    }
                }
                div {
                    class: "flex-1 overflow-y-auto p-4 font-mono text-xs md:text-sm leading-relaxed space-y-0.5 scrollbar-custom",
                    id: "console-output",

                    // Simple example logs with conditional timestamps
                    for (timestamp, text, class_name) in [
                        ("[10:41:58]", "Connecting to port COM3...", "text-gray-500 opacity-50"),
                        ("[10:42:01]", "System initialization started", "text-gray-300"),
                        ("[10:42:01]", "Bootloader v2.1.4 check... PASS", "text-gray-300"),
                        ("[10:42:02]", "Loading kernel modules", "text-gray-300"),
                        ("[10:42:03]", "RX: <DATA_PACKET_01 id=\"442\" val=\"0x4F\">", "text-gray-300"),
                    ] {
                        div { class: "flex gap-2 {class_name}",
                            if show_timestamps {
                                span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none", "{timestamp}" }
                            }
                            span { "{text}" }
                        }
                    }
                }

                if !autoscroll {
                    button {
                        class: "absolute bottom-6 right-6 bg-primary text-surface rounded-full w-10 h-10 shadow-lg shadow-black/50 hover:bg-white active:scale-95 transition-all duration-300 z-20 flex items-center justify-center cursor-pointer group/fab",
                        onclick: move |_| state.autoscroll.set(true),
                        span { class: "material-symbols-outlined text-[20px] font-bold", "arrow_downward" }
                        span { class: "absolute -top-8 right-0 bg-surface text-[9px] font-bold text-gray-300 px-2 py-1 rounded border border-white/5 opacity-0 group-hover/fab:opacity-100 transition-opacity whitespace-nowrap pointer-events-none uppercase tracking-widest",
                            "Resume Scroll"
                        }
                    }
                }
            }
        }
    }
}
