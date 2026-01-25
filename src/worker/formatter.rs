use crate::state::LineEnding;
use chrono::Timelike;
use std::fmt::Write;

pub trait LogFormatterStrategy {
    fn format(&self, text: &str, timestamp: &str) -> String;
    fn format_chunk(&self, chunk: &[u8]) -> String;
    fn clean_line_ending<'a>(&self, line: &'a str) -> &'a str;
    fn max_line_length(&self) -> usize;
}

pub struct DefaultFormatter {
    pub line_ending: LineEnding,
    pub max_bytes: usize,
}

impl LogFormatterStrategy for DefaultFormatter {
    fn format(&self, text: &str, timestamp: &str) -> String {
        format!("{} {}\n", timestamp, text)
    }

    fn format_chunk(&self, _chunk: &[u8]) -> String {
        String::new()
    }

    fn clean_line_ending<'a>(&self, line: &'a str) -> &'a str {
        let mut clean = line;
        if self.line_ending == LineEnding::NL && clean.ends_with('\r') {
            clean = &clean[..clean.len() - 1];
        }
        if self.line_ending == LineEnding::CR && clean.starts_with('\n') {
            clean = &clean[1..];
        }
        clean
    }

    fn max_line_length(&self) -> usize {
        self.max_bytes
    }
}

pub struct HexFormatter {
    pub line_ending: LineEnding,
    pub max_bytes: usize,
}

impl LogFormatterStrategy for HexFormatter {
    fn format(&self, text: &str, timestamp: &str) -> String {
        if text.is_empty() {
            format!("{}\n", timestamp)
        } else {
            format!("{} {}\n", timestamp, text)
        }
    }

    fn format_chunk(&self, chunk: &[u8]) -> String {
        let mut acc = String::with_capacity(chunk.len() * 3);
        for &b in chunk {
            if b == b'\n' || b == b'\r' {
                acc.push(b as char);
            } else {
                let _ = write!(acc, "{:02X} ", b);
            }
        }
        acc
    }

    fn clean_line_ending<'a>(&self, line: &'a str) -> &'a str {
        let mut clean = line;
        if self.line_ending == LineEnding::NL && clean.ends_with('\r') {
            clean = &clean[..clean.len() - 1];
        }
        if self.line_ending == LineEnding::CR && clean.starts_with('\n') {
            clean = &clean[1..];
        }
        clean
    }

    fn max_line_length(&self) -> usize {
        self.max_bytes * 3 // 3 chars per byte ("XX ")
    }
}

pub struct LogFormatter {
    pub line_ending_mode: LineEnding,
}

impl LogFormatter {
    pub fn new(mode: LineEnding) -> Self {
        Self {
            line_ending_mode: mode,
        }
    }

    pub fn get_timestamp(&self) -> String {
        let now = chrono::Utc::now();
        format!(
            "[{:02}:{:02}:{:02}.{:03}]",
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_subsec_millis()
        )
    }
}
