use crate::storage::{CURRENT_FILES_DIR, DATABASE, DownloadItem};
use dioxus::{
    CapturedError,
    fullstack::{TypedWebsocket, WebSocketOptions, Websocket},
    prelude::*,
};
use lupabase::prelude::*;
use nucleo_matcher::{
    Config, Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};
use std::sync::LazyLock;

use std::result::Result;

pub static YT_DLP_LIBRARIES: LazyLock<yt_dlp::client::Libraries> = LazyLock::new(|| {
    yt_dlp::client::Libraries::new(
        CURRENT_FILES_DIR
            .join("yt_dlp")
            .with_extension(std::env::consts::EXE_EXTENSION),
        CURRENT_FILES_DIR
            .join("ffmpeg")
            .with_extension(std::env::consts::EXE_EXTENSION),
    )
});

pub struct MainStates {
    pub download_items: Vec<DownloadItem>,
    pub filter_query: String,
}

impl MainStates {
    pub fn initialize() -> Result<Self, CapturedError> {
        let download_items = DATABASE.get_all::<DownloadItem>()?;

        Ok(Self {
            download_items,
            filter_query: String::new(),
        })
    }

    pub fn insert_download_item(
        &mut self,
        download_item: DownloadItem,
    ) -> Result<(), CapturedError> {
        DATABASE.insert(download_item)?;
        self.download_items = DATABASE.get_all::<DownloadItem>()?;

        Ok(())
    }

    pub fn update_download_item(
        &mut self,
        download_item: DownloadItem,
    ) -> Result<(), CapturedError> {
        DATABASE.update(download_item)?;
        self.download_items = DATABASE.get_all::<DownloadItem>()?;

        Ok(())
    }

    pub fn filtered_download_items(&self) -> Vec<DownloadItem> {
        let pattern = Pattern::parse(&self.filter_query, CaseMatching::Ignore, Normalization::Smart);
        let mut matcher = Matcher::new(Config::DEFAULT);
        let mut haystack_buffer = vec![];

        let download_items = self.download_items.clone();
        let mut download_items_filtered = Vec::with_capacity(download_items.len());

        for download_item in download_items {
            let utf_str = Utf32Str::new(&download_item.title, &mut haystack_buffer);

            if let Some(matched_score) = pattern.score(utf_str, &mut matcher) {
                download_items_filtered.push((matched_score, download_item));
            }
        }

        download_items_filtered.sort_unstable_by(|(a_score, a_item), (b_score, b_item)| {
            b_score
                .cmp(a_score)
                .then_with(|| b_item.item_date_added.cmp(&a_item.item_date_added))
        });

        download_items_filtered
            .into_iter()
            .map(|(_, item)| item)
            .collect()
    }
}

pub trait WebSocketOptionsExt {
    fn try_on_upgrade<F, Fut, In: 'static, Out: 'static, Enc: 'static>(
        self,
        callback: F,
    ) -> Websocket<In, Out, Enc>
    where
        F: FnOnce(TypedWebsocket<In, Out, Enc>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), dioxus::CapturedError>> + 'static;
}

impl WebSocketOptionsExt for WebSocketOptions {
    fn try_on_upgrade<F, Fut, In: 'static, Out: 'static, Enc: 'static>(
        self,
        callback: F,
    ) -> Websocket<In, Out, Enc>
    where
        F: FnOnce(TypedWebsocket<In, Out, Enc>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), dioxus::CapturedError>> + 'static, {
        self.on_upgrade(move |ws| async move {
            match callback(ws).await {
                Ok(()) => debug!(
                    "Websocket [In: ({}), Out: ({}), Enc: ({})] closed",
                    type_name_pretty::<In>(),
                    type_name_pretty::<Out>(),
                    type_name_pretty::<Enc>()
                ),
                Err(e) => warn!(
                    "Websocket [In: ({}), Out: ({}), Enc: ({})] encountered an error: {e}",
                    type_name_pretty::<In>(),
                    type_name_pretty::<Out>(),
                    type_name_pretty::<Enc>()
                ),
            }
        })
    }
}

pub fn type_name_pretty<T>() -> &'static str {
    let name = std::any::type_name::<T>();

    return name.split("::").last().unwrap_or(name);
}
