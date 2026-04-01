use std::collections::HashSet;

use chrono::{DateTime, Utc};
use lupabase::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DownloadItemState {
    Pending,
    Downloading { progress: u8 },
    Error,
    Ok { date_finished: DateTime<Utc> },
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub uploader: Option<String>,
    pub duration: Option<i64>,
    pub item_date_added: DateTime<Utc>,
    pub item_state: DownloadItemState,
}

impl DatabaseRecord for DownloadItem {
    type Unique = String;

    fn unique_value(&self) -> Self::Unique { self.id.clone() }
}

impl DatabaseRecordPartitioned for DownloadItem {
    const PARTITION: &str = "download_items";
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct DownloadUrlInfo {
    pub id: String,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub uploader: Option<String>,
    pub duration: Option<i64>,
    #[serde(with = "serde_bytes")]
    pub thumbnail: Option<Vec<u8>>,
    pub formats: Vec<DownloadUrlInfoFormat>,
    pub subtitles: Vec<DownloadUrlInfoSubtitle>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct DownloadUrlInfoFormat {
    pub format: String,
    pub format_id: String,
    pub extension: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub total_size: Option<String>,
    pub total_bitrate: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DownloadUrlInfoSubtitle {
    Subtitle {
        id: String,
        language_code: String,
        language_name: String,
        extensions: Vec<String>,
        is_automatic: bool,
    },
    LiveChat {
        extensions: String,
    },
}

impl DownloadUrlInfoSubtitle {
    pub fn id(&self) -> &str {
        match self {
            DownloadUrlInfoSubtitle::Subtitle { id, .. } => id,
            DownloadUrlInfoSubtitle::LiveChat { .. } => "live_chat",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DownloadUrlSelection {
    pub audio_video_format_id: Option<AudioVideoFormatId>,
    pub storyboard_format_ids: HashSet<String>,
    pub subtitle_ids: HashSet<String>,
    pub thumbnail: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AudioVideoFormatId {
    AudioOnly(String),
    VideoOnly(String),
    VideoAudio(String),
    MixVideoAudio(String, String),
}

impl AudioVideoFormatId {
    pub fn contains(&self, other: &Self) -> bool {
        match self {
            AudioVideoFormatId::AudioOnly(c_a_id) => {
                matches!(other, AudioVideoFormatId::AudioOnly(o_a_id)
                | AudioVideoFormatId::MixVideoAudio(_, o_a_id) if c_a_id == o_a_id)
            }
            AudioVideoFormatId::VideoOnly(c_v_id) => {
                matches!(other, AudioVideoFormatId::VideoOnly(o_v_id)
                | AudioVideoFormatId::MixVideoAudio(o_v_id, _) if c_v_id == o_v_id)
            }
            AudioVideoFormatId::VideoAudio(c_va_id) => {
                matches!(other, AudioVideoFormatId::VideoAudio(o_va_id) if c_va_id == o_va_id)
            }
            AudioVideoFormatId::MixVideoAudio(c_v_id, c_a_id) => {
                matches!(other, AudioVideoFormatId::MixVideoAudio(o_v_id, o_a_id) if (c_v_id == o_v_id && c_a_id == o_a_id))
                    || matches!(other, AudioVideoFormatId::AudioOnly(o_a_id)
                | AudioVideoFormatId::MixVideoAudio(_, o_a_id) if c_a_id == o_a_id)
                    || matches!(other, AudioVideoFormatId::VideoOnly(o_v_id)
                | AudioVideoFormatId::MixVideoAudio(o_v_id, _) if c_v_id == o_v_id)
            }
        }
    }

    pub fn as_ids(&self) -> Vec<String> {
        match self.clone() {
            AudioVideoFormatId::AudioOnly(id)
            | AudioVideoFormatId::VideoOnly(id)
            | AudioVideoFormatId::VideoAudio(id) => vec![id],
            AudioVideoFormatId::MixVideoAudio(id_v, id_a) => vec![id_v, id_a],
        }
    }
}

pub fn optional_audio_video_format_id_contains(
    option: &Option<AudioVideoFormatId>,
    other: &AudioVideoFormatId,
) -> bool {
    let Some(s) = option else {
        return false;
    };

    return s.contains(other);
}
