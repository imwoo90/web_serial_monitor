use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use wasm_streams::ReadableStream;

pub fn create_simulation_stream() -> web_sys::ReadableStream {
    let stream = futures_util::stream::unfold((), |()| async move {
        TimeoutFuture::new(10).await; // Using 10ms to prevent overwhelming the UI, can be adjusted.

        let rnd = js_sys::Math::random();
        // Generate random bytes directly to support simulation of corrupted data
        let mut bytes = Vec::new();

        if rnd < 0.05 {
            // Simulate garbage / corrupted data (invalid UTF-8)
            // 0xFF, 0xC0 (invalid start byte), 0x80 (continuation byte without start)
            bytes.extend_from_slice(&[0xFF, 0xC0, 0xFE, 0x80, 0x12, 0x34]);
        } else if rnd < 0.15 {
            bytes.extend_from_slice(
                format!("Error: System overheat at {:.1}°C\n", 80.0 + rnd * 20.0).as_bytes(),
            );
        } else if rnd < 0.35 {
            bytes.extend_from_slice(
                format!("Warning: Voltage fluctuation detected: {:.2}V\n", 3.0 + rnd).as_bytes(),
            );
        } else {
            bytes.extend_from_slice(
                format!(
                    "Info: Sensor reading: A={:.2}, B={:.2}, C={:.2}\n",
                    rnd * 100.0,
                    rnd * 50.0,
                    rnd * 10.0
                )
                .as_bytes(),
            );
        }

        let chunk = js_sys::Uint8Array::from(bytes.as_slice());
        // Stream expects Result<JsValue, JsValue>
        Some((Ok(JsValue::from(chunk)), ()))
    });

    ReadableStream::from_stream(stream).into_raw()
}
