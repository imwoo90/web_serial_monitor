use regex::Regex;
use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct ByteOffset(pub u64);

impl Add<u64> for ByteOffset {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        ByteOffset(self.0 + rhs)
    }
}

impl Sub<ByteOffset> for ByteOffset {
    type Output = u64;
    fn sub(self, rhs: ByteOffset) -> u64 {
        self.0 - rhs.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct LineIndex(pub usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineRange {
    pub start: ByteOffset,
    pub end: ByteOffset,
}

#[derive(Clone)]
pub struct ActiveFilter {
    pub query: String,
    pub query_lower: String,
    pub match_case: bool,
    pub regex: Option<Regex>,
    pub invert: bool,
}

pub struct ActiveFilterBuilder {
    query: String,
    match_case: bool,
    use_regex: bool,
    invert: bool,
}

impl ActiveFilterBuilder {
    pub fn new(query: String) -> Self {
        Self {
            query,
            match_case: true,
            use_regex: false,
            invert: false,
        }
    }

    pub fn case_sensitive(mut self, yes: bool) -> Self {
        self.match_case = yes;
        self
    }

    pub fn regex(mut self, yes: bool) -> Self {
        self.use_regex = yes;
        self
    }

    pub fn invert(mut self, yes: bool) -> Self {
        self.invert = yes;
        self
    }

    pub fn build(self) -> Result<ActiveFilter, String> {
        let regex = if self.use_regex {
            Some(
                regex::RegexBuilder::new(&self.query)
                    .case_insensitive(!self.match_case)
                    .build()
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };

        let query_lower = if !self.match_case {
            self.query.to_lowercase()
        } else {
            String::new()
        };

        Ok(ActiveFilter {
            query: self.query,
            query_lower,
            match_case: self.match_case,
            regex,
            invert: self.invert,
        })
    }
}

impl ActiveFilter {
    pub fn matches(&self, text: &str) -> bool {
        let matched = if let Some(re) = &self.regex {
            re.is_match(text)
        } else if self.match_case {
            text.contains(&self.query)
        } else {
            // Optimization: Use pre-calculated lowercased query
            // optimization: In the future, we could avoid text.to_lowercase() allocation
            // by using a case-insensitive search iterator, but for now strict structure fix.
            text.to_lowercase().contains(&self.query_lower)
        };
        if self.invert {
            !matched
        } else {
            matched
        }
    }
}

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
