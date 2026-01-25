use crate::components::console::bridge::use_worker_bridge;
use crate::serial;
use crate::state::{AppState, ReaderWrapper, SerialPortWrapper};
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::JsCast;
use web_sys::ReadableStreamDefaultReader;

pub fn use_serial_controller() -> SerialController {
    let state = use_context::<AppState>();
    let bridge = use_worker_bridge();

    let connect = {
        let mut state = state;
        let bridge = bridge;
        move || {
            spawn(async move {
                if let Ok(port) = serial::request_port().await {
                    let baud = (state.baud_rate)().parse().unwrap_or(115200);
                    let data_bits = (state.data_bits)().parse().unwrap_or(8);
                    let stop_bits = if (state.stop_bits)() == "2" { 2 } else { 1 };

                    if serial::open_port(
                        &port,
                        baud,
                        data_bits,
                        stop_bits,
                        (state.parity)(),
                        (state.flow_control)(),
                    )
                    .await
                    .is_ok()
                    {
                        bridge.new_session();
                        state.port.set(Some(SerialPortWrapper(port.clone())));
                        state.is_connected.set(true);

                        let readable = port.readable();
                        let reader = readable.get_reader();
                        let reader: ReadableStreamDefaultReader = reader.unchecked_into();
                        state.reader.set(Some(ReaderWrapper(reader.clone())));

                        state.success("Connected");

                        serial::read_loop(
                            reader,
                            move |data| {
                                let is_hex = (state.is_hex_view)();
                                bridge.append_chunk(&data, is_hex);
                            },
                            move |_| {
                                state.is_connected.set(false);
                                state.port.set(None);
                                state.reader.set(None);
                                state.error("Connection Lost");
                            },
                        )
                        .await;

                        if (state.is_connected)() {
                            if (state.reader)().is_some() {
                                state.is_connected.set(false);
                                state.port.set(None);
                                state.reader.set(None);
                                state.info("Connection Closed");
                            }
                        }
                    } else {
                        state.error("Failed to Open Port");
                    }
                }
            });
        }
    };

    let disconnect = {
        let mut state = state;
        move || {
            spawn(async move {
                let maybe_reader = (state.reader)();
                let maybe_port = (state.port)();

                state.reader.set(None);

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

                state.port.set(None);
                state.is_connected.set(false);
            });
        }
    };

    let start_simulation = {
        let mut state = state;
        let bridge = bridge;
        move || {
            state.is_simulating.set(true);
            state.info("Simulation Started");
            bridge.clear();

            let sim_sig = state.is_simulating;
            let hex_sig = state.is_hex_view;

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
    };

    let stop_simulation = {
        let mut state = state;
        move || {
            state.is_simulating.set(false);
            state.info("Simulation Stopped");
        }
    };

    SerialController {
        connect: Box::new(connect),
        disconnect: Box::new(disconnect),
        start_simulation: Box::new(start_simulation),
        stop_simulation: Box::new(stop_simulation),
    }
}

pub struct SerialController {
    pub connect: Box<dyn FnMut()>,
    pub disconnect: Box<dyn FnMut()>,
    pub start_simulation: Box<dyn FnMut()>,
    pub stop_simulation: Box<dyn FnMut()>,
}
