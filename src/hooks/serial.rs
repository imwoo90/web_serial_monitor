use crate::hooks::{use_worker_controller, WorkerController};
use crate::state::AppState;
use crate::utils::serial;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::JsCast;
use web_sys::ReadableStreamDefaultReader;

pub fn use_serial_controller() -> SerialController {
    let state = use_context::<AppState>();
    let bridge = use_worker_controller();
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
            if let Ok(port) = serial::request_port().await {
                let baud = (state.serial.baud_rate)();
                let data_bits = (state.serial.data_bits)();
                let stop_bits = (state.serial.stop_bits)();
                let parity = (state.serial.parity)().to_string();
                let flow_control = (state.serial.flow_control)().to_string();

                if serial::open_port(&port, baud, data_bits, stop_bits, &parity, &flow_control)
                    .await
                    .is_ok()
                {
                    bridge.new_session();
                    let readable = port.readable();
                    let reader = readable.get_reader();
                    let reader: ReadableStreamDefaultReader = reader.unchecked_into();
                    state
                        .conn
                        .set_connected(Some(port.clone()), Some(reader.clone()));

                    state.success("Connected");

                    serial::read_loop(
                        reader,
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

                    if (state.conn.is_connected)() && (state.conn.reader)().is_some() {
                        state.conn.set_connected(None, None);
                        state.info("Connection Closed");
                    }
                } else {
                    state.error("Failed to Open Port");
                }
            }
        });
    }

    pub fn disconnect(&self) {
        let state = self.state;
        spawn(async move {
            let maybe_reader = (state.conn.reader)();
            let maybe_port = (state.conn.port)();

            let mut r = state.conn.reader;
            r.set(None);

            if let Some(reader_wrapper) = maybe_reader {
                let _ = serial::cancel_reader(&reader_wrapper.0).await;
            }

            TimeoutFuture::new(100).await;

            if let Some(wrapper) = maybe_port {
                if serial::close_port(&wrapper.0).await.is_ok() {
                    state.info("Disconnected");
                } else {
                    state.error("Failed to close port");
                }
            }

            state.conn.set_connected(None, None);
        });
    }

    pub fn start_simulation(&self) {
        let state = self.state;
        let bridge = self.bridge;
        state.conn.set_simulating(true);
        state.success("Simulation Started");
        bridge.clear();

        let sim_sig = state.conn.is_simulating;
        let hex_sig = state.ui.is_hex_view;

        spawn(async move {
            loop {
                if !sim_sig() {
                    break;
                }
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
        });
    }

    pub fn stop_simulation(&self) {
        self.state.conn.set_simulating(false);
        self.state.warning("Simulation Stopped");
    }
}
