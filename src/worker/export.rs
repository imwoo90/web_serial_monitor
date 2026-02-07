use crate::config::EXPORT_CHUNK_SIZE;
use crate::worker::error::LogError;
use crate::worker::repository::index::ByteOffset;
use wasm_bindgen::prelude::*;
use wasm_streams::ReadableStream;
use web_sys::{FileSystemReadWriteOptions, FileSystemSyncAccessHandle, TextDecoder, TextEncoder};

/// Handles log export functionality
pub struct LogExporter;

impl LogExporter {
    pub fn new() -> Self {
        Self
    }

    /// Creates a ReadableStream for exporting logs
    pub fn export_logs(
        handle: FileSystemSyncAccessHandle,
        decoder: TextDecoder,
        encoder: TextEncoder,
        file_size: ByteOffset,
        include_timestamp: bool,
    ) -> Result<js_sys::Object, LogError> {
        let size = file_size;
        let backend = handle;
        let dec = decoder;
        let enc = encoder;
        let ts = include_timestamp;

        let stream = futures_util::stream::unfold(ByteOffset(0), move |off| {
            let (h, d, e) = (backend.clone(), dec.clone(), enc.clone());
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

                let res = if ts {
                    JsValue::from(js_sys::Uint8Array::from(&buf[..]))
                } else {
                    let text = d.decode_with_u8_array(&buf).unwrap_or_default();
                    let out = text
                        .split('\n')
                        .map(|l| if l.len() > 15 { &l[15..] } else { l })
                        .collect::<Vec<_>>()
                        .join("\n");
                    JsValue::from(e.encode_with_input(&out))
                };
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
