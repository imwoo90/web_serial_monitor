use crate::worker::error::LogError;
use crate::worker::index::{ByteOffset, LineIndex, LineRange, LogIndex};
use crate::worker::storage::{LogStorage, StorageBackend};

/// Repository that manages log storage and indexing together
/// Ensures consistency between storage writes and index updates
pub struct LogRepository {
    pub storage: LogStorage,
    pub index: LogIndex,
}

impl LogRepository {
    pub fn new() -> Result<Self, LogError> {
        Ok(Self {
            storage: LogStorage::new()?,
            index: LogIndex::new(),
        })
    }

    /// Appends lines to storage and updates index atomically
    /// This ensures storage and index remain synchronized
    pub fn append_lines(
        &mut self,
        text: &str,
        offsets: Vec<ByteOffset>,
        filtered: Vec<LineRange>,
    ) -> Result<(), LogError> {
        let start = self.storage.backend.get_file_size()?;

        // Write to storage first
        self.storage
            .backend
            .write_at(start, self.storage.encoder.encode_with_input(text).as_ref())?;

        // Only update index if write succeeded
        for off in offsets {
            self.index.push_line(start + off.0);
        }

        for mut r in filtered {
            r.start = start + r.start.0;
            r.end = start + r.end.0;
            self.index.push_filtered(r);
        }

        Ok(())
    }

    /// Gets the current line count (filtered or total)
    pub fn get_line_count(&self) -> usize {
        self.index.get_total_count()
    }

    /// Gets the range for a specific line
    pub fn get_line_range(&self, index: LineIndex) -> Option<LineRange> {
        self.index.get_line_range(index)
    }

    /// Reads a line from storage
    pub fn read_line(&self, range: LineRange) -> Result<Vec<u8>, LogError> {
        let mut buf = vec![0u8; (range.end.0 - range.start.0) as usize];
        self.storage.backend.read_at(range.start, &mut buf)?;
        Ok(buf)
    }

    /// Clears all logs
    pub fn clear(&mut self) -> Result<(), LogError> {
        self.storage.backend.truncate(0)?;
        self.storage.backend.flush()?;
        self.index.reset_base();
        Ok(())
    }

    /// Resets the index (used when loading existing data)
    pub fn reset_index(&mut self) {
        self.index.reset_base();
    }
}
