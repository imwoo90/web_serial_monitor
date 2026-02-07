use crate::state::Highlight;
use regex::Regex;

/// Processes log text to remove timestamps and split into highlight segments including ANSI colors
pub fn decode_ansi_text(
    text: &str,
    highlights: &[Highlight],
    show_highlights: bool,
) -> Vec<(String, Option<String>)> {
    let content = text;

    // 2. ANSI Code Parsing
    // We treat ANSI codes as the base segmentation, then apply user highlights on top.
    let mut segments = Vec::new();

    // Using thread_local for Regex to avoid recompilation
    thread_local! {
        // Matches CSI sequences: ESC [ params command
        static ANSI_RE: Regex = Regex::new(r"\x1B\[([0-9;]*)([A-Za-z])").unwrap();
    }

    let mut last_pos = 0;
    let mut current_color: Option<String> = None;

    ANSI_RE.with(|re| {
        for cap in re.captures_iter(content) {
            let m = if let Some(m) = cap.get(0) {
                m
            } else {
                continue;
            };
            let start = m.start();
            let end = m.end();

            // Push text before the code
            if start > last_pos {
                segments.push((content[last_pos..start].to_string(), current_color.clone()));
            }

            // Command Processing
            if let Some(cmd_match) = cap.get(2) {
                let params = cap.get(1).map_or("", |m| m.as_str());
                let cmd = cmd_match.as_str();

                match cmd {
                    "m" => {
                        // SGR - Select Graphic Rendition (Colors)
                        if params.is_empty() {
                            // \x1B[m is equivalent to \x1B[0m (Reset)
                            current_color = None;
                        } else {
                            for code in params.split(';') {
                                match code {
                                    "0" => current_color = None,
                                    // Standard Foreground Colors
                                    "30" | "90" => current_color = Some("#9ca3af".to_string()), // Gray-400
                                    "31" | "91" => current_color = Some("#ef4444".to_string()), // Red-500
                                    "32" | "92" => current_color = Some("#10b981".to_string()), // Emerald-500
                                    "33" | "93" => current_color = Some("#f59e0b".to_string()), // Amber-500
                                    "34" | "94" => current_color = Some("#3b82f6".to_string()), // Blue-500
                                    "35" | "95" => current_color = Some("#d946ef".to_string()), // Fuchsia-500
                                    "36" | "96" => current_color = Some("#06b6d4".to_string()), // Cyan-500
                                    "37" | "97" => current_color = Some("#f3f4f6".to_string()), // Gray-100
                                    // Note: RGB/256 colors structure (38;2;... or 38;5;...) is partially split here.
                                    // Since this logic processes code-by-code splitting by ';', it's imperfect for multi-param codes.
                                    // However, for standard logs, it works. True robust parsing needs a stateful iterator.
                                    _ => {}
                                }
                            }
                        }
                    }
                    "C" => {
                        // CUF - Cursor Forward (Spaces)
                        // \x1B[nC moves right n times. Default 1.
                        let count = params.parse::<usize>().unwrap_or(1);
                        let spaces = " ".repeat(count);
                        // We push spaces using current color (relevant if background color logic existed)
                        segments.push((spaces, current_color.clone()));
                    }
                    "K" => {
                        // EL - Erase in Line
                        // usually \x1B[K or \x1B[0K (clear to end).
                        // HTML renderer doesn't need to explicitly clear "void".
                    }
                    _ => {
                        // Unknown command, ignore.
                    }
                }
            }

            last_pos = end;
        }
    });

    // Push remaining text
    if last_pos < content.len() {
        segments.push((content[last_pos..].to_string(), current_color));
    } else if segments.is_empty() {
        // If empty content or fully consumed by codes (unlikely to result in empty segment list if logic is right, but safe guard)
        // Actually if content was just "\x1B[32m", we have last_pos == len, segments empty? No, last_pos would be len.
        // Wait, if loop runs once, segments might be updated if start > last_pos.
        // If content is just ANSI code, start=0, last_pos becomes len. loops end.
        // Pushes?
        // No, start > last_pos (0 > 0) is false.
        // Loop finishes. last_pos == len.
        // Checked: if last_pos < content.len() -> False.
        // Result: Empty segments.
        // But we probably want at least one empty segment if input was non-empty logic-wise?
        // Actually empty list is fine, log line will just render nothing.
    }

    // Fallback if no ANSI codes were found, we treat the whole thing as one segment
    if segments.is_empty() && !content.is_empty() {
        segments.push((content.to_string(), None));
    }
    // If original content was empty, segments is empty, which is correct.

    // 3. User Highlighting Overlay
    // We process existing segments and split them further if keywords match
    if show_highlights {
        for h in highlights {
            if h.text.is_empty() {
                continue;
            }

            let mut next_segments = Vec::new();

            for (seg_text, color) in segments {
                // We allow user highlights to override ANSI colors?
                // Or only if ANSI color is None?
                // Typically User Highlight is "Search" or "Important", so it should override.

                // However, splitting logic relies on text.
                if seg_text.contains(&h.text) {
                    // Recover delimiters

                    // Simple split_once logic was used before:
                    // Only highlighted the FIRST occurrence per segment?
                    // The previous code:
                    // if let Some((prefix, suffix)) = seg_text.split_once(&h.text)

                    // Let's stick to split_once for simplicity and stability as per previous implementation
                    if let Some((prefix, suffix)) = seg_text.split_once(&h.text) {
                        if !prefix.is_empty() {
                            next_segments.push((prefix.to_string(), color.clone()));
                        }

                        // The Highlighted Part (Force User Color)
                        next_segments.push((h.text.clone(), Some(h.color.to_string())));

                        if !suffix.is_empty() {
                            next_segments.push((suffix.to_string(), color.clone()));
                            // Keep original background/ANSI color for suffix
                        }
                    } else {
                        // Should not happen if contains is true
                        next_segments.push((seg_text, color));
                    }
                } else {
                    next_segments.push((seg_text, color));
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
    fn test_ansi_parsing() {
        let highlights = vec![];

        // Green text
        let res = decode_ansi_text("\x1B[32mHello\x1B[0m", &highlights, false);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].0, "Hello");
        assert_eq!(res[0].1.as_deref(), Some("#10b981"));

        // Mixed
        let res = decode_ansi_text("A\x1B[31mB\x1B[0mC", &highlights, false);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].0, "A");
        assert_eq!(res[0].1, None);
        assert_eq!(res[1].0, "B");
        assert_eq!(res[1].1.as_deref(), Some("#ef4444"));
        assert_eq!(res[2].0, "C");
        assert_eq!(res[2].1, None);
    }

    #[test]
    fn test_highlight_overlay() {
        let highlights = vec![Highlight {
            id: 1,
            text: "Error".to_string(),
            color: "blue",
        }];

        // ANSI Green text containing "Error"
        let res = decode_ansi_text("\x1B[32mNoErrorHere\x1B[0m", &highlights, true);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].0, "No");
        assert_eq!(res[0].1.as_deref(), Some("#10b981")); // Green
        assert_eq!(res[1].0, "Error");
        assert_eq!(res[1].1.as_deref(), Some("blue")); // User Blue wins
        assert_eq!(res[2].0, "Here");
        assert_eq!(res[2].1.as_deref(), Some("#10b981")); // Green
    }
}
