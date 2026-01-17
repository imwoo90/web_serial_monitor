use crate::state::Highlight;

/// 로그 텍스트를 처리하여 타임스탬프 제거 및 하이라이트 세그먼트로 분할
pub fn process_log_segments(
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
