use crate::components::ui::{ToastMessage, ToastType};
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use web_sys::{ReadableStreamDefaultReader, SerialPort};

#[derive(Clone, Debug)]
pub struct SerialPortWrapper(pub SerialPort);
// Safety: In WASM, we are single-threaded. Dioxus requires Send/Sync for Context, but we know it's local.
unsafe impl Send for SerialPortWrapper {}
unsafe impl Sync for SerialPortWrapper {}
impl PartialEq for SerialPortWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Debug)]
pub struct ReaderWrapper(pub ReadableStreamDefaultReader);
unsafe impl Send for ReaderWrapper {}
unsafe impl Sync for ReaderWrapper {}
impl PartialEq for ReaderWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Highlight {
    pub id: usize,
    pub text: String,
    pub color: &'static str,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum LineEnding {
    #[default]
    None,
    NL,
    CR,
    NLCR,
}

#[derive(Clone, Copy)]
pub struct UIState {
    pub show_settings: Signal<bool>,
    pub show_highlights: Signal<bool>,
    pub show_timestamps: Signal<bool>,
    pub autoscroll: Signal<bool>,
    pub is_hex_view: Signal<bool>,
}

#[derive(Clone, Copy)]
pub struct SerialSettings {
    pub baud_rate: Signal<String>,
    pub data_bits: Signal<&'static str>,
    pub stop_bits: Signal<&'static str>,
    pub parity: Signal<&'static str>,
    pub flow_control: Signal<&'static str>,
    pub rx_line_ending: Signal<LineEnding>,
    pub tx_line_ending: Signal<LineEnding>,
    pub tx_local_echo: Signal<bool>,
}

#[derive(Clone, Copy)]
pub struct ConnectionState {
    pub port: Signal<Option<SerialPortWrapper>>,
    pub reader: Signal<Option<ReaderWrapper>>,
    pub is_connected: Signal<bool>,
    pub is_simulating: Signal<bool>,
    pub log_worker: Signal<Option<web_sys::Worker>>,
}

#[derive(Clone, Copy)]
pub struct LogState {
    pub total_lines: Signal<usize>,
    pub visible_logs: Signal<Vec<String>>,
    pub filter_query: Signal<String>,
    pub match_case: Signal<bool>,
    pub use_regex: Signal<bool>,
    pub invert_filter: Signal<bool>,
    pub highlights: Signal<Vec<Highlight>>,
    pub toasts: Signal<Vec<ToastMessage>>,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub ui: UIState,
    pub serial: SerialSettings,
    pub conn: ConnectionState,
    pub log: LogState,
}

impl UIState {
    pub fn toggle_settings(&self) {
        let mut s = self.show_settings;
        s.set(!s());
    }
    pub fn toggle_highlights(&self) {
        let mut s = self.show_highlights;
        s.set(!s());
    }
    pub fn toggle_timestamps(&self) {
        let mut s = self.show_timestamps;
        s.set(!s());
    }
    pub fn toggle_autoscroll(&self) {
        let mut s = self.autoscroll;
        s.set(!s());
    }
    pub fn set_autoscroll(&self, value: bool) {
        let mut s = self.autoscroll;
        s.set(value);
    }
    pub fn toggle_hex_view(&self) {
        let mut s = self.is_hex_view;
        s.set(!s());
    }
}

impl SerialSettings {
    pub fn set_baud_rate(&self, rate: String) {
        let mut b = self.baud_rate;
        b.set(rate);
    }
    pub fn set_data_bits(&self, bits: &'static str) {
        let mut s = self.data_bits;
        s.set(bits);
    }
    pub fn set_stop_bits(&self, bits: &'static str) {
        let mut s = self.stop_bits;
        s.set(bits);
    }
    pub fn set_parity(&self, p: &'static str) {
        let mut s = self.parity;
        s.set(p);
    }
    pub fn set_flow_control(&self, f: &'static str) {
        let mut s = self.flow_control;
        s.set(f);
    }
}

impl ConnectionState {
    pub fn set_connected(
        &self,
        port: Option<SerialPort>,
        reader: Option<ReadableStreamDefaultReader>,
    ) {
        let mut p = self.port;
        let mut r = self.reader;
        let mut c = self.is_connected;
        p.set(port.map(SerialPortWrapper));
        r.set(reader.map(ReaderWrapper));
        c.set(p.read().is_some());
    }

    pub fn set_simulating(&self, simulating: bool) {
        let mut s = self.is_simulating;
        s.set(simulating);
    }
}

impl LogState {
    pub fn clear(&self) {
        let mut t = self.total_lines;
        let mut v = self.visible_logs;
        t.set(0);
        v.set(Vec::new());
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
        let mut h = self.highlights;
        let mut list = h.read().clone();
        let next_id = list.iter().map(|h| h.id).max().unwrap_or(0) + 1;
        list.push(Highlight {
            id: next_id,
            text,
            color,
        });
        h.set(list);
    }

    pub fn remove_highlight(&self, id: usize) {
        let mut h = self.highlights;
        let mut list = h.read().clone();
        list.retain(|h| h.id != id);
        h.set(list);
    }
}

impl AppState {
    pub fn add_toast(&self, message: &str, type_: ToastType) {
        self.log.add_toast(message, type_);
    }

    pub fn clear_logs(&self) {
        self.log.clear();
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
}

pub const HIGHLIGHT_COLORS: &[&str] = &[
    "red", "blue", "yellow", "green", "purple", "orange", "teal", "pink", "indigo", "lime", "cyan",
    "rose", "fuchsia", "amber", "emerald", "sky", "violet",
];
