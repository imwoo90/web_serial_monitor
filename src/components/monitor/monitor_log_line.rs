use crate::config::LINE_HEIGHT;
use crate::state::Highlight;
use crate::utils::decode_ansi_text;
use dioxus::prelude::*;

#[component]
pub fn MonitorLogLine(text: String, highlights: Vec<Highlight>, show_highlights: bool) -> Element {
    let segments = decode_ansi_text(&text, &highlights, show_highlights);

    rsx! {
        div {
            style: "height: {LINE_HEIGHT}px; line-height: {LINE_HEIGHT}px;",
            class: "text-gray-300 whitespace-pre text-[12px] font-mono",
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
