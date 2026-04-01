#![feature(duration_millis_float)]

use crate::route::Route;
use dioxus::prelude::*;

pub mod components;
pub mod encoded;
pub mod icons;
pub mod layouts;
pub mod route;
pub mod storage;
pub mod views;

#[cfg(feature = "server")]
pub mod server_state;

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use crate::{server_state::*, storage::*};
        use lupabase::prelude::DatabaseOps;

        DATABASE.try_initialize_storage::<DownloadItem, Vec<DownloadItem>>(vec![])?;

        YT_DLP_LIBRARIES.install_dependencies().await?;

        Ok(dioxus::server::router(App))
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
