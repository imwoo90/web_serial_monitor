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
                move |_| {
                    state.conn.set_connected(None, None);
                    state.error("Connection Lost");
                },
            )
            .await;

            if state.conn.is_connected() && (state.conn.reader)().is_some() {
                state.conn.set_connected(None, None);
                state.info("Connection Closed");
            }
        }
    });

    // Simulation resource
    use_resource(move || {
        let simulating = (state.conn.is_simulating)();
        let hex_sig = state.ui.is_hex_view;
        async move {
            if !simulating {
                return;
            }
            loop {
                let rnd = js_sys::Math::random();
                let content = if rnd < 0.1 {
                    format!("Error: System overheat at {:.1}°C\n", 80.0 + rnd * 20.0)
                } else if rnd < 0.3 {
                    format!("Warning: Voltage fluctuation detected: {:.2}V\n", 3.0 + rnd)
                } else {
                    format!(
                        "Info: Sensor reading: A={:.2}, B={:.2}, C={:.2}\n",
                        rnd * 100.0,
                        rnd * 50.0,
                        rnd * 10.0
                    )
                };
                bridge.append_chunk(content.as_bytes(), hex_sig());
                TimeoutFuture::new(1).await;
            }
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
        let mut state = self.state;
        spawn(async move {
            let maybe_reader = (state.conn.reader)();
            let maybe_port = (state.conn.port)();

            state.conn.reader.set(None);

            if let Some(reader_wrapper) = maybe_reader {
                let _ = crate::utils::serial_api::cancel_reader(&reader_wrapper).await;
            }

            TimeoutFuture::new(100).await;

            if let Some(conn_port) = maybe_port {
                if crate::utils::serial_api::close_port(&conn_port)
                    .await
                    .is_ok()
                {
                    state.info("Disconnected");
                } else {
                    state.error("Failed to close port");
                }
            }

            state.conn.set_connected(None, None);
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
    }
}
