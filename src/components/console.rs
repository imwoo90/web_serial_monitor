use crate::state::{AppState, Highlight};
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, Worker};

/// Web Worker와 통신하기 위한 메시지 프로토콜
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
enum WorkerMsg {
    #[serde(rename = "INITIALIZED")]
    Initialized(String),
    #[serde(rename = "TOTAL_LINES")]
    TotalLines(usize),
    #[serde(rename = "LOG_WINDOW")]
    LogWindow {
        #[serde(rename = "startLine")]
        start_line: usize,
        lines: Vec<String>,
    },
    #[serde(rename = "APPEND_LOG")]
    AppendLog(String),
    #[serde(rename = "REQUEST_WINDOW")]
    RequestWindow {
        #[serde(rename = "startLine")]
        start_line: usize,
        count: usize,
    },
}

/// 한 줄 높이 (px) 정의
const LINE_HEIGHT: f64 = 20.0;
/// 헤더 및 여백 높이 (px)
const HEADER_OFFSET: f64 = 150.0;
/// 가상 스크롤 렌더링을 위한 상단 버퍼 (줄)
const TOP_BUFFER: usize = 10;
/// 가상 스크롤 렌더링을 위한 하단 추가 버퍼 (줄)
const BOTTOM_BUFFER_EXTRA: usize = 40; // window_size 계산 시 TOP_BUFFER + 40

#[component]
pub fn Console() -> Element {
    let mut state = use_context::<AppState>();

    // Core Signals
    let mut worker = use_signal(|| None::<Worker>);
    let mut visible_logs = use_signal(|| Vec::<String>::new());
    let mut total_lines = use_signal(|| 0usize);
    let mut start_index = use_signal(|| 0usize);

    // Layout Signals
    let mut console_height = use_signal(|| 600.0);
    // 윈도우 사이즈 계산 (상하단 충분한 버퍼 확보)
    let window_size =
        ((console_height() / LINE_HEIGHT).ceil() as usize) + TOP_BUFFER + BOTTOM_BUFFER_EXTRA;

    // DOM Handles
    let mut console_handle = use_signal(|| None::<Rc<MountedData>>);
    let mut sentinel_handle = use_signal(|| None::<Rc<MountedData>>);

    // 1. Worker 초기화 및 메시지 핸들러
    use_effect(move || {
        let worker_path = asset!("/assets/log_worker.js").to_string();
        let w = Worker::new(&worker_path).expect("Failed to create worker");

        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(msg) = serde_wasm_bindgen::from_value::<WorkerMsg>(e.data()) {
                match msg {
                    WorkerMsg::TotalLines(count) => total_lines.set(count),
                    WorkerMsg::LogWindow { lines, .. } => visible_logs.set(lines),
                    _ => {}
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        w.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
        worker.set(Some(w));
    });

    // 2. Window Resize 핸들러 (패닉 방지를 위한 동기적 처리)
    use_effect(move || {
        let mut update_height = move || {
            let window = web_sys::window().unwrap();
            if let Ok(h) = window.inner_height() {
                if let Some(h) = h.as_f64() {
                    console_height.set((h - HEADER_OFFSET).max(100.0));

                    // 자동 스크롤 활성화 시, 리사이즈 중에도 바닥 고정 강제
                    if (state.autoscroll)() {
                        if let Some(sentinel) = sentinel_handle.peek().as_ref() {
                            let _ = sentinel.scroll_to(ScrollBehavior::Instant);
                        }
                    }
                }
            }
        };

        // 초기 실행
        update_height();

        let onresize = Closure::wrap(Box::new(update_height) as Box<dyn FnMut()>);
        let window = web_sys::window().unwrap();
        window.set_onresize(Some(onresize.as_ref().unchecked_ref()));
        onresize.forget();
    });

    // 3. 로그 데이터 윈도우 요청 (start_index 또는 window_size 변경 시)
    use_effect(move || {
        let start = start_index();
        let size = window_size;
        total_lines(); // 전체 라인 수 변화도 구독

        if let Some(w) = worker.peek().as_ref() {
            let msg = WorkerMsg::RequestWindow {
                start_line: start,
                count: size,
            };
            if let Ok(js_obj) = serde_wasm_bindgen::to_value(&msg) {
                let _ = w.post_message(&js_obj);
            }
        }
    });

    // 4. 자동 스크롤 (Tracking Bottom)
    use_effect(move || {
        total_lines();
        if (state.autoscroll)() {
            if let Some(handle) = sentinel_handle.peek().as_ref() {
                let _ = handle.scroll_to(ScrollBehavior::Instant);
            }
        }
    });

    // 5. 테스트용 로그 시뮬레이터 (추후 제거 가능)
    use_resource(move || async move {
        let mut count = 0;
        loop {
            TimeoutFuture::new(50).await;
            if let Some(w) = worker.peek().as_ref() {
                let now = js_sys::Date::new_0();
                let log = format!(
                    "[{:02}:{:02}:{:02}] RX DATA: PKT_{:05} STATUS=OK TEMP=24.5C",
                    now.get_hours(),
                    now.get_minutes(),
                    now.get_seconds(),
                    count
                );
                let msg = WorkerMsg::AppendLog(log);
                if let Ok(js_obj) = serde_wasm_bindgen::to_value(&msg) {
                    let _ = w.post_message(&js_obj);
                }
                count += 1;
            }
        }
    });

    let total_height = (total_lines() as f64) * LINE_HEIGHT;
    let offset_top = (start_index() as f64) * LINE_HEIGHT;

    rsx! {
        main { class: "flex-1 min-h-0 mx-4 mb-0 mt-0 relative group/console",
            div { class: "absolute inset-0 bg-console-bg rounded-t-2xl border-t border-x border-[#222629] shadow-[inset_0_0_20px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col",
                // 스캔라인 효과 (데코레이션)
                div { class: "absolute inset-0 scanlines opacity-20 pointer-events-none z-10" }

                ConsoleHeader { autoscroll: (state.autoscroll)(), count: total_lines() }

                div {
                    class: "flex-1 overflow-y-auto font-mono text-xs md:text-sm leading-relaxed scrollbar-custom relative",
                    id: "console-output",
                    // 마운트 시 핸들 저장 및 초기 높이 설정
                    onmounted: move |evt| {
                        let handle = evt.data();
                        let h_clone = handle.clone();
                        spawn(async move {
                            if let Ok(rect) = h_clone.get_client_rect().await {
                                console_height.set(rect.height());
                            }
                        });
                        console_handle.set(Some(handle));
                    },
                    // 스크롤 핸들러: 가상 윈도우 인덱스 계산
                    onscroll: move |_| {
                        let handle = console_handle.peek().as_ref().cloned();
                        spawn(async move {
                            if let Some(handle) = handle {
                                if let Ok(offset) = handle.get_scroll_offset().await {
                                    let raw_index = (offset.y / LINE_HEIGHT).floor() as usize;
                                    // 상단 버퍼만큼 미리 렌더링
                                    let new_index = raw_index.saturating_sub(TOP_BUFFER);

                                    if start_index() != new_index {
                                        start_index.set(new_index);
                                    }
                                }
                            }
                        });
                    },

                    // 1. 가상 스크롤 높이 확보용 Spacer
                    div { style: "height: {total_height}px; width: 100%; position: absolute; top: 0; left: 0; pointer-events: none;" }

                    // 2. 실제 렌더링 영역 (Transform으로 위치 조정)
                    // padding-bottom 20px 추가로 하단 잘림 방지 (SafeArea)
                    div { style: "position: absolute; top: 0; left: 0; right: 0; transform: translateY({offset_top}px); padding: 0.5rem 1rem 20px 1rem; pointer-events: auto;",
                        {
                            // ReadGuard 수명 문제를 피하기 위해 데이터 전체를 복제
                            let highlights = (state.highlights)().clone();
                            let show_timestamps = (state.show_timestamps)();
                            let show_highlights = (state.show_highlights)();

                            visible_logs
                                .read()
                                .iter()
                                .filter(|text| {
                                    let query = (state.filter_query)();
                                    query.is_empty() || text.contains(&*query)
                                })
                                .map(move |text| {
                                    let segments = process_log_segments(
                                        text,
                                        &highlights,
                                        show_timestamps,
                                        show_highlights,
                                    );
                                    rsx! {
                                        div {
                                            style: "height: {LINE_HEIGHT}px; line-height: {LINE_HEIGHT}px;",
                                            class: "text-gray-300 whitespace-pre text-[12px]",
                                            for (content , color) in segments {
                                                if let Some(c) = color {
                                                    span { class: "font-bold", style: "color: {c};", "{content}" }
                                                } else {
                                                    "{content}"
                                                }
                                            }
                                        }
                                    }
                                })
                        }
                        // 초기 로딩 인디케이터
                        if visible_logs.read().is_empty() && total_lines() > 0 {
                            div { class: "text-gray-500 animate-pulse text-[12px] px-4",
                                "Loading buffer..."
                            }
                        }
                    }

                    // 3. 바닥 감시자 (Sentinel)
                    div {
                        style: "position: absolute; top: {total_height}px; height: 1px; width: 100%; pointer-events: none;",
                        onvisible: move |evt| {
                            let visible = evt.data().is_intersecting().unwrap_or(false);
                            if (state.autoscroll)() != visible {
                                state.autoscroll.set(visible);
                            }
                        },
                        onmounted: move |evt| sentinel_handle.set(Some(evt.data())),
                    }
                }

                if !(state.autoscroll)() {
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

/// 로그 텍스트를 처리하여 타임스탬프 제거 및 하이라이트 세그먼트로 분할
fn process_log_segments(
    text: &str,
    highlights: &[Highlight],
    show_timestamps: bool,
    show_highlights: bool,
) -> Vec<(String, Option<String>)> {
    // 1. Timestamp Parsing
    let content = if !show_timestamps && text.len() > 11 && text.starts_with('[') {
        &text[11..]
    } else {
        text
    };

    // 2. Highlighting
    let mut segments = vec![(content.to_string(), None::<String>)];

    if show_highlights {
        for h in highlights {
            if h.text.is_empty() {
                continue;
            }

            let mut next_segments = Vec::new();
            let mut found_for_keyword = false; // Reset for each highlight keyword

            for (seg_text, color) in segments {
                // 이미 색칠된 세그먼트는 패스
                if color.is_some() {
                    next_segments.push((seg_text, color));
                    continue;
                }

                // 키워드 검색 (라인 당 1회 제한)
                if !found_for_keyword && seg_text.contains(&h.text) {
                    if let Some((prefix, suffix)) = seg_text.split_once(&h.text) {
                        if !prefix.is_empty() {
                            next_segments.push((prefix.to_string(), None));
                        }
                        // Highlighted Keyword
                        next_segments.push((h.text.clone(), Some(h.color.to_string())));

                        if !suffix.is_empty() {
                            next_segments.push((suffix.to_string(), None));
                        }
                        found_for_keyword = true; // Mark as found for this keyword in this line
                    } else {
                        // This case should ideally not be reached if seg_text.contains(&h.text) is true
                        next_segments.push((seg_text, None));
                    }
                } else {
                    next_segments.push((seg_text, None));
                }
            }
            segments = next_segments;
        }
    }

    segments
}

#[component]
fn ConsoleHeader(autoscroll: bool, count: usize) -> Element {
    rsx! {
        div { class: "shrink-0 h-6 bg-[#16181a] border-b border-[#222629] flex items-center justify-between px-3",
            div { class: "flex items-center gap-4",
                div { class: "flex gap-1.5",
                    div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                    div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                    div { class: "w-2 h-2 rounded-full bg-[#394f56]" }
                }
                span { class: "text-[10px] text-gray-500 font-mono", "[ LINES: {count} / OPFS ENABLED ]" }
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
