use regex::Regex;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Clone)]
pub struct ActiveFilter {
    pub query: String,
    pub match_case: bool,
    pub regex: Option<Regex>,
    pub invert: bool,
}

impl ActiveFilter {
    pub fn matches(&self, text: &str) -> bool {
        let matched = if let Some(re) = &self.regex {
            re.is_match(text)
        } else if self.match_case {
            text.contains(&self.query)
        } else {
            text.to_lowercase().contains(&self.query.to_lowercase())
        };
        if self.invert {
            !matched
        } else {
            matched
        }
    }
}

pub struct LogIndex {
    pub line_offsets: Vec<u64>,
    pub line_count: usize,
    pub filtered_lines: Vec<LineRange>,
    pub is_filtering: bool,
    pub active_filter: Option<ActiveFilter>,
}

impl LogIndex {
    pub fn new() -> Self {
        Self {
            line_offsets: vec![0],
            line_count: 0,
            filtered_lines: Vec::new(),
            is_filtering: false,
            active_filter: None,
        }
    }

    pub fn reset_base(&mut self) {
        self.line_offsets = vec![0];
        self.line_count = 0;
        self.filtered_lines.clear();
    }

    pub fn push_line(&mut self, absolute_end_offset: u64) {
        self.line_offsets.push(absolute_end_offset);
        self.line_count += 1;
    }

    pub fn push_filtered(&mut self, range: LineRange) {
        self.filtered_lines.push(range);
    }

    pub fn get_total_count(&self) -> usize {
        if self.is_filtering {
            self.filtered_lines.len()
        } else {
            self.line_count
        }
    }

    pub fn get_line_range(&self, index: usize) -> Option<LineRange> {
        if self.is_filtering {
            self.filtered_lines.get(index).cloned()
        } else {
            if index < self.line_count {
                Some(LineRange {
                    start: self.line_offsets[index],
                    end: self.line_offsets[index + 1],
                })
            } else {
                None
            }
        }
    }

    pub fn clear_filter(&mut self) {
        self.is_filtering = false;
        self.active_filter = None;
        self.filtered_lines.clear();
    }
}
