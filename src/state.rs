use crate::components::ui::{ToastMessage, ToastType};
pub use crate::types::*;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use web_sys::{ReadableStreamDefaultReader, SerialPort};

#[derive(Clone, Copy)]
pub struct UIState {
    pub show_settings: Signal<bool>,
    pub show_highlights: Signal<bool>,
    pub show_timestamps: Signal<bool>,
    pub autoscroll: Signal<bool>,
    pub is_hex_view: Signal<bool>,
    pub view_mode: Signal<ViewMode>,
    pub font_size: Signal<u32>,
}

#[derive(Clone, Copy)]
pub struct SerialSettings {
    pub baud_rate: Signal<u32>,
    pub data_bits: Signal<u8>,
    pub stop_bits: Signal<u8>,
    pub parity: Signal<Parity>,
    pub flow_control: Signal<FlowControl>,

    pub tx_line_ending: Signal<LineEnding>,
    pub tx_local_echo: Signal<bool>,
}

#[derive(Clone, Copy)]
pub struct ConnectionState {
    pub port: Signal<Option<SerialPort>>,
    pub reader: Signal<Option<ReadableStreamDefaultReader>>,
    pub is_simulating: Signal<bool>,
    pub log_worker: Signal<Option<web_sys::Worker>>,
    pub is_busy: Signal<bool>,
    pub is_reading: Signal<bool>,
}

#[derive(Clone, Copy)]
pub struct LogState {
    pub total_lines: Signal<usize>,
    pub visible_logs: Signal<Vec<(usize, String)>>,
    pub filter_query: Signal<String>,
    pub match_case: Signal<bool>,
    pub use_regex: Signal<bool>,
    pub invert_filter: Signal<bool>,
    pub highlights: Signal<Vec<Highlight>>,
    pub toasts: Signal<Vec<ToastMessage>>,
    pub active_line: Signal<Option<String>>,
}

#[derive(Clone, Copy)]
pub struct TerminalState {
    pub received_data: Signal<Vec<u8>>,
    pub scrollback: Signal<u32>,
    pub lines: Signal<usize>,
    pub autoscroll: Signal<bool>,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub ui: UIState,
    pub serial: SerialSettings,
    pub conn: ConnectionState,
    pub log: LogState,
    pub terminal: TerminalState,
}

impl UIState {
    pub fn toggle_settings(&self) {
        { self.show_settings }.toggle();
    }
    pub fn toggle_highlights(&self) {
        { self.show_highlights }.toggle();
    }
    pub fn toggle_timestamps(&self) {
        { self.show_timestamps }.toggle();
    }
    pub fn toggle_autoscroll(&self) {
        { self.autoscroll }.toggle();
    }
    pub fn set_autoscroll(&self, value: bool) {
        { self.autoscroll }.set(value);
    }
    pub fn toggle_hex_view(&self) {
        { self.is_hex_view }.toggle();
    }
    pub fn set_view_mode(&self, mode: ViewMode) {
        { self.view_mode }.set(mode);
    }
}

impl SerialSettings {
    pub fn set_baud_rate(&self, rate: u32) {
        { self.baud_rate }.set(rate);
    }
    pub fn set_data_bits(&self, bits: u8) {
        { self.data_bits }.set(bits);
    }
    pub fn set_stop_bits(&self, bits: u8) {
        { self.stop_bits }.set(bits);
    }
    pub fn set_parity(&self, p: Parity) {
        { self.parity }.set(p);
    }
    pub fn set_flow_control(&self, f: FlowControl) {
        { self.flow_control }.set(f);
    }
}

impl ConnectionState {
    pub fn is_connected(&self) -> bool {
        self.port.read().is_some()
    }

    pub fn set_connected(
        &self,
        port: Option<SerialPort>,
        reader: Option<ReadableStreamDefaultReader>,
    ) {
        { self.port }.set(port);
        { self.reader }.set(reader);
    }

    pub fn set_simulating(&self, simulating: bool) {
        { self.is_simulating }.set(simulating);
    }
    pub fn set_busy(&self, busy: bool) {
        { self.is_busy }.set(busy);
    }
    pub fn set_reading(&self, reading: bool) {
        { self.is_reading }.set(reading);
    }
}

impl LogState {
    pub fn clear(&self) {
        { self.total_lines }.set(0);
        { self.visible_logs }.set(Vec::new());
    }

    pub fn add_toast(&self, message: &str, type_: ToastType) {
        let mut toasts = self.toasts;
        let id = js_sys::Date::now() as usize;

        toasts.write().push(ToastMessage {
            id,
            message: message.to_string(),
            type_,
        });

        spawn(async move {
            TimeoutFuture::new(crate::config::TOAST_DURATION_MS).await;
            toasts.write().retain(|t| t.id != id);
        });
    }

    pub fn add_highlight(&self, text: String, color: &'static str) {
        let mut highlights = self.highlights;
        let mut list = highlights.write();
        let next_id = list.last().map(|h| h.id).unwrap_or(0) + 1;
        list.push(Highlight {
            id: next_id,
            text,
            color,
        });
    }

    pub fn remove_highlight(&self, id: usize) {
        { self.highlights }.write().retain(|h| h.id != id);
    }
}

impl TerminalState {
    pub fn push_data(&self, data: Vec<u8>) {
        let mut signal = self.received_data;
        let mut buffer = signal.write();
        buffer.extend(data);
    }

    pub fn take_data(&self) -> Vec<u8> {
        let mut signal = self.received_data;
        if signal.peek().is_empty() {
            return Vec::new();
        }

        let mut buffer = signal.write();
        std::mem::take(&mut *buffer)
    }

    pub fn clear(&self) {
        { self.received_data }.set(Vec::new());
    }
}

pub fn use_provide_app_state() -> AppState {
    let app_state = AppState {
        ui: UIState {
            show_settings: use_signal(|| false),
            show_highlights: use_signal(|| false),
            show_timestamps: use_signal(|| false),
            autoscroll: use_signal(|| true),
            is_hex_view: use_signal(|| false),
            view_mode: use_signal(|| ViewMode::Monitoring),
            font_size: use_signal(|| 14),
        },
        serial: SerialSettings {
            baud_rate: use_signal(|| 115200u32),
            data_bits: use_signal(|| 8u8),
            stop_bits: use_signal(|| 1u8),
            parity: use_signal(|| Parity::None),
            flow_control: use_signal(|| FlowControl::None),

            tx_line_ending: use_signal(|| LineEnding::None),
            tx_local_echo: use_signal(|| false),
        },
        conn: ConnectionState {
            port: use_signal(|| None),
            reader: use_signal(|| None),
            is_simulating: use_signal(|| false),
            log_worker: use_signal(|| None::<web_sys::Worker>),
            is_busy: use_signal(|| false),
            is_reading: use_signal(|| false),
        },
        log: LogState {
            total_lines: use_signal(|| 0usize),
            visible_logs: use_signal(Vec::<(usize, String)>::new),
            filter_query: use_signal(String::new),
            match_case: use_signal(|| false),
            use_regex: use_signal(|| false),
            invert_filter: use_signal(|| false),
            highlights: use_signal(Vec::new),
            toasts: use_signal(Vec::new),
            active_line: use_signal(|| None),
        },
        terminal: TerminalState {
            received_data: use_signal(Vec::new),
            scrollback: use_signal(|| 1000),
            lines: use_signal(|| 0),
            autoscroll: use_signal(|| true),
        },
    };

    use_context_provider(|| app_state);
    app_state
}

impl AppState {
    pub fn add_toast(&self, message: &str, type_: ToastType) {
        self.log.add_toast(message, type_);
    }

    pub fn success(&self, msg: &str) {
        self.add_toast(msg, ToastType::Success);
    }
    pub fn error(&self, msg: &str) {
        self.add_toast(msg, ToastType::Error);
    }
    pub fn info(&self, msg: &str) {
        self.add_toast(msg, ToastType::Info);
    }
    pub fn warning(&self, msg: &str) {
        self.add_toast(msg, ToastType::Warning);
    }
}
