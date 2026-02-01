use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ReadableStreamDefaultReader, SerialOptions, SerialPort, WritableStreamDefaultWriter,
};

#[derive(Serialize)]
struct SerialOptionsParams {
    #[serde(rename = "baudRate")]
    baud_rate: u32,
    #[serde(rename = "dataBits")]
    data_bits: u8,
    #[serde(rename = "stopBits")]
    stop_bits: u8,
    parity: String,
    #[serde(rename = "flowControl")]
    flow_control: String,
}

pub async fn request_port() -> Result<SerialPort, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let navigator = window.navigator();
    let serial = navigator.serial();

    // web-sys binding might return Promise directly and take 0 args in this version?
    let promise = serial.request_port();
    let result = JsFuture::from(promise).await?;
    Ok(result.into())
}

pub async fn open_port(
    port: &SerialPort,
    baud_rate: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: &str,
    flow_control: &str,
) -> Result<(), JsValue> {
    let params = SerialOptionsParams {
        baud_rate,
        data_bits,
        stop_bits,
        parity: parity.to_lowercase(),
        flow_control: flow_control.to_lowercase(),
    };

    let options_val = serde_wasm_bindgen::to_value(&params)?;
    let options: SerialOptions = options_val.unchecked_into();

    let promise = port.open(&options);
    JsFuture::from(promise).await.map(|_| ())
}

pub async fn read_loop(
    reader: ReadableStreamDefaultReader,
    mut on_data: impl FnMut(Vec<u8>) + 'static,
    mut on_error: impl FnMut(String) + 'static,
) {
    loop {
        let promise = reader.read();
        match JsFuture::from(promise).await {
            Ok(result) => {
                let done = js_sys::Reflect::get(&result, &"done".into())
                    .ok()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let value = match js_sys::Reflect::get(&result, &"value".into()) {
                    Ok(v) => v,
                    Err(_) => {
                        on_error("Failed to get read value".to_string());
                        reader.release_lock();
                        break;
                    }
                };

                if done {
                    reader.release_lock();
                    break;
                }

                if !value.is_undefined() && !value.is_null() {
                    let array = js_sys::Uint8Array::new(&value);
                    let bytes = array.to_vec();
                    on_data(bytes);
                }
            }
            Err(e) => {
                let err_str = format!("{:?}", e);
                // Check for fatal errors that require closing the connection
                // "NetworkError" is the standard DOMException for lost device
                // "The device has been lost" is the common message text
                if err_str.contains("NetworkError") || err_str.contains("device has been lost") {
                    on_error(format!("Fatal Error: {:?}", e));
                    reader.release_lock();
                    break;
                } else {
                    // Non-fatal errors (Framing, Parity, Break, BufferOverrun)
                    // Just log warning and continue reading
                    web_sys::console::warn_1(&format!("Non-fatal read error: {:?}", e).into());
                }
            }
        }
    }
}

pub async fn cancel_reader(reader: &ReadableStreamDefaultReader) -> Result<(), JsValue> {
    let promise = reader.cancel();
    JsFuture::from(promise).await.map(|_| ())
}

pub async fn send_data(port: &SerialPort, data: &[u8]) -> Result<(), JsValue> {
    let writable = port.writable();
    // get_writer can throw
    let writer = writable.get_writer()?;
    let writer: WritableStreamDefaultWriter = writer.unchecked_into();

    let array = js_sys::Uint8Array::from(data);
    let promise = writer.write_with_chunk(&array);
    match JsFuture::from(promise).await {
        Ok(_) => {
            writer.release_lock();
            Ok(())
        }
        Err(e) => {
            writer.release_lock();
            Err(e)
        }
    }
}

pub async fn close_port(port: &SerialPort) -> Result<(), JsValue> {
    let promise = port.close();
    JsFuture::from(promise).await.map(|_| ())
}
