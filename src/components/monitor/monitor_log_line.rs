use crate::config::line_height_from_font;
use crate::state::{AppState, Highlight};
use crate::utils::decode_ansi_text;
use dioxus::prelude::*;

#[component]
pub fn MonitorLogLine(text: String, highlights: Vec<Highlight>, show_highlights: bool) -> Element {
    let state = use_context::<AppState>();
    let font_size = *state.ui.font_size.read();
    let line_height = line_height_from_font(font_size);
    let segments = decode_ansi_text(&text, &highlights, show_highlights);

    rsx! {
        div {
            style: "height: {line_height}px; line-height: {line_height}px;",
            class: "text-gray-300 whitespace-pre font-mono",
            style: "font-size: {font_size}px;",
            for (content , color) in segments {
                if let Some(c) = color {
                    span { class: "font-bold", style: "color: {c};", "{content}" }
                } else {
                    "{content}"
                }
            }
        }
    }
}
