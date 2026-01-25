use crate::components::common::CustomInputSelect;
use dioxus::prelude::*;

#[component]
pub fn BaudRatePicker(
    baud_rate: Signal<String>,
    disabled: bool,
    onchange: EventHandler<String>,
) -> Element {
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
                selected: baud_rate,
                onchange: move |val| onchange.call(val),
                class: "w-full",
                disabled: disabled,
            }
        }
    }
}
