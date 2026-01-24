use crate::worker::error::LogError;
use crate::worker::index::ByteOffset;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub trait StorageBackend {
    fn read_at(&self, offset: ByteOffset, buf: &mut [u8]) -> Result<usize, LogError>;
    fn write_at(&self, offset: ByteOffset, data: &[u8]) -> Result<usize, LogError>;
    fn get_file_size(&self) -> Result<ByteOffset, LogError>;
    fn truncate(&self, size: u64) -> Result<(), LogError>;
    fn flush(&self) -> Result<(), LogError>;
}

pub struct OpfsBackend {
    pub handle: Option<web_sys::FileSystemSyncAccessHandle>,
}

impl StorageBackend for OpfsBackend {
    fn read_at(&self, offset: ByteOffset, buf: &mut [u8]) -> Result<usize, LogError> {
        let handle = self
            .handle
            .as_ref()
            .ok_or_else(|| LogError::Storage("No handle".into()))?;
        let opts = web_sys::FileSystemReadWriteOptions::new();
        opts.set_at(offset.0 as f64);
        handle
            .read_with_u8_array_and_options(buf, &opts)
            .map(|n| n as usize)
            .map_err(LogError::from)
    }

    fn write_at(&self, offset: ByteOffset, data: &[u8]) -> Result<usize, LogError> {
        let handle = self
            .handle
            .as_ref()
            .ok_or_else(|| LogError::Storage("No handle".into()))?;
        let opts = web_sys::FileSystemReadWriteOptions::new();
        opts.set_at(offset.0 as f64);
        handle
            .write_with_u8_array_and_options(data, &opts)
            .map(|n| n as usize)
            .map_err(LogError::from)
    }

    fn get_file_size(&self) -> Result<ByteOffset, LogError> {
        self.handle
            .as_ref()
            .ok_or_else(|| LogError::Storage("No handle".into()))?
            .get_size()
            .map(|s| ByteOffset(s as u64))
            .map_err(LogError::from)
    }

    fn truncate(&self, size: u64) -> Result<(), LogError> {
        self.handle
            .as_ref()
            .ok_or_else(|| LogError::Storage("No handle".into()))?
            .truncate_with_f64(size as f64)
            .map_err(LogError::from)
    }

    fn flush(&self) -> Result<(), LogError> {
        self.handle
            .as_ref()
            .ok_or_else(|| LogError::Storage("No handle".into()))?
            .flush()
            .map_err(LogError::from)
    }
}

pub async fn get_opfs_root() -> Result<web_sys::FileSystemDirectoryHandle, JsValue> {
    let global = js_sys::global();
    let navigator = js_sys::Reflect::get(&global, &"navigator".into())?;
    let storage = js_sys::Reflect::get(&navigator, &"storage".into())?;
    let storage: web_sys::StorageManager = storage.unchecked_into();
    let root = wasm_bindgen_futures::JsFuture::from(storage.get_directory()).await?;
    Ok(root.into())
}

pub async fn get_lock(
    file_handle: web_sys::FileSystemFileHandle,
) -> Result<web_sys::FileSystemSyncAccessHandle, JsValue> {
    for _ in 0..20 {
        match wasm_bindgen_futures::JsFuture::from(file_handle.create_sync_access_handle()).await {
            Ok(h) => return Ok(h.into()),
            Err(e) => {
                let error_name = js_sys::Reflect::get(&e, &"name".into()).unwrap_or_default();
                if error_name == "NoModificationAllowedError" || error_name == "InvalidStateError" {
                    gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err("Failed to acquire OPFS lock after retries".into())
}

async fn get_files(
    root: &web_sys::FileSystemDirectoryHandle,
) -> Result<Vec<(String, web_sys::FileSystemFileHandle)>, JsValue> {
    let mut files = Vec::new();
    let entries_fn = js_sys::Reflect::get(root, &"entries".into())?;
    let iterator = js_sys::Function::from(entries_fn)
        .call0(root)?
        .unchecked_into::<js_sys::AsyncIterator>();

    loop {
        let result = wasm_bindgen_futures::JsFuture::from(iterator.next()?).await?;
        let done = js_sys::Reflect::get(&result, &"done".into())?
            .as_bool()
            .unwrap_or(true);
        if done {
            break;
        }
        let value = js_sys::Reflect::get(&result, &"value".into())?;
        let entry = value.unchecked_into::<js_sys::Array>();
        let name = entry.get(0).as_string().unwrap_or_default();
        if name.starts_with("logs_") && name.ends_with(".txt") {
            let handle = entry
                .get(1)
                .unchecked_into::<web_sys::FileSystemFileHandle>();
            files.push((name, handle));
        }
    }

    files.sort_by(|a, b| {
        let ts_a = a.0[5..a.0.len() - 4].parse::<u64>().unwrap_or(0);
        let ts_b = b.0[5..b.0.len() - 4].parse::<u64>().unwrap_or(0);
        ts_b.cmp(&ts_a)
    });

    Ok(files)
}

pub async fn new_session(
    root: &web_sys::FileSystemDirectoryHandle,
    cleanup_current: bool,
    current_filename: &mut Option<String>,
) -> Result<web_sys::FileSystemSyncAccessHandle, JsValue> {
    if cleanup_current {
        if let Some(name) = current_filename {
            let _ = wasm_bindgen_futures::JsFuture::from(root.remove_entry(name)).await;
        }
    }

    let filename = format!("logs_{}.txt", chrono::Utc::now().timestamp_millis());
    let opts = web_sys::FileSystemGetFileOptions::new();
    opts.set_create(true);
    let file_handle =
        wasm_bindgen_futures::JsFuture::from(root.get_file_handle_with_options(&filename, &opts))
            .await?;
    let file_handle: web_sys::FileSystemFileHandle = file_handle.into();

    let lock = get_lock(file_handle).await?;
    *current_filename = Some(filename);
    Ok(lock)
}

// Redefine setup_opfs_manual properly to return the handle
pub async fn init_opfs_session(
    current_filename: &mut Option<String>,
) -> Result<web_sys::FileSystemSyncAccessHandle, JsValue> {
    let root = get_opfs_root().await?;
    let files = get_files(&root).await?;

    if let Some((name, handle)) = files.first().cloned() {
        match get_lock(handle).await {
            Ok(lock) => {
                *current_filename = Some(name);
                // Cleanup others
                for i in 1..files.len() {
                    let _ =
                        wasm_bindgen_futures::JsFuture::from(root.remove_entry(&files[i].0)).await;
                }
                Ok(lock)
            }
            Err(_) => {
                // If lock fails, start new
                let res = new_session(&root, false, current_filename).await;
                // Cleanup all including the failed one
                for i in 0..files.len() {
                    let _ =
                        wasm_bindgen_futures::JsFuture::from(root.remove_entry(&files[i].0)).await;
                }
                res
            }
        }
    } else {
        new_session(&root, false, current_filename).await
    }
}
