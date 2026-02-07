use chrono::Timelike;
use std::fmt::Write;

pub trait LogFormatterStrategy {
    fn format(&self, text: &str, timestamp: &str) -> String;
    fn format_chunk(&self, chunk: &[u8]) -> String;
    fn max_line_length(&self) -> usize;
}

pub struct DefaultFormatter {
    pub max_bytes: usize,
}

impl LogFormatterStrategy for DefaultFormatter {
    fn format(&self, text: &str, timestamp: &str) -> String {
        if timestamp.is_empty() {
            format!("{}\n", text)
        } else {
            format!("{} {}\n", timestamp, text)
        }
    }

    fn format_chunk(&self, _chunk: &[u8]) -> String {
        String::new()
    }

    fn max_line_length(&self) -> usize {
        self.max_bytes
    }
}

pub struct HexFormatter {
    pub max_bytes: usize,
}

impl LogFormatterStrategy for HexFormatter {
    fn format(&self, text: &str, timestamp: &str) -> String {
        if timestamp.is_empty() {
            format!("{}\n", text)
        } else if text.is_empty() {
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

    fn max_line_length(&self) -> usize {
        self.max_bytes * 3 // 3 chars per byte ("XX ")
    }
}

pub struct LogFormatter;

impl LogFormatter {
    pub fn new() -> Self {
        Self
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

    pub fn create_strategy(&self, is_hex: bool, max_bytes: usize) -> Box<dyn LogFormatterStrategy> {
        if is_hex {
            Box::new(HexFormatter { max_bytes })
        } else {
            Box::new(DefaultFormatter { max_bytes })
        }
    }
}
