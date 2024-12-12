use crate::state::AppStateExt;
use dioxus::prelude::*;
use streak::filtering::FilterByStatus;

#[derive(Debug, Clone, Props, PartialEq)]
struct FilterButtonProps {
    button_text: String,
    button_status: FilterByStatus,
}

#[component]
pub fn FilterButton(props: FilterButtonProps) -> Element {
    let mut state = use_context::<Signal<dyn AppStateExt>>();
    let filter_by = state.read().filter_by.clone();

    rsx! {
        div {
            label {
                class: "transition cursor-pointer inline-block rounded-md px-4 py-2 text-sm text-purple-500 dark:text-purple-900 hover:text-purple-700 focus:relative has-[:checked]:border-purple-500 has-[:checked]:bg-purple-500 has-[:checked]:text-pink-100 dark:has-[:checked]:border-purple-900 dark:has-[:checked]:bg-purple-900",
                input {
                    class: "sr-only",
                    name: "filter",
                    r#type: "radio",
                    checked: filter_by == props.button_status.clone(),
                    onclick: move |e| { state.write().filter_by = props.button_status.clone(); },
                }
                p {
                    class: "text-sm font-medium",
                    "{props.button_text}",
                }
            }
        }

    }
}
