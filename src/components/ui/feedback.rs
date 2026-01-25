use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ToastMessage {
    pub id: usize,
    pub message: String,
    pub type_: ToastType,
}

#[component]
pub fn ToastContainer(toasts: Signal<Vec<ToastMessage>>) -> Element {
    rsx! {
        div { class: "fixed bottom-5 right-5 flex flex-col gap-2 z-50 pointer-events-none",
            for toast in toasts() {
                {
                    let type_class = match toast.type_ {
                        ToastType::Info => "bg-[#16181a] border border-gray-700 text-gray-300",
                        ToastType::Success => "bg-[#0d1f12] border border-green-900 text-green-400",
                        ToastType::Warning => "bg-[#1f1a0d] border border-yellow-900 text-yellow-400",
                        ToastType::Error => "bg-[#1f0d0d] border border-red-900 text-red-400",
                    };
                    let icon = match toast.type_ {
                        ToastType::Info => "info",
                        ToastType::Success => "check_circle",
                        ToastType::Warning => "warning",
                        ToastType::Error => "error",
                    };

                    rsx! {
                        div {
                            key: "{toast.id}",
                            class: "pointer-events-auto min-w-[200px] max-w-[300px] p-3 rounded-lg shadow-lg text-xs font-bold flex items-center gap-2 animate-in slide-in-from-right-5 fade-in duration-300 {type_class}",
                            span { class: "material-symbols-outlined text-[18px]", "{icon}" }
                            span { "{toast.message}" }
                        }
                    }
                }
            }
        }
    }
}

/// A standard header for panels like Highlights or Settings.
#[component]
pub fn PanelHeader(title: &'static str, subtitle: Option<String>) -> Element {
    rsx! {
        div { class: "flex items-center justify-between border-b border-white/5 pb-2",
            span { class: "text-[11px] font-bold text-gray-500 uppercase tracking-widest",
                "{title}"
            }
            if let Some(sub) = subtitle {
                span { class: "text-[10px] text-gray-600", "{sub}" }
            }
        }
    }
}
