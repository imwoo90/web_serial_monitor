use crate::config::EXPORT_CHUNK_SIZE;
use crate::worker::error::LogError;
use crate::worker::repository::index::ByteOffset;
use wasm_bindgen::prelude::*;
use wasm_streams::ReadableStream;
use web_sys::{FileSystemReadWriteOptions, FileSystemSyncAccessHandle};

/// Handles log export functionality
pub struct LogExporter;

impl LogExporter {
    pub fn new() -> Self {
        Self
    }

    /// Creates a ReadableStream for exporting logs
    pub fn export_logs(
        handle: FileSystemSyncAccessHandle,
        file_size: ByteOffset,
    ) -> Result<js_sys::Object, LogError> {
        let size = file_size;
        let backend = handle;

        let stream = futures_util::stream::unfold(ByteOffset(0), move |off| {
            let h = backend.clone();
            async move {
                if off.0 >= size.0 {
                    return None;
                }
                let len = (size.0 - off.0).min(EXPORT_CHUNK_SIZE) as usize;
                let mut buf = vec![0u8; len];
                let opts = FileSystemReadWriteOptions::new();
                opts.set_at(off.0 as f64);
                if h.read_with_u8_array_and_options(&mut buf, &opts).is_err() {
                    return None;
                }

                let res = JsValue::from(js_sys::Uint8Array::from(&buf[..]));
                Some((Ok(res), ByteOffset(off.0 + len as u64)))
            }
        });
        Ok(ReadableStream::from_stream(stream).into_raw().into())
    }
}

impl Default for LogExporter {
    fn default() -> Self {
        Self::new()
    }
}
