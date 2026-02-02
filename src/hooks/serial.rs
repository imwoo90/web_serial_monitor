use crate::hooks::{use_worker_controller, WorkerController};
use crate::state::AppState;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::JsCast;
use web_sys::ReadableStreamDefaultReader;

pub fn use_serial_controller() -> SerialController {
    let state = use_context::<AppState>();
    let bridge = use_worker_controller();

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

            // Simulation doesn't use the robust read_loop/task structure yet,
            // but we update state so disconnect works.
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
        // Prevent action if busy
        if (state.conn.is_busy)() {
            return;
        }
        // Lock immediately to prevent double-click / race conditions
        state.conn.set_busy(true);

        let bridge = self.bridge;
        spawn(async move {
            let Ok(port) = crate::utils::serial_api::request_port().await else {
                state.conn.set_busy(false);
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
                state.conn.set_busy(false);
                return;
            };

            bridge.new_session();

            // Start the read task explicitly
            start_read_task(state, bridge, port);

            state.success("Connected");
            state.conn.set_busy(false);
        });
    }

    pub fn disconnect(&self) {
        let state = self.state;
        // Prevent action if busy
        if (state.conn.is_busy)() {
            return;
        }
        // Lock immediately to prevent double-click
        state.conn.set_busy(true);

        spawn(async move {
            cleanup_serial_connection(state).await;
            state.info("Disconnected");
            state.conn.set_busy(false);
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

// Helper to cleanup serial connection (Reader + Port) safely
async fn cleanup_serial_connection(state: AppState) {
    // Note: Caller must have set busy=true before calling this

    // Yield to let UI update and previous events settle
    TimeoutFuture::new(50).await;

    let maybe_reader = (state.conn.reader)();
    let maybe_port = (state.conn.port)();

    // 2. Cancel Reader if exists
    if let Some(reader_wrapper) = maybe_reader {
        let _ = crate::utils::serial_api::cancel_reader(&reader_wrapper).await;
    }

    // 3. Wait for Read Loop to finish (Release Lock)
    let mut retries = 0;
    while (state.conn.is_reading)() && retries < 50 {
        TimeoutFuture::new(50).await;
        retries += 1;
    }

    if (state.conn.is_reading)() {
        web_sys::console::warn_1(&"Timeout waiting for reader lock release".into());
    }

    // 4. Close Port if exists

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
    // state.conn.set_busy(false); // Caller is now responsible for setting busy to false
}

/// Starts an explicit read task that handles the serial read loop and retries
fn start_read_task(state: AppState, bridge: WorkerController, port: web_sys::SerialPort) {
    use crate::utils::serial_api::ReadStatus;

    spawn(async move {
        // 1. Get Reader
        let readable = port.readable();
        let reader = readable
            .get_reader()
            .unchecked_into::<ReadableStreamDefaultReader>();

        // 2. Update State (Primary location for reader)
        // We clone the port because we need to keep it for retries
        state
            .conn
            .set_connected(Some(port.clone()), Some(reader.clone()));
        state.conn.set_reading(true);

        // 3. Run Loop
        let status = crate::utils::serial_api::read_loop(reader, move |data| {
            let is_hex = (state.ui.is_hex_view)();
            bridge.append_chunk(&data, is_hex);
        })
        .await;

        // Loop finished (Lock released inside read_loop before return)
        state.conn.set_reading(false);

        // 4. Handle Result
        match status {
            ReadStatus::Retry => {
                // Prevent hot-looping on continuous errors (e.g. wrong baud rate)
                TimeoutFuture::new(100).await;

                // If busy (e.g. user clicked disconnect), stop retrying
                if (state.conn.is_busy)() {
                    return;
                }

                // Recursive restart
                start_read_task(state, bridge, port);
            }
            ReadStatus::Done => {
                if state.conn.is_connected() {
                    // If already busy, someone else (Disconnect button) is handling cleanup
                    if !(state.conn.is_busy)() {
                        state.conn.set_busy(true);
                        state.info("Connection Closed");
                        cleanup_serial_connection(state).await;
                        state.conn.set_busy(false); // Release busy lock
                    }
                }
            }
            ReadStatus::Fatal(msg) => {
                if !(state.conn.is_busy)() {
                    state.conn.set_busy(true);
                    state.error(&format!("Connection Lost: {}", msg));
                    cleanup_serial_connection(state).await;
                    state.conn.set_busy(false); // Release busy lock
                }
            }
        }
    });
}
