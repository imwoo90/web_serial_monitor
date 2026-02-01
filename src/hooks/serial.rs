use crate::hooks::{use_worker_controller, WorkerController};
use crate::state::AppState;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::JsCast;
use web_sys::ReadableStreamDefaultReader;

pub fn use_serial_controller() -> SerialController {
    let state = use_context::<AppState>();
    let bridge = use_worker_controller();

    // Read loop resource
    use_resource(move || {
        let reader = (state.conn.reader)();
        let bridge = bridge;
        async move {
            let Some(reader_wrapper) = reader else { return };

            crate::utils::serial_api::read_loop(
                reader_wrapper,
                move |data| {
                    let is_hex = (state.ui.is_hex_view)();
                    bridge.append_chunk(&data, is_hex);
                },
                move |err_msg| {
                    state.error(&format!("Connection Lost: {}", err_msg));
                    spawn(async move {
                        cleanup_serial_connection(state).await;
                    });
                },
            )
            .await;

            if state.conn.is_connected() && (state.conn.reader)().is_some() {
                state.info("Connection Closed");
                cleanup_serial_connection(state).await;
            }
        }
    });

    // Simulation resource
    use_resource(move || {
        let simulating = (state.conn.is_simulating)();
        async move {
            if !simulating {
                return;
            }
            let stream = crate::utils::simulation::create_simulation_stream();
            let reader = stream
                .get_reader()
                .unchecked_into::<ReadableStreamDefaultReader>();
            state.conn.set_connected(None, Some(reader));
        }
    });

    SerialController { state, bridge }
}

#[derive(Clone, Copy)]
pub struct SerialController {
    state: AppState,
    bridge: WorkerController,
}

impl SerialController {
    pub fn connect(&self) {
        let state = self.state;
        let bridge = self.bridge;
        spawn(async move {
            let Ok(port) = crate::utils::serial_api::request_port().await else {
                return;
            };

            if crate::utils::serial_api::open_port(
                &port,
                (state.serial.baud_rate)(),
                (state.serial.data_bits)(),
                (state.serial.stop_bits)(),
                &(state.serial.parity)().to_string(),
                &(state.serial.flow_control)().to_string(),
            )
            .await
            .is_err()
            {
                state.error("Failed to Open Port");
                return;
            };

            bridge.new_session();
            let readable = port.readable();
            let reader = readable.get_reader();
            let reader: ReadableStreamDefaultReader = reader.unchecked_into();
            state
                .conn
                .set_connected(Some(port.clone()), Some(reader.clone()));
            state.success("Connected");
        });
    }

    pub fn disconnect(&self) {
        let state = self.state;
        spawn(async move {
            cleanup_serial_connection(state).await;
            state.info("Disconnected");
        });
    }

    pub fn start_simulation(&self) {
        self.state.conn.set_simulating(true);
        self.state.success("Simulation Started");
        self.bridge.clear();
    }

    pub fn stop_simulation(&self) {
        self.state.conn.set_simulating(false);
        self.state.warning("Simulation Stopped");
        self.disconnect();
    }
}

/// Helper to cleanup serial connection (Reader + Port) safely
async fn cleanup_serial_connection(mut state: AppState) {
    let maybe_reader = (state.conn.reader)();
    let maybe_port = (state.conn.port)();

    // 1. Reset Reader state immediately to stop potential re-entry
    state.conn.reader.set(None);

    // 2. Cancel Reader if exists
    if let Some(reader_wrapper) = maybe_reader {
        let _ = crate::utils::serial_api::cancel_reader(&reader_wrapper).await;
    }

    // 3. Close Port if exists
    // Small delay to ensure reader lock is released properly by browser
    TimeoutFuture::new(100).await;

    if let Some(conn_port) = maybe_port {
        if crate::utils::serial_api::close_port(&conn_port)
            .await
            .is_err()
        {
            // Log error if needed, but we proceed to reset state anyway
            web_sys::console::warn_1(&"Failed to close port cleanly".into());
        }
    }

    // 4. Final State Reset
    state.conn.set_connected(None, None);
}
