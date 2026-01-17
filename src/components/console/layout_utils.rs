use super::types::HEADER_OFFSET;
use dioxus::prelude::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// 윈도우 리사이즈 이벤트를 처리하여 콘솔 높이를 조정하는 훅
pub fn use_window_resize(
    mut console_height: Signal<f64>,
    autoscroll: Signal<bool>,
    sentinel: Signal<Option<Rc<MountedData>>>,
) {
    use_effect(move || {
        let mut update = move || {
            if let Ok(Some(h)) = web_sys::window()
                .unwrap()
                .inner_height()
                .map(|jv| jv.as_f64())
            {
                console_height.set((h - HEADER_OFFSET).max(100.0));
                // 자동 스크롤 활성화 시, 리사이즈 중에도 바닥 고정 강제
                if (autoscroll)() {
                    if let Some(s) = sentinel.peek().as_ref() {
                        let _ = s.scroll_to(ScrollBehavior::Instant);
                    }
                }
            }
        };
        update(); // 초기 실행
        let onresize = Closure::wrap(Box::new(update) as Box<dyn FnMut()>);
        web_sys::window()
            .unwrap()
            .set_onresize(Some(onresize.as_ref().unchecked_ref()));
        onresize.forget();
    });
}

/// 자동 스크롤 기능을 관리하는 훅
pub fn use_auto_scroller(
    autoscroll: Signal<bool>,
    total_lines: Signal<usize>,
    sentinel: Signal<Option<Rc<MountedData>>>,
) {
    use_effect(move || {
        total_lines(); // 전체 라인 수 변화에 반응
        if (autoscroll)() {
            if let Some(h) = sentinel.peek().as_ref() {
                let _ = h.scroll_to(ScrollBehavior::Instant);
            }
        }
    });
}

#[component]
pub fn ConsoleHeader(autoscroll: bool, count: usize) -> Element {
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
