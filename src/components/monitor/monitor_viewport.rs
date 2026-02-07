use crate::components::monitor::monitor_log_line::MonitorLogLine;
use crate::config::{CONSOLE_BOTTOM_PADDING, CONSOLE_TOP_PADDING};
use crate::state::{AppState, LineEnding};
use crate::utils::serial;
use dioxus::prelude::*;
use js_sys::Uint8Array;

#[component]
pub fn MonitorViewport(
    total_height: f64,
    offset_top: f64,
    onmounted_console: EventHandler<MountedEvent>,
    onscroll: EventHandler<ScrollEvent>,
    onmounted_sentinel: EventHandler<MountedEvent>,
) -> Element {
    let state = use_context::<AppState>();
    let bridge = crate::hooks::use_worker_controller();
    let visible_logs = state.log.visible_logs;
    let total_lines = state.log.total_lines;

    rsx! {
        div {
            class: "flex-1 overflow-y-auto font-mono text-xs md:text-sm leading-[20px] scrollbar-custom relative",
            style: "overflow-anchor: none;",
            id: "console-output",
            tabindex: "0",
            onkeydown: move |evt| {
                let modifiers = evt.modifiers();
                if modifiers.contains(Modifiers::CONTROL) || modifiers.contains(Modifiers::ALT)

                    || modifiers.contains(Modifiers::META)
                {
                    return;
                }
                let key = evt.key();
                let data = match key {
                    Key::Character(c) => c.into_bytes(),
                    Key::Enter => {
                        match *state.serial.tx_line_ending.peek() {
                            LineEnding::NL => vec![b'\n'],
                            LineEnding::CR => vec![b'\r'],
                            LineEnding::NLCR => vec![b'\r', b'\n'],
                            LineEnding::None => vec![b'\r'],
                        }
                    }
                    Key::Backspace => vec![0x08],
                    Key::Tab => vec![0x09],
                    Key::Escape => vec![0x1B],
                    _ => return,
                };
                let port = state.conn.port.peek().as_ref().cloned();
                let local_echo = *state.serial.tx_local_echo.peek();
                let bridge = bridge.clone();
                spawn(async move {
                    if let Some(p) = port {
                        if serial::send_data(&p, &data).await.is_ok() {
                            if local_echo {
                                let array = Uint8Array::from(data.as_slice());
                                bridge.append_chunk(array, false);
                            }
                        }
                    }
                });
            },
            onmounted: move |evt| onmounted_console.call(evt),
            onscroll: move |evt| onscroll.call(evt),

            // Virtual Scroll Spacer & Content
            div { style: "height: {total_height}px; width: 100%; position: absolute; top: 0; left: 0; pointer-events: none;" }
            div { style: "position: absolute; top: 0; left: 0; right: 0; transform: translateY({offset_top}px); padding: {CONSOLE_TOP_PADDING}px 0 {CONSOLE_BOTTOM_PADDING}px 0; pointer-events: auto; min-width: 100%; width: max-content;",
                {
                    let highlights = (state.log.highlights)().clone();
                    let show_highlights = (state.ui.show_highlights)();
                    let active_line = (state.log.active_line)();
                    let logs = visible_logs.read();
                    let is_at_bottom = logs

                        .last()
                        .map(|(idx, _)| *idx + 1 == total_lines())
                        .unwrap_or(total_lines() == 0);
                    rsx! {
                        for (line_idx , text) in logs.iter() {
                            MonitorLogLine {
                                key: "{line_idx}",
                                text: text.clone(),
                                highlights: highlights.clone(),
                                show_highlights,
                            }
                        }
                        if is_at_bottom {
                            if let Some(text) = active_line {
                                MonitorLogLine {
                                    key: "{0}",
                                    text: text.clone(),
                                    highlights: highlights.clone(),
                                    show_highlights: false, // Maybe don't highlight active line to avoid flicker? // Maybe don't highlight active line to avoid flicker?
                                }
                            }
                        }
                    }
                }
            }

            // Loading & Sentinel
            if visible_logs.read().is_empty() && total_lines() > 0 {
                div { class: "text-gray-500 animate-pulse text-[12px] px-4", "Loading buffer..." }
            }
            div {
                style: "position: absolute; top: {total_height}px; height: 1px; width: 100%; pointer-events: none;",
                onmounted: move |evt| onmounted_sentinel.call(evt),
            }
        }
    }
}
