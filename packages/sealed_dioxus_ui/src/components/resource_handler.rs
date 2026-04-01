use crate::icons::{Icon, lucide};
use dioxus::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum UI {
    Simple,
    #[default]
    Button,
    None,
}

#[component]
pub fn ResourceHandler<T: Clone + 'static, E: Clone + std::fmt::Display + 'static>(
    #[props] resource: Resource<Result<T, E>>,
    #[props(into)] loading: String,
    #[props(optional, into, default = UI::Button)] ui: UI,
    #[props(into)] okay: Callback<T, Element>,
) -> Element {
    match &*resource.read() {
        Some(Ok(ok)) => okay.call(ok.clone()),
        Some(Err(error)) => match ui {
            UI::Simple => rsx! {
                ErrorSimple { error }
            },
            UI::Button => rsx! {
                ErrorButton { error }
            },
            UI::None => rsx! {},
        },
        None => match ui {
            UI::Simple => rsx! {
                LoadingSimple { loading }
            },
            UI::Button => rsx! {
                LoadingButton { loading }
            },
            UI::None => rsx! {},
        },
    }
}

#[component]
pub fn ResourceHandlerIgnored<T: Clone + 'static, E: std::fmt::Display + 'static>(
    #[props] resource: Resource<Result<T, E>>,
    #[props(into)] loading: String,
    #[props(optional, into, default = UI::Button)] ui: UI,
    children: Element,
) -> Element {
    match &*resource.read() {
        Some(Ok(_)) => children,
        Some(Err(error)) => match ui {
            UI::Simple => rsx! {
                ErrorSimple { error }
            },
            UI::Button => rsx! {
                ErrorButton { error }
            },
            UI::None => rsx! {},
        },
        None => match ui {
            UI::Simple => rsx! {
                LoadingSimple { loading }
            },
            UI::Button => rsx! {
                LoadingButton { loading }
            },
            UI::None => rsx! {},
        },
    }
}

#[component]
pub fn ButtonWrapper(
    #[props]
    #[props(optional, into)]
    children: Option<Element>,
) -> Element {
    rsx! {
        div { class: "flex flex-1 justify-center items-center", {children} }
    }
}

#[component]
pub fn ErrorButton(
    #[props]
    #[props(into)]
    error: String,
) -> Element {
    rsx! {
        ButtonWrapper {
            button { class: "btn btn-soft btn-error",
                Icon { class: "text-current", data: lucide::CircleX }
                "Err: [{error}]"
            }
        }
    }
}

#[component]
pub fn ErrorSimple(
    #[props]
    #[props(into)]
    error: String,
) -> Element {
    rsx! {
        span { class: "text-error mr-2", "Err: [{error}]" }
    }
}

#[component]
pub fn LoadingButton(
    #[props]
    #[props(into)]
    loading: String,
) -> Element {
    rsx! {
        ButtonWrapper {
            button { class: "btn btn-soft btn-info",
                span { class: "loading loading-spinner" }
                {loading}
            }
        }
    }
}

#[component]
pub fn LoadingSimple(
    #[props]
    #[props(into)]
    loading: String,
) -> Element {
    rsx! {
        span { class: "loading loading-spinner loading-xs text-primary mr-2" }
        span { {loading} }
    }
}
