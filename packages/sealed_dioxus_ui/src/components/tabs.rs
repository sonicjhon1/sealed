use std::fmt::Display;

use dioxus::prelude::*;

#[component]
pub fn TabDropdownToggle<T: Copy + PartialEq + 'static>(
    #[props] active_tab: WriteSignal<Option<T>>,
    tab: T,
    #[props(into)] tab_name: String,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "dropdown",
            class: if active_tab() == Some(tab) { "dropdown-open" } else { "dropdown-close" },
            div {
                class: "tabs tabs-border",
                onclick: move |_| {
                    if active_tab() == Some(tab) {
                        active_tab.set(None);
                    } else {
                        active_tab.set(Some(tab))
                    }
                },
                div {
                    class: "tab text-base-content",
                    class: if active_tab() == Some(tab) { "tab-active" },
                    tabindex: "0",
                    role: "tab",
                    "{tab_name}"
                }
            }
            ul {
                class: "menu dropdown-content bg-base-100 rounded-box z-1 w-52 p-2 shadow-sm",
                tabindex: "-1",
                {children}
            }
        }
    }
}

#[component]
pub fn TabRequired<T: Clone + PartialEq + 'static>(
    #[props] active_tab: WriteSignal<T>,
    tab: T,
    #[props(into)] tab_name: String,
) -> Element {
    rsx! {
        div {
            class: "tabs tabs-border",
            onclick: {
                let tab = tab.clone();
                move |_| active_tab.set(tab.clone())
            },
            div {
                class: "tab text-base-content",
                class: if active_tab() == tab { "tab-active" },
                tabindex: "0",
                role: "tab",
                "{tab_name}"
            }
        }
    }
}

#[component]
pub fn TabButtonRequired<T: Clone + PartialEq + Display + 'static>(
    #[props] active_tab: WriteSignal<T>,
    tab: T,
) -> Element {
    rsx! {
        div {
            class: "btn",
            class: if active_tab == tab { "btn-neutral" } else { "btn-ghost" },
            onclick: move |_| { active_tab.set(tab.clone()) },
            "{tab}"
        }
    }
}
