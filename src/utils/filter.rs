use regex::Regex;

pub struct LogFilter {
    query: String,
    match_case: bool,
    invert: bool,
    regex: Option<Regex>,
}

impl LogFilter {
    pub fn new(query: String, match_case: bool, use_regex: bool, invert: bool) -> Self {
        let regex = if use_regex && !query.is_empty() {
            Regex::new(&query).ok()
        } else {
            None
        };

        Self {
            query,
            match_case,
            invert,
            regex,
        }
    }

    pub fn matches(&self, line: &str) -> bool {
        if self.query.is_empty() {
            return true;
        }

        let is_match = if let Some(re) = &self.regex {
            re.is_match(line)
        } else {
            if self.match_case {
                line.contains(&self.query)
            } else {
                line.to_lowercase().contains(&self.query.to_lowercase())
            }
        };

        if self.invert {
            !is_match
        } else {
            is_match
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_filter() {
        let filter = LogFilter::new("Error".to_string(), true, false, false);
        assert!(filter.matches("Critical Error occurred"));
        assert!(!filter.matches("critical error occurred"));

        let filter = LogFilter::new("error".to_string(), false, false, false);
        assert!(filter.matches("Critical Error occurred"));

        let filter = LogFilter::new(r"Err\d+".to_string(), false, true, false);
        assert!(filter.matches("Err123"));
        assert!(!filter.matches("Err"));

        let filter = LogFilter::new("debug".to_string(), false, false, true);
        assert!(!filter.matches("Debug log"));
        assert!(filter.matches("Info log"));
    }
}
