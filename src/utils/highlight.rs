use crate::state::Highlight;

/// Processes log text to remove timestamps and split into highlight segments
pub fn process_log_segments(
    text: &str,
    highlights: &[Highlight],
    show_timestamps: bool,
    show_highlights: bool,
) -> Vec<(String, Option<String>)> {
    // 1. Timestamp Parsing
    let content = if !show_timestamps && text.starts_with('[') {
        if let Some(end_pos) = text.find("] ") {
            &text[end_pos + 2..]
        } else {
            text
        }
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
            let mut found_for_keyword = false;

            for (seg_text, color) in segments {
                // Skip segments that are already colored
                if color.is_some() {
                    next_segments.push((seg_text, color));
                    continue;
                }

                // Search for keyword (currently processed only once per line using split_once)
                if !found_for_keyword && seg_text.contains(&h.text) {
                    if let Some((prefix, suffix)) = seg_text.split_once(&h.text) {
                        if !prefix.is_empty() {
                            next_segments.push((prefix.to_string(), None));
                        }

                        next_segments.push((h.text.clone(), Some(h.color.to_string())));

                        if !suffix.is_empty() {
                            next_segments.push((suffix.to_string(), None));
                        }
                        found_for_keyword = true;
                    } else {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Highlight;

    #[test]
    fn test_highlight_processing() {
        let highlights = vec![
            Highlight {
                id: 1,
                text: "Error".to_string(),
                color: "red",
            },
            Highlight {
                id: 2,
                text: "Warning".to_string(),
                color: "yellow",
            },
        ];

        // 1. No highlights
        let res = process_log_segments("Normal text", &highlights, true, false);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].0, "Normal text");
        assert!(res[0].1.is_none());

        // 2. Single match
        let res = process_log_segments("Critical Error found", &highlights, true, true);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].0, "Critical ");
        assert_eq!(res[1].0, "Error");
        assert_eq!(res[1].1.as_deref(), Some("red"));
        assert_eq!(res[2].0, " found");

        // 3. Timestamp removal (Variable format)
        let log = "[12:00:00.000] Message";
        let res = process_log_segments(log, &highlights, false, true);
        assert_eq!(res[0].0, "Message");
    }
}
