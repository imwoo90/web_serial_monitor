use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Terminal;

    #[wasm_bindgen(constructor)]
    pub fn new(options: &JsValue) -> Terminal;

    #[wasm_bindgen(method, js_name = open)]
    pub fn open(this: &Terminal, parent: &web_sys::HtmlElement);

    #[wasm_bindgen(method, js_name = write)]
    pub fn write(this: &Terminal, data: &str);

    #[wasm_bindgen(method, js_name = write)]
    pub fn write_chunk(this: &Terminal, data: &js_sys::Uint8Array);

    #[wasm_bindgen(method, js_name = loadAddon)]
    pub fn load_addon(this: &Terminal, addon: &JsValue);

    #[wasm_bindgen(js_namespace = ["FitAddon"], js_name = "FitAddon")]
    pub type XtermFitAddon;

    #[wasm_bindgen(constructor, js_namespace = ["FitAddon"], js_class = "FitAddon")]
    pub fn new_fit() -> XtermFitAddon;

    #[wasm_bindgen(method, js_name = fit)]
    pub fn fit(this: &XtermFitAddon);

    // --- New Bindings ---

    #[wasm_bindgen(method, js_name = onData)]
    pub fn on_data(this: &Terminal, callback: &js_sys::Function) -> js_sys::Object; // Returns IDisposable

    #[wasm_bindgen(method, js_name = clear)]
    pub fn clear(this: &Terminal);

    #[wasm_bindgen(method, js_name = scrollToBottom)]
    pub fn scroll_to_bottom(this: &Terminal);

    #[wasm_bindgen(method, js_name = dispose)]
    pub fn dispose(this: &Terminal);

    #[wasm_bindgen(method, js_name = onScroll)]
    pub fn on_scroll(this: &Terminal, callback: &js_sys::Function) -> js_sys::Object; // Returns IDisposable

    #[wasm_bindgen(method, getter)]
    pub fn buffer(this: &Terminal) -> TerminalBufferNamespace;

    #[wasm_bindgen(method, getter)]
    pub fn options(this: &Terminal) -> TerminalOptions;
}

#[wasm_bindgen]
extern "C" {
    pub type TerminalBufferNamespace;

    #[wasm_bindgen(method, getter)]
    pub fn active(this: &TerminalBufferNamespace) -> TerminalBuffer;
}

#[wasm_bindgen]
extern "C" {
    pub type TerminalBuffer;

    #[wasm_bindgen(method, getter, js_name = length)]
    pub fn length(this: &TerminalBuffer) -> u32;

    #[wasm_bindgen(method, getter, js_name = viewportY)]
    pub fn viewport_y(this: &TerminalBuffer) -> i32;

    #[wasm_bindgen(method, getter, js_name = baseY)]
    pub fn base_y(this: &TerminalBuffer) -> i32;
}

#[wasm_bindgen]
extern "C" {
    pub type TerminalOptions;

    #[wasm_bindgen(method, setter = scrollback)]
    pub fn set_scrollback(this: &TerminalOptions, value: u32);

    #[wasm_bindgen(method, setter = fontSize)]
    pub fn set_font_size(this: &TerminalOptions, value: u32);

    #[wasm_bindgen(method, setter = scrollOnData)]
    pub fn set_scroll_on_data(this: &TerminalOptions, value: bool);

    #[wasm_bindgen(method, setter = scrollOnUserInput)]
    pub fn set_scroll_on_user_input(this: &TerminalOptions, value: bool);
}
