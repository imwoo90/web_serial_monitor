use crate::state::AppState;
use dioxus::prelude::*;
use std::rc::Rc;

#[component]
pub fn Console() -> Element {
    let mut state = use_context::<AppState>();
    let show_timestamps = (state.show_timestamps)();
    let autoscroll = (state.autoscroll)();
    let highlights = (state.highlights)();
    let filter_query = (state.filter_query)();
    let match_case = (state.match_case)();
    let use_regex = (state.use_regex)();
    let invert_filter = (state.invert_filter)();

    // 하단 감시 요소(Sentinel)의 핸들
    let mut sentinel_handle = use_signal(|| None::<Rc<MountedData>>);

    // Dioxus Idiomatic Scroll:
    // JS 문자열 대신 감시 요소를 '화면 안으로 끌어당기는' 네이티브 API 호출
    use_effect(move || {
        if autoscroll {
            if let Some(handle) = sentinel_handle.read().as_ref() {
                // 이 핸들이 가리키는 요소를 즉시 화면 바닥에 정렬하도록 명령
                let _ = handle.scroll_to(ScrollBehavior::Instant);
            }
        }
    });

    rsx! {
        main { class: "flex-1 min-h-0 mx-4 mb-0 mt-0 relative group/console",
            div { class: "absolute inset-0 bg-console-bg rounded-t-2xl border-t border-x border-[#222629] shadow-[inset_0_0_20px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col",
                div { class: "absolute inset-0 scanlines opacity-20 pointer-events-none z-10" }
                ConsoleHeader { autoscroll }

                div {
                    class: "flex-1 overflow-y-auto p-4 font-mono text-xs md:text-sm leading-relaxed space-y-0.5 scrollbar-custom",
                    id: "console-output",
                    // 1. 로그 리스트 출력
                    for (timestamp , text , base_class) in get_mock_logs() {
                        LogLine {
                            timestamp,
                            text,
                            base_class,
                            show_timestamps,
                            filter_query: filter_query.clone(),
                            match_case,
                            use_regex,
                            invert_filter,
                            highlights: highlights.clone(),
                        }
                    }

                    // 2. 가시성 감시 및 스크롤 타겟 (Idiomatic Dioxus Sentinel)
                    div {
                        class: "h-px w-full pointer-events-none opacity-0",
                        // 사용자의 스크롤 위치 감지
                        onvisible: move |evt| {
                            let visible = evt.data().is_intersecting().unwrap_or(false);
                            if (state.autoscroll)() != visible {
                                state.autoscroll.set(visible);
                            }
                        },
                        // 이 요소의 핸들을 시그널에 보관
                        onmounted: move |evt| sentinel_handle.set(Some(evt.data())),
                    }
                }
                if !autoscroll {
                    ResumeScrollButton {
                        onclick: move |_| {
                            state.autoscroll.set(true);
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn ConsoleHeader(autoscroll: bool) -> Element {
    rsx! {
        div { class: "shrink-0 h-6 bg-[#16181a] border-b border-[#222629] flex items-center justify-between px-3",
            div { class: "flex gap-1.5",
                div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
            }
            div { class: "flex items-center gap-2",
                if autoscroll {
                    div { class: "text-[9px] font-mono text-primary/60 uppercase tracking-widest flex items-center gap-1",
                        span { class: "w-1 h-1 rounded-full bg-primary animate-pulse" }
                        "Tracking Bottom"
                    }
                } else {
                    div { class: "text-[9px] font-mono text-yellow-500/60 uppercase tracking-widest",
                        "Scroll Paused"
                    }
                }
                div { class: "text-[9px] font-mono text-[#4a555a] uppercase tracking-widest",
                    "/dev/tty.usbserial"
                }
            }
        }
    }
}

#[component]
fn LogLine(
    timestamp: &'static str,
    text: &'static str,
    base_class: &'static str,
    show_timestamps: bool,
    filter_query: String,
    match_case: bool,
    use_regex: bool,
    invert_filter: bool,
    highlights: Vec<crate::state::Highlight>,
) -> Element {
    let mut is_visible = if filter_query.is_empty() {
        true
    } else if use_regex {
        if let Ok(re) = if match_case {
            regex::Regex::new(&filter_query)
        } else {
            regex::RegexBuilder::new(&filter_query)
                .case_insensitive(true)
                .build()
        } {
            re.is_match(text)
        } else {
            true
        }
    } else {
        if match_case {
            text.contains(&filter_query)
        } else {
            text.to_lowercase().contains(&filter_query.to_lowercase())
        }
    };

    if invert_filter {
        is_visible = !is_visible;
    }

    if !is_visible {
        return rsx! {
            div { class: "hidden" }
        };
    }

    let mut parts = vec![text.to_string()];
    for h in highlights.iter() {
        let mut new_parts = Vec::new();
        for part in parts {
            if part.contains(&h.text) && !part.starts_with("\x01") {
                let split_parts: Vec<&str> = part.split(&h.text).collect();
                for (i, p) in split_parts.iter().enumerate() {
                    if i > 0 {
                        new_parts.push(format!("\x01{}\x01{}", h.id, h.text));
                    }
                    if !p.is_empty() {
                        new_parts.push(p.to_string());
                    }
                }
            } else {
                new_parts.push(part);
            }
        }
        parts = new_parts;
    }

    rsx! {
        div { class: "flex gap-2 {base_class} transition-all duration-300",
            if show_timestamps {
                span { class: "log-timestamp text-[#4a555a] shrink-0 tabular-nums select-none",
                    "{timestamp}"
                }
            }
            span { class: "whitespace-pre-wrap",
                for part in parts {
                    if part.starts_with("\x01") {
                        {
                            let marker_end = part[1..].find('\x01').unwrap_or(0) + 1;
                            let id_str = &part[1..marker_end];
                            let match_text = &part[marker_end + 1..];
                            let id = id_str.parse::<usize>().unwrap_or(0);

                            let h = highlights.iter().find(|h| h.id == id);
                            let color = h.map(|h| h.color).unwrap_or("primary");
                            let highlight_class = get_highlight_class(color);

                            rsx! {
                                span { class: "{highlight_class}", "{match_text}" }
                            }
                        }
                    } else {
                        "{part}"
                    }
                }
            }
        }
    }
}

#[component]
fn ResumeScrollButton(onclick: EventHandler<MouseEvent>) -> Element {
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

fn get_highlight_class(color: &str) -> &'static str {
    match color {
        "red" => "text-red-400 font-bold",
        "blue" => "text-blue-400 font-bold",
        "yellow" => "text-yellow-400 font-bold",
        "green" => "text-green-400 font-bold",
        "purple" => "text-purple-400 font-bold",
        "orange" => "text-orange-400 font-bold",
        "teal" => "text-teal-400 font-bold",
        "pink" => "text-pink-400 font-bold",
        "indigo" => "text-indigo-400 font-bold",
        "lime" => "text-lime-400 font-bold",
        "cyan" => "text-cyan-400 font-bold",
        "rose" => "text-rose-400 font-bold",
        "fuchsia" => "text-fuchsia-400 font-bold",
        "amber" => "text-amber-400 font-bold",
        "emerald" => "text-emerald-400 font-bold",
        "sky" => "text-sky-400 font-bold",
        "violet" => "text-violet-400 font-bold",
        _ => "text-primary font-bold",
    }
}

fn get_mock_logs() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            "[10:41:58]",
            "Connecting to port COM3...",
            "text-gray-500 opacity-50",
        ),
        (
            "[10:42:01]",
            "System initialization started",
            "text-gray-300",
        ),
        (
            "[10:42:01]",
            "Bootloader v2.1.4 check... PASS",
            "text-gray-300",
        ),
        ("[10:42:02]", "Loading kernel modules", "text-gray-300"),
        (
            "[10:42:03]",
            "RX: <DATA_PACKET_01 id=\"442\" val=\"0x4F\">",
            "text-gray-300",
        ),
        (
            "[10:42:08]",
            "Warning: Pressure drift detected (-1hPa)",
            "text-gray-300",
        ),
        ("[10:42:15]", "Sensor array read complete", "text-gray-300"),
    ]
}
