use crate::worker::error::LogError;
use crate::worker::index::{ActiveFilterBuilder, LineRange, LogIndex};
use crate::worker::storage::{LogStorage, StorageBackend};

pub struct LogSearcher;

const SEARCH_BATCH_SIZE: usize = 5000;

impl LogSearcher {
    pub fn search(
        storage: &mut LogStorage,
        index: &mut LogIndex,
        query: String,
        case: bool,
        regex: bool,
        invert: bool,
    ) -> Result<u32, LogError> {
        if query.trim().is_empty() {
            index.clear_filter();
            return Ok(index.line_count as u32);
        }

        index.active_filter = Some(
            ActiveFilterBuilder::new(query)
                .case_sensitive(case)
                .regex(regex)
                .invert(invert)
                .build()
                .map_err(LogError::Regex)?,
        );
        index.is_filtering = true;
        index.filtered_lines.clear();

        let total_lines = index.line_count;
        let mut buf = vec![0u8; 512 * 1024];
        let mut idx = 0;

        while idx < total_lines {
            let batch_end = (idx + SEARCH_BATCH_SIZE).min(total_lines);
            let (s_off, e_off) = {
                let off = &index.line_offsets;
                (off[idx], off[batch_end])
            };
            let size = (e_off.0 - s_off.0) as usize;
            if buf.len() < size {
                buf.resize(size, 0);
            }

            storage.backend.read_at(s_off, &mut buf[..size])?;

            let text = storage
                .decoder
                .decode_with_u8_array(&buf[..size])
                .map_err(LogError::Js)?;

            let filter = index
                .active_filter
                .as_ref()
                .ok_or_else(|| LogError::Regex("Filter missing during search".into()))?
                .clone();

            for (j, line) in text.trim_end_matches('\n').split('\n').enumerate() {
                if filter.matches(line) {
                    let off_ptr = &index.line_offsets;
                    // Ensure we don't go out of bounds if split produces more lines than expected
                    if idx + j + 1 < off_ptr.len() {
                        let range = LineRange {
                            start: off_ptr[idx + j],
                            end: off_ptr[idx + j + 1],
                        };
                        index.push_filtered(range);
                    }
                }
            }
            idx = batch_end;
        }
        Ok(index.filtered_lines.len() as u32)
    }
}
