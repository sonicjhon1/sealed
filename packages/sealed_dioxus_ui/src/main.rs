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

        if YT_DLP_LIBRARIES.ffmpeg.exists() && YT_DLP_LIBRARIES.youtube.exists() {
            if let Ok(metadata) = YT_DLP_LIBRARIES.youtube.metadata()
                && let Ok(modified) = metadata.modified()
                && let Ok(elapsed) = modified.elapsed()
            {
                if elapsed >= std::time::Duration::from_hours(12) {
                    info!(
                        "Updating yt-dlp to latest nightly at: ({})",
                        std::path::absolute(&YT_DLP_LIBRARIES.youtube)?.display()
                    );

                    if !std::process::Command::new(&YT_DLP_LIBRARIES.youtube)
                        .args(["--update", "--update-to", "nightly"])
                        .status()?
                        .success()
                    {
                        error!("Failed to update yt-dlp to latest nightly");
                    }
                }
            } else {
                warn!("Failed to check if yt-dlp needs an update");
            };
        } else {
            info!(
                "Installing ffmpeg at: ({}) and yt-dlp at: ({})",
                std::path::absolute(&YT_DLP_LIBRARIES.ffmpeg)?.display(),
                std::path::absolute(&YT_DLP_LIBRARIES.youtube)?.display()
            );

            YT_DLP_LIBRARIES.install_dependencies().await?;

            if !std::process::Command::new(&YT_DLP_LIBRARIES.youtube)
                .args(["--update", "--update-to", "nightly"])
                .status()?
                .success()
            {
                error!("Failed to update yt-dlp to latest nightly");
            }
        }

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
