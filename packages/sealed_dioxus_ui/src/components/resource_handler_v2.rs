use dioxus::prelude::*;

#[component]
pub fn ResourceHandlerOptionV2<T: Clone + 'static>(
    #[props] resource: Resource<Option<T>>,
    #[props(into)] okay: Callback<T, Element>,
) -> Element {
    match &*resource.read() {
        Some(Some(ok)) => okay.call(ok.clone()),
        _ => rsx! {},
    }
}
