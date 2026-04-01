use crate::route::Route;
use dioxus::prelude::*;

const TAILWIND_CSS: Asset = asset!(
    "/assets/tailwind.css",
    AssetOptions::css().with_minify(true).with_preload(true)
);

#[component]
pub fn BaseLayout() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        div { class: "h-svh flex flex-col p-2 gap-2", Outlet::<Route> {} }
    }
}
