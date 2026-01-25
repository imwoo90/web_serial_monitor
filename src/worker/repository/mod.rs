pub mod index;
pub mod storage;

const NEWLINE: u8 = b'\n';

use self::index::{ByteOffset, LineIndex, LineRange, LogIndex};
use self::storage::{LogStorage, StorageBackend};
use crate::config::READ_BUFFER_SIZE;
use crate::worker::error::LogError;
use web_sys::FileSystemSyncAccessHandle;

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

    pub fn initialize_storage(
        &mut self,
        handle: FileSystemSyncAccessHandle,
    ) -> Result<(), LogError> {
        self.storage.backend.handle = Some(handle);
        let size = self.storage.backend.get_file_size()?;

        if size.0 > 0 {
            self.reset_index();
            let (mut off, mut buf) = (ByteOffset(0), vec![0u8; READ_BUFFER_SIZE]);
            while off.0 < size.0 {
                let len = (size.0 - off.0).min(buf.len() as u64) as usize;
                self.storage.backend.read_at(off, &mut buf[..len])?;
                for (i, &b) in buf[..len].iter().enumerate() {
                    if b == NEWLINE {
                        self.index.push_line(off + (i as u64 + 1));
                    }
                }
                off = off + (len as u64);
            }
        }
        Ok(())
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

    /// Checks if filtering is active
    pub fn is_filtering(&self) -> bool {
        self.index.is_filtering
    }

    /// Checks if text matches the active filter
    pub fn matches_active_filter(&self, text: &str) -> bool {
        if !self.index.is_filtering {
            return false;
        }
        self.index
            .active_filter
            .as_ref()
            .is_some_and(|f| f.matches(text))
    }

    /// Decodes a chunk of bytes using the storage's decoder with streaming enabled
    pub fn decode_chunk(&self, chunk: &[u8]) -> Result<String, LogError> {
        let opts = web_sys::TextDecodeOptions::new();
        opts.set_stream(true);
        self.storage
            .decoder
            .decode_with_u8_array_and_options(chunk, &opts)
            .map_err(LogError::from)
    }
}
