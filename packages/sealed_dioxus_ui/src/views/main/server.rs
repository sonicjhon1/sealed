use crate::storage::*;
use dioxus::{
    fullstack::{CborEncoding, WebSocketOptions, Websocket},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[server(
    input = Cbor,
    output = Cbor
)]
pub async fn try_read_clipboard_text() -> Result<String> {
    let clipboard_text = arboard::Clipboard::new()?.get_text().unwrap_or_default();

    let clipboard_text_sanitized = clipboard_text
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string();

    Ok(clipboard_text_sanitized)
}

#[server(
    input = Cbor,
    output = Cbor
)]
pub async fn try_fetch_url_info(download_url: String) -> Result<DownloadUrlInfo> {
    use crate::server_state::*;
    use yt_dlp::{download::SpeedProfile, model::caption::Subtitle, prelude::*};

    let downloader = Downloader::builder(YT_DLP_LIBRARIES.clone(), std::env::temp_dir())
        .with_speed_profile(SpeedProfile::Aggressive)
        .build()
        .await?;

    // use dioxus::fullstack::Encoding;
    // let download_info_testing_path = CURRENT_FILES_DIR.join("download_info_file_testing.cbor");
    // let download_info = match std::fs::read(&download_info_testing_path) {
    //     Ok(cbor) if let Some(download_info) = CborEncoding::decode(cbor.clone().into()) => {
    //         download_info
    //     }
    //     _ => {
    //         let download_info = downloader.fetch_video_infos_fresh(&download_url).await?;
    //         let mut download_info_file_cbor = vec![];
    //         CborEncoding::encode(&download_info, &mut download_info_file_cbor);
    //         std::fs::write(download_info_testing_path, download_info_file_cbor)?;

    //         download_info
    //     }
    // };
    let download_info = downloader.fetch_video_infos_fresh(&download_url).await?;

    let uploader = [
        &download_info.uploader,
        &download_info.channel,
        &download_info.uploader_id,
        &download_info.channel_id,
    ]
    .into_iter()
    .find_map(|u| u.clone());

    let thumbnail = if let Some(ref t) = download_info.thumbnail {
        Some(reqwest::get(t).await?.bytes().await?.to_vec())
    } else {
        None
    };

    let mut formats = download_info.clone().formats;
    formats.sort_by(|a, b| {
        use yt_dlp::{VideoSelection, model::format::Format};

        let rank = |f: &Format| {
            let has_video = f.codec_info.video_codec.as_deref().is_some();
            let has_audio = f.codec_info.audio_codec.as_deref().is_some();

            match (has_video, has_audio) {
                (true, false) => 3, // video-only
                (false, true) => 2, // audio-only
                (true, true) => 1,  // muxed
                _ => 0,
            }
        };

        rank(b)
            .cmp(&rank(a))
            .then_with(|| download_info.compare_video_formats(b, a))
            .then_with(|| {
                if a.codec_info.audio_codec.as_deref().is_some()
                    && b.codec_info.audio_codec.as_deref().is_some()
                {
                    download_info.compare_audio_formats(b, a)
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .then_with(|| {
                b.rates_info
                    .total_rate
                    .unwrap_or_default()
                    .cmp(&a.rates_info.total_rate.unwrap_or_default())
            })
    });

    let formats = formats
        .into_iter()
        .map(|f| DownloadUrlInfoFormat {
            format: f.format,
            format_id: f.format_id,
            extension: f.container.map(|c| c.to_string()),
            video_codec: f.codec_info.video_codec,
            audio_codec: f.codec_info.audio_codec,
            total_size: [f.file_info.filesize, f.file_info.filesize_approx]
                .into_iter()
                .find_map(|u| u)
                .map(|v| human_bytes::human_bytes(v as f64)),
            total_bitrate: f
                .rates_info
                .total_rate
                .map(|v| human_bytes::human_bytes(v * 1000.))
                .map(|s| format!("{s}/s")),
        })
        .collect();

    let mut subtitles = download_info
        .subtitles
        .into_iter()
        .chain(download_info.automatic_captions.into_iter().map(|(k, s)| {
            (
                k.clone(),
                s.into_iter()
                    .map(|s| Subtitle::from_automatic_caption(&s, k.clone()))
                    .collect(),
            )
        }))
        .filter_map(|(k, s)| {
            let s_first = s.first()?;

            if k == "live_chat" {
                return Some(DownloadUrlInfoSubtitle::LiveChat {
                    extensions: s_first.extension.to_string(),
                });
            };

            let extensions = s.iter().map(|s| s.extension.to_string()).collect();
            let language_code = s_first.language_code.clone().unwrap_or(k.clone());
            let language_name = s_first.language_name.clone().unwrap_or_default();

            Some(DownloadUrlInfoSubtitle::Subtitle {
                id: k,
                language_code,
                language_name,
                extensions,
                is_automatic: s_first.is_automatic,
            })
        })
        .collect::<Vec<_>>();

    subtitles.sort_by(|a, b| match (a, b) {
        (DownloadUrlInfoSubtitle::LiveChat { .. }, _) => std::cmp::Ordering::Less,
        (_, DownloadUrlInfoSubtitle::LiveChat { .. }) => std::cmp::Ordering::Greater,
        (
            DownloadUrlInfoSubtitle::Subtitle {
                language_name: a_language_name,
                ..
            },
            DownloadUrlInfoSubtitle::Subtitle {
                language_name: b_language_name,
                ..
            },
        ) => a_language_name.cmp(b_language_name),
    });

    Ok(DownloadUrlInfo {
        id: download_info.id,
        url: download_info.webpage_url.unwrap_or(download_url),
        title: download_info.title,
        description: download_info.description,
        uploader,
        duration: download_info.duration,
        thumbnail,
        formats,
        subtitles,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MainWSClient {
    FetchDownloadItems {
        filter_query: String,
    },
    NewDownload {
        download_url_info: Box<DownloadUrlInfo>,
        audio_video_format_ids: Vec<String>,
        storyboard_format_ids: HashSet<String>,
        subtitle_ids: HashSet<String>,
        thumbnail: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MainWSServer {
    DownloadItems(Vec<DownloadItem>),
}

#[get("/api/main_websocket")]
pub async fn main_websocket(
    options: WebSocketOptions,
) -> Result<Websocket<MainWSClient, MainWSServer, CborEncoding>> {
    use crate::server_state::*;

    Ok(options.try_on_upgrade(move |mut ws| async move {
        let mut main_states = MainStates::initialize()?;

        ws.send(MainWSServer::DownloadItems(
            main_states.download_items.clone(),
        ))
        .await?;

        loop {
            match ws.recv().await {
                Ok(MainWSClient::FetchDownloadItems { filter_query }) => {
                    main_states.filter_query = filter_query;

                    ws.send(MainWSServer::DownloadItems(
                        main_states.filtered_download_items(),
                    ))
                    .await?;
                }
                Ok(MainWSClient::NewDownload {
                    download_url_info,
                    audio_video_format_ids,
                    storyboard_format_ids,
                    subtitle_ids,
                    thumbnail,
                }) => {
                    let libraries = crate::server_state::YT_DLP_LIBRARIES.clone();

                    let mut download_command = std::process::Command::new(&libraries.youtube);

                    if audio_video_format_ids.is_empty() {
                        download_command.arg("--skip-download");
                    }

                    let format_ids = [audio_video_format_ids.join("+")]
                        .into_iter()
                        .chain(storyboard_format_ids.into_iter())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(",");
                    if !format_ids.is_empty() {
                        download_command.arg("-f").arg(&format_ids);
                    }

                    let subtitle_ids = subtitle_ids.into_iter().collect::<Vec<_>>().join(",");
                    if !subtitle_ids.is_empty() {
                        download_command
                            .args(["--write-subs", "--write-auto-subs", "--sub-langs"])
                            .arg(&subtitle_ids);
                    }

                    if thumbnail {
                        download_command.arg("--write-thumbnail");
                    }

                    if download_command.get_args().len() != 0 {
                        download_command
                            .arg("-o")
                            .arg(
                                CURRENT_FILES_DIR
                                    .join("output")
                                    .join("%(title).200B [%(id)s].%(ext)s"),
                            )
                            .arg(&download_url_info.url);

                        debug!(
                            "Running yt-dlp with: ({})",
                            download_command
                                .get_args()
                                .collect::<Vec<_>>()
                                .join(std::ffi::OsStr::new(" "))
                                .display()
                        );

                        let mut download_item = DownloadItem {
                            id: format!(
                                "{}-{}-{}",
                                download_url_info.id,
                                download_url_info.url,
                                format_ids + &subtitle_ids,
                            ),
                            url: download_url_info.url,
                            title: download_url_info.title,
                            description: download_url_info.description,
                            uploader: download_url_info.uploader,
                            duration: download_url_info.duration,
                            item_date_added: chrono::Utc::now(),
                            item_state: DownloadItemState::Pending,
                        };
                        main_states.insert_download_item(download_item.clone())?;
                        ws.send(MainWSServer::DownloadItems(
                            main_states.filtered_download_items(),
                        ))
                        .await?;

                        match download_command.status() {
                            Ok(status) if status.success() => {
                                download_item.item_state = DownloadItemState::Ok {
                                    date_finished: chrono::Utc::now(),
                                };
                            }
                            Ok(_) => {
                                download_item.item_state = DownloadItemState::Error;
                            }
                            Err(e) => {
                                download_item.item_state = DownloadItemState::Error;
                            }
                        };
                        main_states.update_download_item(download_item)?;
                        ws.send(MainWSServer::DownloadItems(
                            main_states.filtered_download_items(),
                        ))
                        .await?;
                    }
                }
                Err(e) => break Err(e)?,
            }
        }
    }))
}
