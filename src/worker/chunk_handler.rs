use crate::config::MAX_LINE_BYTES;
use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::repository::index::{ByteOffset, LineRange};
use std::borrow::Cow;
use vt100::Parser;

/// Handles streaming line processing with leftover buffer management
pub struct StreamingLineProcessor {
    pub leftover_buffer: String,
    parser: Parser,
}

impl StreamingLineProcessor {
    pub fn new() -> Self {
        Self {
            leftover_buffer: String::new(),
            // Height 1 ensures we focus on a single line.
            // Width MAX_LINE_BYTES prevents arbitrary wrapping of long lines.
            // Scrollback 0 disables history as we extract confirmed lines immediately.
            parser: Parser::new(1, MAX_LINE_BYTES as u16, 0),
        }
    }

    pub fn process_vt100(
        &mut self,
        chunk: &[u8],
        formatter: &dyn LogFormatterStrategy,
        timestamp: &str,
        is_filtering: bool,
        filter_matcher: impl Fn(&str) -> bool,
    ) -> (String, Vec<ByteOffset>, Vec<LineRange>, Option<String>) {
        let mut batch = String::new();
        let mut offsets = Vec::new();
        let mut filtered = Vec::new();
        let mut relative_offset = ByteOffset(0);

        let mut start = 0;
        let len = chunk.len();

        while start < len {
            if let Some((end, next_start)) = Self::find_next_line_ending(chunk, start) {
                // Process content up to the newline char(s)
                let line_bytes = &chunk[start..end];
                self.parser.process(line_bytes);

                // Extract the formatted line immediately
                if let Some(bytes) = self
                    .parser
                    .screen()
                    .rows_formatted(0, MAX_LINE_BYTES as u16)
                    .next()
                {
                    let line_str = String::from_utf8_lossy(&bytes);

                    self.process_single_line(
                        &line_str,
                        formatter,
                        timestamp,
                        &mut batch,
                        &mut offsets,
                        &mut filtered,
                        &mut relative_offset,
                        is_filtering,
                        &filter_matcher,
                    );
                }

                // Clear the line in the parser to prepare for the next line
                // Carriage Return + Clear Line
                self.parser.process(b"\r\x1b[2K");

                start = next_start;
            } else {
                // No more newlines, the rest is active line content
                break;
            }
        }

        // Process any remaining bytes (incomplete line)
        if start < chunk.len() {
            self.parser.process(&chunk[start..]);
        }

        // Get Current Active Line (Row 0)
        // If the chunk ended with a newline, this will be empty (which is correct)
        let active_line = self
            .parser
            .screen()
            .rows_formatted(0, MAX_LINE_BYTES as u16)
            .next()
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .filter(|s| !s.trim().is_empty())
            .filter(|s| !is_filtering || filter_matcher(s));

        (batch, offsets, filtered, active_line)
    }

    /// Processes a hex chunk (Hex mode)
    pub fn process_hex_lines(
        &mut self,
        chunk: &str,
        formatter: &dyn LogFormatterStrategy,
        timestamp: &str,
        is_filtering: bool,
        filter_matcher: impl Fn(&str) -> bool,
    ) -> (String, Vec<ByteOffset>, Vec<LineRange>) {
        let max_len = formatter.max_line_length();

        // 1. If leftover is already too long, force a split before even adding new chunk
        if !self.leftover_buffer.is_empty() && self.leftover_buffer.len() >= max_len {
            self.leftover_buffer.push('\n');
        }

        let full_text = if self.leftover_buffer.is_empty() {
            Cow::Borrowed(chunk)
        } else {
            Cow::Owned(format!("{}{}", self.leftover_buffer, chunk))
        };

        let mut batch = String::with_capacity(full_text.len() * 2);
        let mut offsets = Vec::new(); // Capacity logic changed slightly, let vector handle realloc or use heuristic
        let mut filtered = Vec::new();
        let mut relative_offset = ByteOffset(0);

        let text_bytes = full_text.as_bytes();
        let len = text_bytes.len();
        let mut start = 0;

        while start < len {
            if let Some((end, next_start)) = Self::find_next_line_ending(text_bytes, start) {
                let line_str = &full_text[start..end];

                self.process_single_line(
                    line_str,
                    formatter,
                    timestamp,
                    &mut batch,
                    &mut offsets,
                    &mut filtered,
                    &mut relative_offset,
                    is_filtering,
                    &filter_matcher,
                );

                start = next_start;
            } else {
                // No more newlines, save the rest as leftover
                self.leftover_buffer = full_text[start..].to_string();
                return (batch, offsets, filtered);
            }
        }

        // If loop finished exactly (ended with newline), clear leftover
        self.leftover_buffer.clear();
        (batch, offsets, filtered)
    }

    #[allow(clippy::too_many_arguments)]
    fn process_single_line(
        &self,
        line: &str,
        formatter: &dyn LogFormatterStrategy,
        timestamp: &str,
        batch: &mut String,
        offsets: &mut Vec<ByteOffset>,
        filtered: &mut Vec<LineRange>,
        current_relative_offset: &mut ByteOffset,
        is_filtering: bool,
        filter_matcher: &impl Fn(&str) -> bool,
    ) {
        let max_len = formatter.max_line_length();
        let mut start = 0;

        // Handle empty line case
        if line.is_empty() {
            let start_pos = batch.len();
            let formatted = formatter.format("", timestamp);
            batch.push_str(&formatted);
            let line_len = (batch.len() - start_pos) as u64;

            if is_filtering && filter_matcher(&batch[start_pos..]) {
                filtered.push(LineRange {
                    start: *current_relative_offset,
                    end: *current_relative_offset + line_len,
                });
            }
            *current_relative_offset = *current_relative_offset + line_len;
            offsets.push(*current_relative_offset);
            return;
        }

        while start < line.len() {
            let mut end = (start + max_len).min(line.len());
            while !line.is_char_boundary(end) {
                end -= 1;
            }
            let sub_line = &line[start..end];

            let start_pos = batch.len();
            let formatted = formatter.format(sub_line, timestamp);
            batch.push_str(&formatted);
            let line_len = (batch.len() - start_pos) as u64;

            if is_filtering && filter_matcher(&batch[start_pos..]) {
                filtered.push(LineRange {
                    start: *current_relative_offset,
                    end: *current_relative_offset + line_len,
                });
            }

            *current_relative_offset = *current_relative_offset + line_len;
            offsets.push(*current_relative_offset);
            start = end;
        }
    }

    pub fn clear(&mut self) {
        self.leftover_buffer.clear();
        // Reset parser state
        self.parser = Parser::new(1, MAX_LINE_BYTES as u16, 0);
    }

    /// Helper to find the next line ending from a byte slice.
    /// Returns Some((content_end_index, next_start_index)) if found.
    /// content_end_index: Index exclusive of the newline char(s).
    /// next_start_index: Index to resume searching for the next line (skipping \n, \r, or \r\n).
    fn find_next_line_ending(chunk: &[u8], start: usize) -> Option<(usize, usize)> {
        let len = chunk.len();
        let mut i = start;
        while i < len {
            let b = chunk[i];
            if b == b'\n' {
                return Some((i, i + 1));
            } else if b == b'\r' {
                if i + 1 < len {
                    if chunk[i + 1] == b'\n' {
                        return Some((i, i + 2)); // CRLF
                    } else {
                        return Some((i, i + 1)); // CR followed by something else
                    }
                } else {
                    // CR at the very end of chunk.
                    // We don't know if next char is LF. Return None to buffer it.
                    return None;
                }
            }
            i += 1;
        }
        None
    }
}

impl Default for StreamingLineProcessor {
    fn default() -> Self {
        Self::new()
    }
}
