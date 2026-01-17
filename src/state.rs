use crate::components::common::{ToastMessage, ToastType};
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use web_sys::{SerialPort, Worker};

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
pub struct AppState {
    pub show_settings: Signal<bool>,
    pub show_highlights: Signal<bool>,
    pub show_timestamps: Signal<bool>,
    pub autoscroll: Signal<bool>,
    pub line_ending: Signal<LineEnding>,
    pub highlights: Signal<Vec<Highlight>>,
    pub filter_query: Signal<String>,
    pub match_case: Signal<bool>,
    pub use_regex: Signal<bool>,
    pub invert_filter: Signal<bool>,
    // New Settings
    pub baud_rate: Signal<String>,
    pub data_bits: Signal<&'static str>,
    pub stop_bits: Signal<&'static str>,
    pub parity: Signal<&'static str>,
    pub flow_control: Signal<&'static str>,
    pub rx_line_ending: Signal<LineEnding>,
    pub is_hex_view: Signal<bool>,
    // Serial State
    pub port: Signal<Option<SerialPortWrapper>>,
    pub is_connected: Signal<bool>,
    pub is_simulating: Signal<bool>,
    pub log_worker: Signal<Option<Worker>>,
    pub toasts: Signal<Vec<ToastMessage>>,
}

impl AppState {
    pub fn add_toast(&self, message: &str, type_: ToastType) {
        let mut toasts = self.toasts;
        let id = js_sys::Date::now() as usize;

        toasts.write().push(ToastMessage {
            id,
            message: message.to_string(),
            type_,
        });

        let mut toasts_clone = toasts;
        spawn(async move {
            TimeoutFuture::new(3000).await;
            toasts_clone.write().retain(|t| t.id != id);
        });
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
