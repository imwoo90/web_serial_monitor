# 📡 Web Serial Monitor (Built with Rust & Dioxus)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Dioxus](https://img.shields.io/badge/Dioxus-0.6-blue?style=for-the-badge)
![WASM](https://img.shields.io/badge/WebAssembly-purple?style=for-the-badge)
![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)

A high-performance, browser-based Serial Monitor that requires **no installation**. Built with **Rust (Dioxus)** and **WebAssembly**, it provides a desktop-class experience directly in your browser.

Unlike typical web serial tools, this project leverages **OPFS (Origin Private File System)** and **Web Workers** to handle **Gigabyte-scale logs** without freezing the UI, featuring real-time filtering and virtual scrolling.

![Screenshot](https://via.placeholder.com/800x450?text=Web+Serial+Monitor+Screenshot)
*(Screenshot placeholder - Replace with actual screenshot)*

---

## ✨ Key Features

### 🚀 Performance & Core
*   **Web Serial API**: Connect to COM ports / TTY devices directly from Chrome/Edge. No drivers or software installation needed.
*   **High-Performance Logging**: Handles **millions of log lines** seamlessly using **OPFS** (Persistent Storage) and asynchronous stream processing.
*   **Zero-Lag UI**: Implements **Virtual Scrolling** to render only visible items, keeping memory usage low even with massive datasets.
*   **Non-Blocking Filter**: Background worker handles search/filtering efficiently using a **Progressive Scan & Yield** algorithm, ensuring the UI never freezes.

### 🛠️ Advanced Tools
*   **Real-time Filtering**: Filter logs by text, case-sensitivity, or **RegEx**. Supports **Invert Log** logic.
*   **Smart Highlighting**: Assign custom colors to specific keywords (e.g., "Error" -> Red, "Warning" -> Yellow).
*   **Hex View Mode**: Inspect raw binary data in Hexadecimal format.
*   **Simulation Mode**: Built-in generic traffic generator for testing the monitor's performance (1000+ lines/sec load testing).
*   **Log Export**: Download full session logs (GBs) as a file instantly without memory crashes using **Stream API**.

---

## 🏗️ Architecture

This project uses a hybrid architecture to maximize performance in a browser environment.

*   **Main Thread (Rust/Dioxus)**: Handles UI rendering, State Management (Signals), and Serial Port I/O.
*   **Web Worker (JavaScript)**: Manages heavy I/O tasks.
    *   **OPFS**: Writes logs to a virtual file system for persistence.
    *   **Search Engine**: Performs reverse/forward scanning for filtering.
    *   **Throttling**: Batches UI updates (max 20fps) to prevent main thread blocking during high-load data ingestion.

```mermaid
graph TD
    User[User / Serial Device] -->|Data Stream| Main[Main Thread (Rust)]
    Main -->|Virtual DOM| Browser[Browser UI]
    Main -->|PostMessage| Worker[Log Worker (JS)]
    Worker -->|SyncAccessHandle| OPFS[OPFS Storage]
    Worker -->|Filtered View| Main
```

---

## 📦 Getting Started

### Prerequisites
*   **Rust**: Stable toolchain installed.
*   **Dioxus CLI**: `cargo install dioxus-cli`
*   **Wasm32 Target**: `rustup target add wasm32-unknown-unknown`

### Running Locally
```bash
# 1. Clone the repository
git clone https://github.com/your-username/web-serial-monitor.git
cd web-serial-monitor

# 2. Run with Trunk / Dioxus CLI
dx serve --port 8080
```
Open `http://localhost:8080` in a supported browser (Chrome, Edge, Opera).

---

## 📖 Usage Guide

1.  **Connect**: Click the **Connect** button and select your serial device. Set Baud Rate (default 115200).
2.  **View Logs**: Logs will appear automatically. Scroll naturally or enable **Auto-Scroll**.
3.  **Filter**:
    *   Type in the input bar to filter logs instantly.
    *   Use buttons for **Aa** (Case Sensitive), **.* (RegEx)**, or **! (Invert)**.
4.  **Highlight**: Click the **Highlighter Icon** to open the panel. Add keywords (e.g., "Error") to highlight them permanently in the stream.
5.  **Export**: Click the **Date Icon** (top-right) to download the current session log.
6.  **Test**: Click the **Bug Icon** (Test Mode) next to Settings to simulate high-speed serial data.

---

## ⚠️ Browser Compatibility
*   **Required**: Browsers supporting **Web Serial API** and **OPFS (Origin Private File System)**.
    *   ✅ Google Chrome (89+)
    *   ✅ Microsoft Edge (89+)
    *   ✅ Opera
    *   ❌ Firefox (Web Serial not supported yet)
    *   ❌ Safari (Web Serial not supported yet)

---

## 📜 License
This project is licensed under the **MIT License**.
