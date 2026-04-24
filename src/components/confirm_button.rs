use dioxus::prelude::*;

#[component]
pub fn ConfirmButton(
    label: String,
    #[props(default = "Confirm?".to_string())]
    confirm_text: String,
    #[props(default = "Yes".to_string())]
    confirm_label: String,
    #[props(default = "No".to_string())]
    cancel_label: String,
    #[props(default = "danger-button".to_string())]
    class: String,
    on_confirm: EventHandler<MouseEvent>,
) -> Element {
    let mut confirming = use_signal(|| false);

    rsx! {
        if confirming() {
            span {
                class: "confirm-inline",
                style: "color: #b91c1c; font-weight: 600;",
                "{confirm_text}"
            }
            button {
                class: "{class}",
                style: "margin-left: 6px;",
                onclick: move |e| {
                    confirming.set(false);
                    on_confirm.call(e);
                },
                "{confirm_label}"
            }
            button {
                class: "small-button",
                style: "margin-left: 4px;",
                onclick: move |_| {
                    confirming.set(false);
                },
                "{cancel_label}"
            }
        } else {
            button {
                class: "{class}",
                onclick: move |_| {
                    confirming.set(true);
                },
                "{label}"
            }
        }
    }
}
