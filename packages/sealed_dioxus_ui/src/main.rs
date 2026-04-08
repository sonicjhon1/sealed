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
    rootcause::hooks::Hooks::new()
        .report_creation_hook(rootcause_backtrace::BacktraceCollector::new_from_env())
        .install()
        .expect("failed to install rootcause's hooks");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
        )
        .with_ansi(true)
        .init();

    let _ = enable_ansi_support::enable_ansi_support();

    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        server_init().await.unwrap();

        info!(
            "Server is currently live at: (http://{})",
            dioxus::cli_config::fullstack_address_or_localhost()
        );

        Ok(dioxus::server::router(App))
    });
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[cfg(feature = "server")]
async fn server_init() -> rootcause::Result<()> {
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

    Ok(())
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
