use crate::components::ui::CustomInputSelect;
use dioxus::prelude::*;

#[component]
pub fn BaudRatePicker(
    baud_rate: Signal<u32>,
    disabled: bool,
    onchange: EventHandler<u32>,
) -> Element {
    let baud_str = use_signal(move || baud_rate().to_string());

    rsx! {
        div { class: "w-32",
            CustomInputSelect {
                options: vec![
                    "1200",
                    "2400",
                    "4800",
                    "9600",
                    "19200",
                    "38400",
                    "57600",
                    "115200",
                    "230400",
                    "460800",
                    "921600",
                ],
                selected: baud_str,
                onchange: move |val: String| {
                    if let Ok(b) = val.parse::<u32>() {
                        baud_rate.set(b);
                        onchange.call(b);
                    }
                },
                class: "w-full",
                disabled: disabled,
            }
        }
    }
}
