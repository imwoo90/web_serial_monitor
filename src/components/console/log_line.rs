use super::types::LINE_HEIGHT;
use crate::state::Highlight;
use crate::utils::process_log_segments;
use dioxus::prelude::*;

#[component]
pub fn LogLine(
    text: String,
    highlights: Vec<Highlight>,
    show_timestamps: bool,
    show_highlights: bool,
) -> Element {
    let segments = process_log_segments(&text, &highlights, show_timestamps, show_highlights);

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
