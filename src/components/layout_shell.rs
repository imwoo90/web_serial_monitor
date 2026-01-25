use dioxus::prelude::*;

#[component]
pub fn LayoutShell(
    header: Element,
    input_bar: Element,
    filter_bar: Element,
    content: Element,
    footer: Element,
) -> Element {
    rsx! {
        div { class: "bg-background-dark h-screen w-full font-display text-white selection:bg-primary/30 selection:text-primary overflow-x-auto overflow-y-hidden",
            div { class: "flex flex-col h-full min-w-[600px]",
                {header}
                {input_bar}
                {filter_bar}
                main { class: "flex-1 min-h-0 relative",
                    {content}
                }
                {footer}
            }
        }
    }
}
