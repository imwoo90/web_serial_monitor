use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::repository::index::{ByteOffset, LineRange};
use std::borrow::Cow;
use vt100::Parser;

/// Handles streaming line processing with leftover buffer management
pub struct StreamingLineProcessor {
    pub leftover_buffer: String,
    parser: Parser,
    last_scrollback_index: usize,
}

impl StreamingLineProcessor {
    pub fn new() -> Self {
        Self {
            leftover_buffer: String::new(),
            // Height 1 ensures lines are pushed to scrollback immediately upon newline.
            // Width 2048 prevents arbitrary wrapping of long lines.
            parser: Parser::new(1, 2048, 100000),
            last_scrollback_index: 0,
        }
    }

    /// Processes bytes using VT100 parser and extracts clean lines with ANSI codes
    pub fn process_vt100(
        &mut self,
        chunk: &[u8],
        formatter: &dyn LogFormatterStrategy,
        timestamp: &str,
        is_filtering: bool,
        filter_matcher: impl Fn(&str) -> bool,
    ) -> (String, Vec<ByteOffset>, Vec<LineRange>) {
        self.parser.process(chunk);

        let mut batch = String::new();
        let mut offsets = Vec::new();
        let mut filtered = Vec::new();
        let mut relative_offset = ByteOffset(0);

        // Get total history lines (hack using set_scrollback(MAX))
        self.parser.screen_mut().set_scrollback(usize::MAX);
        let len = self.parser.screen().scrollback();

        // Handle scrollback clearing external/internal reset
        if len < self.last_scrollback_index {
            self.last_scrollback_index = 0;
        }

        if len > self.last_scrollback_index {
            // Iterate through new lines in history
            for i in self.last_scrollback_index..len {
                // Offset calculation: len - i gives the 'lines back from end' + 1 roughly
                // Logic:
                // i=0 (oldest), offset = len (view from start)
                // i=len-1 (newest), offset = 1 (view from end)
                self.parser.screen_mut().set_scrollback(len - i);

                // Get the first row of the view
                if let Some(bytes) = self.parser.screen().rows_formatted(0, 2048).next() {
                    let line_str = String::from_utf8_lossy(&bytes);

                    // Use common logic to format/filter
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
            }
            self.last_scrollback_index = len;
        }

        // Reset view to normal (live)
        self.parser.screen_mut().set_scrollback(0);

        // Memory Management Hack: Clear scrollback if too large
        if self.last_scrollback_index > 50000 {
            // Attempt to clear scrollback
            self.parser.process(b"\x1b[3J");

            // Re-check length
            self.parser.screen_mut().set_scrollback(usize::MAX);
            if self.parser.screen().scrollback() == 0 {
                self.last_scrollback_index = 0;
            }
            self.parser.screen_mut().set_scrollback(0);
        }

        (batch, offsets, filtered)
    }

    /// Processes a text chunk (Legacy/Hex mode)
    pub fn process_text_lines(
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

        let mut raw_lines: Vec<&str> = self.split_by_line_ending(&full_text, formatter);

        // The last part is the new leftover
        self.leftover_buffer = raw_lines.pop().unwrap_or("").to_string();

        let mut batch = String::with_capacity(full_text.len() * 2);
        let mut offsets = Vec::with_capacity(raw_lines.len());
        let mut filtered = Vec::new();
        let mut relative_offset = ByteOffset(0);

        for line in raw_lines {
            // Legacy cleaning (removes \r etc)
            let cleaned = formatter.clean_line_ending(line);
            self.process_single_line(
                cleaned,
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

        (batch, offsets, filtered)
    }

    fn split_by_line_ending<'a>(
        &self,
        text: &'a str,
        formatter: &dyn LogFormatterStrategy,
    ) -> Vec<&'a str> {
        use crate::state::LineEnding;

        match formatter.get_line_ending() {
            LineEnding::NL => text.split('\n').collect(),
            LineEnding::CR => text.split('\r').collect(),
            LineEnding::NLCR => text.split("\r\n").collect(),
            LineEnding::None => vec![text],
        }
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
        // Reset parser? Recreating is safer.
        self.parser = Parser::new(1, 2048, 100000);
        self.last_scrollback_index = 0;
    }
}

impl Default for StreamingLineProcessor {
    fn default() -> Self {
        Self::new()
    }
}
