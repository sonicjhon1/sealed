use crate::{
    components::tabs::TabButtonRequired,
    icons::{Icon, lucide},
    storage::{
        AudioVideoFormatId, DownloadItemState, DownloadUrlInfoSubtitle, DownloadUrlSelection,
        optional_audio_video_format_id_contains,
    },
    views::main::server::{
        MainWSClient, MainWSServer, main_websocket, try_fetch_url_info, try_read_clipboard_text,
    },
};
use base64::prelude::*;
use derive_more::Display;
use dioxus::{
    fullstack::{WebSocketOptions, use_websocket},
    prelude::*,
};

pub const KEMONOMIMI_CHAN: Asset = asset!(
    "/assets/kemonomimi_chan/af2c5d1fc588af7745b942944af1755e.jpg",
    AssetOptions::image()
        .with_format(ImageFormat::Avif)
        .with_preload(true)
);

#[component]
pub fn MainPage() -> Element {
    let active_section = use_signal(MainPageSection::default);

    rsx! {
        div { class: "flex flex-1 gap-2",
            div { class: "flex basis-3xs rounded-lg p-2 bg-base-200",
                div { class: "flex flex-1 flex-col gap-2",
                    div { class: "card overflow-clip rounded-box shadow-md transition-all hover:brightness-50",
                        a { href: "https://www.pixiv.net/en/artworks/68132875",
                            img { src: KEMONOMIMI_CHAN, alt: "kemonomimi-chan" }
                        }
                    }
                    SideBar { active_section }
                }
            }
            div { class: "flex flex-1 rounded-lg p-2",
                match active_section() {
                    MainPageSection::DownloadQueue => rsx! {
                        MainBarDownloadQueue {}
                    },
                    MainPageSection::Downloads => rsx! {
                        MainBarNotImplementedYet {}
                    },
                    MainPageSection::CustomCommand => rsx! {
                        MainBarNotImplementedYet {}
                    },
                    MainPageSection::Settings => rsx! {
                        MainBarNotImplementedYet {}
                    },
                    MainPageSection::Sponsor => rsx! {
                        MainBarNotImplementedYet {}
                    },
                    MainPageSection::Troubleshooting => rsx! {
                        MainBarNotImplementedYet {}
                    },
                    MainPageSection::About => rsx! {
                        MainBarNotImplementedYet {}
                    },
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Display)]
pub enum MainPageSection {
    #[default]
    #[display("Download queue")]
    DownloadQueue,
    Downloads,
    #[display("Custom command")]
    CustomCommand,
    Settings,
    Sponsor,
    Troubleshooting,
    About,
}

#[component]
pub fn SideBar(#[props] active_section: WriteSignal<MainPageSection>) -> Element {
    rsx! {
        div { class: "flex flex-1 flex-col",
            div { class: "flex flex-1 flex-col gap-2",
                TabButtonRequired {
                    active_tab: active_section,
                    tab: MainPageSection::DownloadQueue,
                }
                TabButtonRequired {
                    active_tab: active_section,
                    tab: MainPageSection::Downloads,
                }
                TabButtonRequired {
                    active_tab: active_section,
                    tab: MainPageSection::CustomCommand,
                }
                TabButtonRequired {
                    active_tab: active_section,
                    tab: MainPageSection::Settings,
                }
            }
            div { class: "flex flex-col gap-2",
                TabButtonRequired { active_tab: active_section, tab: MainPageSection::Sponsor }
                TabButtonRequired {
                    active_tab: active_section,
                    tab: MainPageSection::Troubleshooting,
                }
                TabButtonRequired { active_tab: active_section, tab: MainPageSection::About }
            }
            div { class: "divider my-2 h-0" }
            div { class: "flex gap-2",
                div { class: "btn btn-square btn-ghost",
                    Icon {
                        class: "text-current",
                        size: "1.25rem",
                        data: lucide::Globe,
                    }
                }
                div { class: "btn btn-square btn-ghost",
                    Icon {
                        class: "text-current",
                        size: "1.25rem",
                        data: lucide::Github,
                    }
                }
            }
        }
    }
}

#[component]
pub fn MainBarDownloadQueue() -> Element {
    let download_items_filter_query = use_signal(String::new);
    let mut download_items = use_signal(|| None);

    let mut new_download_url = use_signal(|| None);
    let mut new_download_url_info =
        use_action(move |url| async move { try_fetch_url_info(url).await });
    let mut new_download_selection = use_signal(DownloadUrlSelection::default);
    let mut active_new_download_phase = use_signal(|| None);

    let mut websocket =
        use_websocket(move || main_websocket(WebSocketOptions::new().with_automatic_reconnect()));

    use_future(move || async move {
        while let Ok(event) = websocket.recv().await {
            match event {
                MainWSServer::DownloadItems(data) => {
                    download_items.set(Some(data));
                }
            }
        }
    });

    use_effect(move || {
        let filter_query = download_items_filter_query();

        spawn(async move {
            let _ = websocket
                .send(MainWSClient::FetchDownloadItems { filter_query })
                .await
                .inspect_err(|e| error!("Failed to request FetchDownloadItems: {e}"));
        });
    });

    // TODO: Testing only
    // use_effect(move || {
    //     active_new_download_phase.set(Some(MainBarNewDownloadPhase::Configuration {}));
    //     new_download_url_info.call("https://www.youtube.com/watch?v=apKvlJxVO34".to_string());
    // });

    rsx! {
        div { class: "flex-1 grid grid-cols-[repeat(auto-fill,minmax(24rem,1fr))] content-start gap-2",
            MainBarDownloadQueueCard { class_add: "card-dash border-primary",
                h2 { class: "card-title mb-4",
                    Icon {
                        class: "text-current",
                        size: "1.25rem",
                        data: lucide::Download,
                    }
                    "Add a new download"
                }
                div { class: "join",
                    label { class: "join-item input w-full floating-label",
                        span { "Download link" }
                        input {
                            r#type: "text",
                            value: new_download_url,
                            placeholder: "Download link here",
                            oninput: move |evt| {
                                new_download_url.set(Some(evt.value()));
                            },
                        }
                    }
                    div {
                        class: "join-item btn btn-neutral",
                        onclick: move |_| {
                            spawn(async move {
                                if let Ok(clipboard_text) = try_read_clipboard_text().await
                                    && !clipboard_text.is_empty()
                                {
                                    new_download_url.set(Some(clipboard_text));
                                }
                            });
                        },
                        Icon {
                            class: "text-current",
                            size: "1rem",
                            data: lucide::Clipboard,
                        }
                        "Paste"
                    }
                }
                div {
                    a {
                        class: "label link",
                        href: "https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md",
                        target: "_blank",
                        "View supported sites"
                    }
                }
                div { class: "card-actions justify-end mt-4",
                    div {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            new_download_url.set(None);
                            active_new_download_phase.set(None);
                        },
                        Icon {
                            class: "text-current",
                            size: "1rem",
                            data: lucide::CircleX,
                        }
                        "Clear"
                    }
                    div {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            let Some(new_download_url) = new_download_url() else {
                                // TODO: Popup/toast error
                                return;
                            };

                            new_download_url_info.call(new_download_url.clone());

                            active_new_download_phase
                                .set(
                                    Some(MainBarNewDownloadPhase::Configuration {
                                    }),
                                );
                        },
                        Icon {
                            class: "text-current",
                            size: "1rem",
                            data: lucide::CircleArrowRight,
                        }
                        "Continue"
                    }
                }
            }
            if let Some(download_items_read) = download_items() {
                for download_item in download_items_read {
                    MainBarDownloadQueueCard {
                        div { class: "flex flex-col flex-1",
                            div { class: "flex flex-col flex-1",
                                div { class: "font-semibold", "{download_item.title}" }
                                div { class: "text-xs opacity-50", "{download_item.id}" }
                            }
                            div { class: "flex",
                                match download_item.item_state {
                                    DownloadItemState::Pending => rsx! {
                                        div { class: "badge badge-neutral",
                                            span { class: "loading loading-spinner loading-xs text-current" }
                                            "Pending"
                                        }
                                    },
                                    DownloadItemState::Downloading { progress } => rsx! {
                                        div { class: "badge badge-warning",
                                            span { class: "loading loading-spinner loading-xs text-current" }
                                            "{progress * 100 / 255}%"
                                        }
                                    },
                                    DownloadItemState::Error => rsx! {
                                        div { class: "badge badge-error",
                                            Icon { class: "text-current", size: "1rem", data: lucide::CircleX }
                                            "Error"
                                        }
                                    },
                                    DownloadItemState::Ok { date_finished } => rsx! {
                                        div { class: "badge badge-success",
                                            Icon { class: "text-current", size: "1rem", data: lucide::CircleCheck }
                                            {
                                                date_finished
                                                    .with_timezone(&chrono::Local)
                                                    .format("%Y-%m-%d %H:%M:%S")
                                                    .to_string()
                                            }
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
        if active_new_download_phase().is_some() {
            dialog { class: "modal modal-open modal-bottom sm:modal-middle",
                div { class: "modal-box max-w-none flex flex-col gap-4",
                    h2 { class: "card-title",
                        match active_new_download_phase() {
                            Some(MainBarNewDownloadPhase::Configuration {}) => rsx! {
                                Icon { class: "text-current", size: "1.25rem", data: lucide::Settings }
                                "Configure download format"
                            },
                            None => rsx! {},
                        }
                    }
                    div { class: "flex flex-1 flex-col gap-4",
                        match active_new_download_phase() {
                            Some(MainBarNewDownloadPhase::Configuration {}) => {
                                match new_download_url_info.value() {
                                    Some(Ok(url_info)) => rsx! {
                                        div { class: "flex h-24 gap-2 max-w-prose",
                                            img {
                                                class: "rounded-box overflow-clip h-full",
                                                src: if let Some(thumbnail) = url_info().thumbnail { format!("data:image/png;base64,{}", BASE64_STANDARD.encode(thumbnail)) },
                                            }
                                            div { class: "flex flex-col",
                                                div { class: "text-multiline-ellipsis [--line-clamp:1] font-semibold", {url_info().title} }
                                                pre { class: "text-multiline-ellipsis [--line-clamp:2] text-sm",
                                                    {url_info().description.map(|d| d.replace("\n\n", "\n"))}
                                                }
                                                div { class: "text-multiline-ellipsis [--line-clamp:1] text-xs text-current/50",
                                                    {url_info().uploader}
                                                }
                                            }
                                        }
                                        div { class: "flex flex-col",
                                            div { class: "divider divider-start divider-accent font-semibold text-neutral mb-4",
                                                "Audio only"
                                            }
                                            div { class: "grid grid-cols-[repeat(auto-fill,minmax(14rem,1fr))] gap-2",
                                                for format in url_info()
                                                    .formats
                                                    .into_iter()
                                                    .filter(|f| f.video_codec.is_none() && f.audio_codec.is_some())
                                                {
                                                    div {
                                                        key: "{format.format}",
                                                        class: "btn items-start flex-col py-4 h-auto",
                                                        class: if optional_audio_video_format_id_contains(
                                            &new_download_selection().audio_video_format_id,
                                            &AudioVideoFormatId::AudioOnly(format.format_id.clone()),
                                        ) { "btn-primary" } else { "btn-soft" },
                                                        onclick: move |_| {
                                                            let a_id = &format.format_id;

                                                            new_download_selection
                                                                .with_mut(|s| { with_audio_format(&mut s.audio_video_format_id, a_id) });
                                                        },
                                                        div { {format.format} }
                                                        div { class: "flex flex-wrap",
                                                            div {
                                                                span { class: "uppercase", {format.total_size.unwrap_or_default().to_string()} }
                                                                " "
                                                                span { class: "normal-case", {format.total_bitrate.unwrap_or_default()} }
                                                            }
                                                            div {
                                                                span { class: "uppercase", {format.extension} }
                                                                " "
                                                                span { class: "uppercase",
                                                                    {
                                                                        format!(
                                                                            "({}, {})",
                                                                            format.video_codec.as_ref().unwrap_or(&"NONE".to_string()),
                                                                            format.audio_codec.as_ref().unwrap_or(&"NONE".to_string()),
                                                                        )
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            div { class: "divider divider-start divider-accent font-semibold text-neutral mb-4",
                                                "Video only"
                                            }
                                            div { class: "grid grid-cols-[repeat(auto-fill,minmax(14rem,1fr))] gap-2",
                                                for format in url_info()
                                                    .formats
                                                    .into_iter()
                                                    .filter(|f| f.video_codec.is_some() && f.audio_codec.is_none())
                                                {
                                                    div {
                                                        key: "{format.format}",
                                                        class: "btn items-start flex-col py-4 h-auto",
                                                        class: if optional_audio_video_format_id_contains(
                                            &new_download_selection().audio_video_format_id,
                                            &AudioVideoFormatId::VideoOnly(format.format_id.clone()),
                                        ) { "btn-primary" } else { "btn-soft" },
                                                        onclick: move |_| {
                                                            let v_id = &format.format_id;

                                                            new_download_selection
                                                                .with_mut(|s| { with_video_format(&mut s.audio_video_format_id, v_id) });
                                                        },
                                                        div { {format.format} }
                                                        div { class: "flex flex-wrap",
                                                            div {
                                                                span { class: "uppercase", {format.total_size.unwrap_or_default().to_string()} }
                                                                " "
                                                                span { class: "normal-case", {format.total_bitrate.unwrap_or_default()} }
                                                            }
                                                            div {
                                                                span { class: "uppercase", {format.extension} }
                                                                " "
                                                                span { class: "uppercase",
                                                                    {
                                                                        format!(
                                                                            "({}, {})",
                                                                            format.video_codec.as_ref().unwrap_or(&"NONE".to_string()),
                                                                            format.audio_codec.as_ref().unwrap_or(&"NONE".to_string()),
                                                                        )
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            div { class: "divider divider-start divider-accent font-semibold text-neutral mb-4",
                                                "Video with audio"
                                            }
                                            div { class: "grid grid-cols-[repeat(auto-fill,minmax(14rem,1fr))] gap-2",
                                                for format in url_info()
                                                    .formats
                                                    .into_iter()
                                                    .filter(|f| f.video_codec.is_some() && f.audio_codec.is_some())
                                                {
                                                    div {
                                                        key: "{format.format}",
                                                        class: "btn items-start flex-col py-4 h-auto",
                                                        class: if optional_audio_video_format_id_contains(
                                            &new_download_selection().audio_video_format_id,
                                            &AudioVideoFormatId::VideoAudio(format.format_id.clone()),
                                        ) { "btn-primary" } else { "btn-soft" },
                                                        onclick: move |_| {
                                                            let va_id = &format.format_id;

                                                            new_download_selection
                                                                .with_mut(|s| {
                                                                    with_video_audio_format(&mut s.audio_video_format_id, va_id)
                                                                });
                                                        },
                                                        div { {format.format} }
                                                        div { class: "flex flex-wrap",
                                                            div {
                                                                span { class: "uppercase", {format.total_size.unwrap_or_default().to_string()} }
                                                                " "
                                                                span { class: "normal-case", {format.total_bitrate.unwrap_or_default()} }
                                                            }
                                                            div {
                                                                span { class: "uppercase", {format.extension} }
                                                                " "
                                                                span { class: "uppercase",
                                                                    {
                                                                        format!(
                                                                            "({}, {})",
                                                                            format.video_codec.as_ref().unwrap_or(&"NONE".to_string()),
                                                                            format.audio_codec.as_ref().unwrap_or(&"NONE".to_string()),
                                                                        )
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            div { class: "divider divider-start divider-accent font-semibold text-neutral mb-4",
                                                "Other"
                                            }
                                            div { class: "grid grid-cols-[repeat(auto-fill,minmax(14rem,1fr))] gap-2",
                                                for format in url_info()
                                                    .formats
                                                    .into_iter()
                                                    .filter(|f| f.video_codec.is_none() && f.audio_codec.is_none())
                                                {
                                                    div {
                                                        class: "btn justify-between",
                                                        class: if new_download_selection().storyboard_format_ids.contains(&format.format_id) { "btn-primary" } else { "btn-soft" },
                                                        key: "{format.format}",
                                                        onclick: move |_| {
                                                            let id = &format.format_id;

                                                            new_download_selection
                                                                .with_mut(|s| {
                                                                    if s.storyboard_format_ids.take(id).is_none() {
                                                                        s.storyboard_format_ids.insert(id.to_string());
                                                                    }
                                                                });
                                                        },
                                                        div { {format.format} }
                                                        div { class: "flex flex-wrap",
                                                            div {
                                                                span { class: "uppercase", {format.total_size.unwrap_or_default().to_string()} }
                                                                " "
                                                                span { class: "normal-case", {format.total_bitrate.unwrap_or_default()} }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "flex flex-col",
                                            div { class: "divider divider-start divider-accent font-semibold text-neutral mb-4",
                                                "Subtitle"
                                            }
                                            div { class: "grid grid-cols-[repeat(auto-fill,minmax(14rem,1fr))] gap-2",
                                                for subtitle in url_info().subtitles {
                                                    div {
                                                        key: "{subtitle.id()}",
                                                        class: "btn justify-between",
                                                        class: if new_download_selection().subtitle_ids.contains(subtitle.id()) { "btn-primary" } else { "btn-soft" },
                                                        onclick: move |_| {
                                                            let id = subtitle.id();

                                                            new_download_selection
                                                                .with_mut(|s| {
                                                                    if s.subtitle_ids.take(id).is_none() {
                                                                        s.subtitle_ids.insert(id.to_string());
                                                                    }
                                                                });
                                                        },
                                                        match &subtitle {
                                                            DownloadUrlInfoSubtitle::Subtitle {
                                                                id: _,
                                                                language_code,
                                                                language_name,
                                                                extensions,
                                                                is_automatic,
                                                            } => rsx! {
                                                                div { "{language_name}" }
                                                                if *is_automatic {
                                                                    span { class: "badge badge-neutral", "Auto" }
                                                                }
                                                            },
                                                            DownloadUrlInfoSubtitle::LiveChat { extensions } => rsx! {
                                                                div { "Live Chat" }
                                                            },
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    Some(Err(e)) => rsx! {
                                        div { "Failed to fetch URL info, caused by: ({e})" }
                                    },
                                    None => rsx! {
                                        div { class: "flex flex-1 justify-center my-16",
                                            span { class: "loading loading-spinner loading-xl text-info" }
                                        }
                                    },
                                }
                            }
                            None => rsx! {},
                        }
                    }
                    div { class: "flex modal-actions justify-end gap-2",
                        match active_new_download_phase() {
                            Some(MainBarNewDownloadPhase::Configuration {}) => rsx! {
                                div {
                                    class: "btn btn-secondary",
                                    onclick: move |_| {
                                        active_new_download_phase.set(None);
                                    },
                                    Icon { class: "text-current", size: "1rem", data: lucide::CircleX }
                                    "Cancel"
                                }
                                match new_download_url_info.value() {
                                    Some(Ok(url_info)) => rsx! {
                                        div {
                                            class: "btn btn-primary",
                                            onclick: move |_| {
                                                let DownloadUrlSelection {
                                                    audio_video_format_id,
                                                    storyboard_format_ids,
                                                    subtitle_ids,
                                                    thumbnail,
                                                } = new_download_selection();

                                                spawn(async move {
                                                    let Ok(()) = websocket
                                                        .send(MainWSClient::NewDownload {
                                                            download_url_info: Box::new(url_info()),
                                                            audio_video_format_ids: audio_video_format_id
                                                                .map(|f| f.as_ids())
                                                                .unwrap_or_default(),
                                                            storyboard_format_ids,
                                                            subtitle_ids,
                                                            thumbnail,
                                                        })
                                                        .await else {
                                                        return;
                                                    };
                                                    active_new_download_phase.set(None);
                                                });
                                            },
                                            Icon { class: "text-current", size: "1rem", data: lucide::Download }
                                            "Download"
                                        }
                                    },
                                    Some(Err(_)) => rsx! {
                                        div {
                                            class: "btn btn-error",
                                            onclick: move |_| {
                                                active_new_download_phase.set(None);
                                                new_download_url_info.reset();
                                            },
                                            Icon { class: "text-current", size: "1rem", data: lucide::CircleX }
                                            "Close"
                                        }
                                    },
                                    None => rsx! {
                                        div { class: "btn btn-disabled",
                                            span { class: "loading loading-spinner" }
                                            "Fetching info"
                                        }
                                    },
                                }
                            },
                            None => rsx! {},
                        }
                    }
                }
                form {
                    class: "modal-backdrop",
                    method: "dialog",
                    onclick: move |_| {
                        active_new_download_phase.set(None);
                    },
                    button { "Close dialog" }
                }
            }
        }
    }
}

#[component]
pub fn MainBarDownloadQueueCard(
    #[props]
    #[props(into, optional)]
    class_add: String,
    #[props(into)] children: Element,
) -> Element {
    rsx! {
        div { class: "card card-border {class_add}",
            div { class: "card-body", {children} }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MainBarNewDownloadPhase {
    Configuration {},
}

#[component]
pub fn MainBarNotImplementedYet() -> Element {
    rsx! {
        div { class: "text-error", "Not Implemented yet" }
    }
}

fn with_audio_format(state: &mut Option<AudioVideoFormatId>, a_id: &str) {
    *state = match state {
        Some(AudioVideoFormatId::AudioOnly(s_a_id)) if s_a_id == a_id => None,
        Some(AudioVideoFormatId::MixVideoAudio(s_v_id, s_a_id)) if s_a_id == a_id => {
            Some(AudioVideoFormatId::VideoOnly(s_v_id.to_string()))
        }
        Some(AudioVideoFormatId::VideoOnly(s_v_id))
        | Some(AudioVideoFormatId::MixVideoAudio(s_v_id, ..)) => Some(
            AudioVideoFormatId::MixVideoAudio(s_v_id.clone(), a_id.to_string()),
        ),
        _ => Some(AudioVideoFormatId::AudioOnly(a_id.to_string())),
    }
}

fn with_video_format(state: &mut Option<AudioVideoFormatId>, v_id: &str) {
    *state = match state {
        Some(AudioVideoFormatId::VideoOnly(s_v_id)) if s_v_id == v_id => None,
        Some(AudioVideoFormatId::MixVideoAudio(s_v_id, s_a_id)) if s_v_id == v_id => {
            Some(AudioVideoFormatId::AudioOnly(s_a_id.to_string()))
        }
        Some(AudioVideoFormatId::AudioOnly(s_a_id))
        | Some(AudioVideoFormatId::MixVideoAudio(.., s_a_id)) => Some(
            AudioVideoFormatId::MixVideoAudio(v_id.to_string(), s_a_id.clone()),
        ),
        _ => Some(AudioVideoFormatId::VideoOnly(v_id.to_string())),
    }
}

fn with_video_audio_format(state: &mut Option<AudioVideoFormatId>, va_id: &str) {
    *state = match state {
        Some(AudioVideoFormatId::VideoAudio(s_va_id)) if s_va_id == va_id => None,
        _ => Some(AudioVideoFormatId::VideoAudio(va_id.to_string())),
    }
}
