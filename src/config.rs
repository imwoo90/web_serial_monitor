/// --- Networking & Buffer Config ---
pub const READ_BUFFER_SIZE: usize = 64 * 1024;
pub const EXPORT_CHUNK_SIZE: u64 = 64 * 1024;
pub const MAX_LINE_BYTES: usize = 256;

/// --- UI Timing & Intervals ---
pub const TOAST_DURATION_MS: u32 = 3000;
pub const WORKER_UPDATE_INTERVAL_MS: u32 = 16;
pub const APP_SUBTITLE: &str = "Monitor v2.1.0";

/// --- Layout & Virtual Scroll ---
pub const LINE_HEIGHT: f64 = 20.0;
pub const HEADER_OFFSET: f64 = 150.0;
pub const TOP_BUFFER: usize = 10;
pub const BOTTOM_BUFFER_EXTRA: usize = 40;
pub const CONSOLE_TOP_PADDING: f64 = 8.0; // 0.5rem
pub const CONSOLE_BOTTOM_PADDING: f64 = 20.0;
pub const VIRTUAL_SCROLL_THRESHOLD: f64 = 10_000_000.0;

pub const HIGHLIGHT_COLORS: &[&str] = &[
    "red", "blue", "yellow", "green", "purple", "orange", "teal", "pink", "indigo", "lime", "cyan",
    "rose", "fuchsia", "amber", "emerald", "sky", "violet",
];
