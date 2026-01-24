use crate::state::LineEnding;
use chrono::Timelike;
use regex::Regex;
use std::borrow::Cow;
use std::fmt::Write;

/// Represents a byte range within the log file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineRange {
    pub start: u64,
    pub end: u64,
}

/// Active filtering configuration
#[derive(Clone)]
pub struct ActiveFilter {
    pub query: String,
    pub lower_query: String,
    pub match_case: bool,
    pub regex: Option<Regex>,
    pub invert: bool,
}

impl ActiveFilter {
    pub fn matches(&self, text: &str) -> bool {
        let mut matched = if let Some(re) = &self.regex {
            re.is_match(text)
        } else if self.match_case {
            text.contains(&self.query)
        } else {
            text.to_lowercase().contains(&self.lower_query)
        };

        if self.invert {
            matched = !matched;
        }
        matched
    }
}

/// Core processing logic separated from WASM/IO for testability
pub struct CoreProcessor {
    pub line_offsets: Vec<u64>,
    pub line_count: usize,
    pub filtered_lines: Vec<LineRange>,
    pub is_filtering: bool,
    pub active_filter: Option<ActiveFilter>,
    pub leftover_chunk: String,
}

impl CoreProcessor {
    pub fn new() -> Self {
        Self {
            line_offsets: vec![0],
            line_count: 0,
            filtered_lines: Vec::new(),
            is_filtering: false,
            active_filter: None,
            leftover_chunk: String::new(),
        }
    }

    /// Processes decoded text and prepares formatted lines with timestamps.
    /// Returns (Formatted Buffer String, Tuple of (LineOffsetsToAdd, FilteredLinesToAdd))
    pub fn prepare_batch(
        &mut self,
        chunk_text: &str,
        line_ending_mode: LineEnding,
    ) -> (String, Vec<u64>, Vec<LineRange>) {
        if !self.leftover_chunk.is_empty() && self.leftover_chunk.len() > 4 * 1024 {
            // Safety: If buffer grows too large without newline, force a flush (e.g. 4KB)
            self.leftover_chunk.push('\n');
        }

        let full_text = if self.leftover_chunk.is_empty() {
            Cow::Borrowed(chunk_text)
        } else {
            Cow::Owned(format!("{}{}", self.leftover_chunk, chunk_text))
        };

        let lines_iter = match line_ending_mode {
            LineEnding::None | LineEnding::NL => full_text.split("\n"),
            LineEnding::CR => full_text.split("\r"),
            LineEnding::NLCR => full_text.split("\r\n"),
        };
        let mut lines_iter = lines_iter.peekable();

        let estimated_line_overhead = 24;
        let mut batch_buffer = String::with_capacity(
            chunk_text.len() + chunk_text.len() / 20 * estimated_line_overhead,
        );
        let mut new_offsets = Vec::new();
        let mut new_filtered = Vec::new();

        let mut relative_pos = 0u64;

        while let Some(line) = lines_iter.next() {
            if lines_iter.peek().is_none() {
                self.leftover_chunk = line.to_string();
                break;
            }

            let mut clean_line = line;
            if line_ending_mode == LineEnding::NL && clean_line.ends_with('\r') {
                clean_line = &clean_line[..clean_line.len() - 1];
            }
            if line_ending_mode == LineEnding::CR && clean_line.starts_with('\n') {
                clean_line = &clean_line[1..];
            }

            let start_len = batch_buffer.len();
            let now = chrono::Utc::now();
            let _ = write!(
                batch_buffer,
                "[{:02}:{:02}:{:02}.{:03}] ",
                now.hour(),
                now.minute(),
                now.second(),
                now.timestamp_subsec_millis()
            );

            batch_buffer.push_str(clean_line);
            batch_buffer.push('\n');

            let added_len = (batch_buffer.len() - start_len) as u64;

            if self.is_filtering {
                let final_line = &batch_buffer[start_len..];
                if let Some(filter) = &self.active_filter {
                    if filter.matches(final_line) {
                        new_filtered.push(LineRange {
                            start: relative_pos,
                            end: relative_pos + added_len,
                        });
                    }
                }
            }

            relative_pos += added_len;
            new_offsets.push(relative_pos);
        }

        (batch_buffer, new_offsets, new_filtered)
    }

    pub fn get_total_lines(&self) -> usize {
        if self.is_filtering {
            self.filtered_lines.len()
        } else {
            self.line_count
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::LineEnding;

    #[test]
    fn test_active_filter_matches() {
        let filter = ActiveFilter {
            query: "ERROR".into(),
            lower_query: "error".into(),
            match_case: true,
            regex: None,
            invert: false,
        };
        assert!(filter.matches("System ERROR occurred"));
        assert!(!filter.matches("system error occurred"));

        let filter_nocase = ActiveFilter {
            query: "ERROR".into(),
            lower_query: "error".into(),
            match_case: false,
            regex: None,
            invert: false,
        };
        assert!(filter_nocase.matches("system error occurred"));
    }

    #[test]
    fn test_prepare_batch_splitting() {
        let mut core = CoreProcessor::new();
        let (batch, offsets, _) = core.prepare_batch("Hello\nWorld\nIncompl", LineEnding::NL);

        // Lines should be timestamped.
        // We can't predict exact timestamp but check format.
        let lines: Vec<&str> = batch.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("Hello"));
        assert!(lines[1].contains("World"));
        assert_eq!(core.leftover_chunk, "Incompl");
        assert_eq!(offsets.len(), 2);
    }

    #[test]
    fn test_filter_integration() {
        let mut core = CoreProcessor::new();
        core.is_filtering = true;
        core.active_filter = Some(ActiveFilter {
            query: "Critical".into(),
            lower_query: "critical".into(),
            match_case: true,
            regex: None,
            invert: false,
        });

        let (batch, _, filtered) =
            core.prepare_batch("Info: log\nCritical: error\n", LineEnding::NL);

        assert_eq!(filtered.len(), 1);
        // The second line matches
        let lines: Vec<&str> = batch.lines().collect();
        assert!(lines[1].contains("Critical"));
    }
}
