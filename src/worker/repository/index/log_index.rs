use crate::worker::repository::index::filter::ActiveFilter;
use crate::worker::repository::index::types::{ByteOffset, LineIndex, LineRange};

/// Log index that tracks line offsets and filtering state
pub struct LogIndex {
    pub line_offsets: Vec<ByteOffset>,
    pub line_count: usize,
    pub filtered_lines: Vec<LineRange>,
    pub is_filtering: bool,
    pub active_filter: Option<ActiveFilter>,
}

impl LogIndex {
    pub fn new() -> Self {
        Self {
            line_offsets: vec![ByteOffset(0)],
            line_count: 0,
            filtered_lines: Vec::new(),
            is_filtering: false,
            active_filter: None,
        }
    }

    pub fn reset_base(&mut self) {
        self.line_offsets = vec![ByteOffset(0)];
        self.line_count = 0;
        self.filtered_lines.clear();
    }

    pub fn push_line(&mut self, absolute_end_offset: ByteOffset) {
        self.line_offsets.push(absolute_end_offset);
        self.line_count += 1;
    }

    pub fn push_filtered(&mut self, range: LineRange) {
        self.filtered_lines.push(range);
    }

    pub fn prepend_filtered(&mut self, ranges: Vec<LineRange>) {
        if self.filtered_lines.is_empty() {
            self.filtered_lines = ranges;
        } else {
            self.filtered_lines.splice(0..0, ranges);
        }
    }

    pub fn get_total_count(&self) -> usize {
        if self.is_filtering {
            self.filtered_lines.len()
        } else {
            self.line_count
        }
    }

    pub fn get_line_range(&self, index: LineIndex) -> Option<LineRange> {
        if self.is_filtering {
            self.filtered_lines.get(index.0).cloned()
        } else if index.0 < self.line_count {
            Some(LineRange {
                start: self.line_offsets[index.0],
                end: self.line_offsets[index.0 + 1],
            })
        } else {
            None
        }
    }

    pub fn clear_filter(&mut self) {
        self.is_filtering = false;
        self.active_filter = None;
        self.filtered_lines.clear();
    }
}

impl Default for LogIndex {
    fn default() -> Self {
        Self::new()
    }
}
