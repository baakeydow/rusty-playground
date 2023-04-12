#![allow(missing_docs)]

use std::ops::{Index, IndexMut};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::dtkutils::utils::null_if_empty;

use super::pocket_utils::{get_pocket_src_type, get_valid_title, get_valid_url, inject_tags_from_url};

#[derive(Serialize)]
/// Pocket url response
pub struct PockerUrlResponse {
    pub url: String,
    pub code: String,
    pub is_connected: bool,
}

#[derive(Deserialize)]
/// Pocket data response
pub struct PocketCodeResponseBody {
    pub code: String,
    // state: String
}

#[derive(Serialize, Deserialize, Debug)]
/// Response body for get access token
pub struct PocketTokenResponseBody {
    /// The access token for the user
    pub access_token: String,
    /// The name of the user
    pub username: String,
    /// The state that was passed in the request
    pub state: String,
}

/// Pocket source type
pub enum PocketSrcType {
    INSTAGRAM,
    TWITTER,
    YOUTUBE,
    GITHUB,
    ARTICLE,
}

impl PocketSrcType {
    pub fn to_string(&self) -> &'static str {
        match self {
            PocketSrcType::INSTAGRAM => "instagram",
            PocketSrcType::TWITTER => "twitter",
            PocketSrcType::YOUTUBE => "youtube",
            PocketSrcType::GITHUB => "github",
            PocketSrcType::ARTICLE => "article",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PocketData {
    pub item_id: String,
    pub resolved_id: String,
    pub given_url: String,
    pub given_title: String,
    pub favorite: String,
    pub status: String,
    pub time_added: String,
    pub time_updated: String,
    pub time_read: String,
    pub time_favorited: String,
    pub sort_id: u16,
    pub resolved_title: String,
    pub resolved_url: String,
    pub excerpt: String,
    pub is_article: String,
    pub is_index: String,
    pub has_video: String,
    pub has_image: String,
    pub word_count: String,
    pub lang: String,
    pub listen_duration_estimate: u16,

    pub tags: Option<serde_json::Value>,
    pub domain_metadata: Option<serde_json::Value>,
    pub authors: Option<serde_json::Value>,
    pub image: Option<serde_json::Value>,
    pub images: Option<serde_json::Value>,
    pub videos: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PocketSendResponse {
    pub action_results: serde_json::Value,
    pub status: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PocketExtractResponse {
    pub status: serde_json::Value,
    pub complete: serde_json::Value,
    pub list: serde_json::Value,
    pub error: Option<String>,
    pub search_meta: Option<serde_json::Value>,
    pub since: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
pub struct DtkPocketResponse {
    pub qualified: Vec<QualifiedPocketData>,
    pub instagram: Vec<DtkPocketData>,
    pub twitter: Vec<DtkPocketData>,
    pub unique_tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct QualifiedPocketData {
    pub is_main_feed: bool,
    pub is_valid: bool,
    pub is_link_only: bool,
    pub is_video: bool,
    pub is_article: bool,
    pub is_github: bool,
    pub is_twitter: bool,
    pub is_instagram: bool,
    pub is_youtube: bool,
    pub rusty_pocket_item: DtkPocketData,
}

/// !todo improve if you want but do not save QualifiedPocketData into mongodb !!!!! (only DtkPocketData)
/// QualifiedPocketData is only for client side !!!!!!!!!
impl From<DtkPocketData> for QualifiedPocketData {
    fn from(rusty_pocket_item: DtkPocketData) -> Self {
        let mut valid_video = rusty_pocket_item.videos.is_some();
        if valid_video {
            let videos: Vec<DtkPocketVideoData> = rusty_pocket_item
                .clone()
                .videos
                .unwrap()
                .as_object()
                .into_iter()
                .map(|video| {
                    let vid_object = video.get("1").unwrap().as_object().unwrap();
                    let item_id = vid_object.get("item_id").unwrap().as_u64().unwrap_or(0) as u16;
                    let video_id = vid_object.get("video_id").unwrap().as_u64().unwrap_or(0) as u8;
                    let src = vid_object.get("src").unwrap().as_str().unwrap().to_string();
                    let width = vid_object.get("width").unwrap().as_u64().unwrap_or(0) as u8;
                    let height = vid_object.get("height").unwrap().as_u64().unwrap_or(0) as u8;
                    let r#type = vid_object.get("type").unwrap().as_u64().unwrap_or(0) as u8;
                    let vid = vid_object.get("vid").unwrap().as_u64().unwrap_or(0) as u16;
                    let length = vid_object.get("length").unwrap().as_u64().unwrap_or(0) as u16;
                    DtkPocketVideoData {
                        item_id,
                        video_id,
                        src,
                        width,
                        height,
                        r#type,
                        vid,
                        length,
                    }
                })
                .collect();

            let videos_without_craps = videos
                .clone()
                .into_iter()
                .filter(|data| !data.src.contains("vimeo"))
                .collect::<Vec<DtkPocketVideoData>>();

            valid_video = !videos_without_craps.is_empty()
                && !rusty_pocket_item.url.contains("embed")
                && !rusty_pocket_item.url.contains("shorts");
        }

        let no_video_no_image = rusty_pocket_item.videos.is_none() && rusty_pocket_item.image.is_none();

        let perfect_article = !rusty_pocket_item.title.clone().is_empty()
            && rusty_pocket_item.excerpt.is_some()
            && rusty_pocket_item.image.is_some();

        QualifiedPocketData {
            is_main_feed: perfect_article || valid_video,
            is_valid: !rusty_pocket_item.title.is_empty(),
            is_link_only: no_video_no_image,
            is_video: valid_video,
            is_article: rusty_pocket_item.src_type == PocketSrcType::ARTICLE.to_string(),
            is_github: rusty_pocket_item.src_type == PocketSrcType::GITHUB.to_string(),
            is_twitter: rusty_pocket_item.src_type == PocketSrcType::TWITTER.to_string(),
            is_instagram: rusty_pocket_item.src_type == PocketSrcType::INSTAGRAM.to_string(),
            is_youtube: rusty_pocket_item.src_type == PocketSrcType::YOUTUBE.to_string(),
            rusty_pocket_item,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DtkPocketData {
    user_id: String,
    pub src_type: String,
    pub item_id: String,
    pub url: String,
    pub title: String,
    pub favorite: u8,
    pub status: u8,
    pub time_added: String,
    pub time_updated: String,
    pub time_read: String,
    pub time_favorited: String,
    pub excerpt: Option<String>,
    pub is_article: u8,
    pub is_index: u8,
    pub has_video: u8,
    pub has_image: u8,
    pub word_count: String,
    pub lang: String,
    pub listen_duration_estimate: u16,
    pub tags: Vec<String>,
    pub domain_metadata: Option<serde_json::Value>,
    pub authors: Option<serde_json::Value>,
    pub image: Option<serde_json::Value>,
    pub images: Option<serde_json::Value>,
    pub videos: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DtkPocketVideoData {
    pub item_id: u16,
    pub video_id: u8,
    pub src: String,
    pub width: u8,
    pub height: u8,
    pub r#type: u8,
    pub vid: u16,
    pub length: u16,
}

impl Index<&'_ str> for DtkPocketData {
    type Output = String;
    fn index(&self, s: &str) -> &String {
        match s {
            "item_id" => &self.item_id,
            "url" => &self.url,
            _ => panic!("unknown field: {}", s),
        }
    }
}

impl IndexMut<&'_ str> for DtkPocketData {
    fn index_mut(&mut self, s: &str) -> &mut String {
        match s {
            "item_id" => &mut self.item_id,
            "url" => &mut self.url,
            _ => panic!("unknown field: {}", s),
        }
    }
}

impl DtkPocketData {
    pub fn from_other_type(pocket_item: PocketData, user_id: &str) -> DtkPocketData {
        DtkPocketData {
            user_id: user_id.to_string(),
            src_type: get_pocket_src_type(pocket_item.clone()),
            url: get_valid_url(pocket_item.clone()),
            title: get_valid_title(pocket_item.clone()),
            tags: inject_tags_from_url(pocket_item.clone()),
            item_id: pocket_item.item_id,
            favorite: pocket_item.favorite.parse::<u8>().unwrap(),
            status: pocket_item.status.parse::<u8>().unwrap(),
            time_added: NaiveDateTime::from_timestamp_opt(pocket_item.time_added.parse::<i64>().unwrap(), 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            time_updated: NaiveDateTime::from_timestamp_opt(pocket_item.time_updated.parse::<i64>().unwrap(), 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            time_read: NaiveDateTime::from_timestamp_opt(pocket_item.time_read.parse::<i64>().unwrap(), 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            time_favorited: NaiveDateTime::from_timestamp_opt(pocket_item.time_favorited.parse::<i64>().unwrap(), 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            excerpt: null_if_empty(pocket_item.excerpt.clone()),
            is_article: pocket_item.is_article.parse::<u8>().unwrap(),
            is_index: pocket_item.is_index.parse::<u8>().unwrap(),
            has_video: pocket_item.has_video.parse::<u8>().unwrap(),
            has_image: pocket_item.has_image.parse::<u8>().unwrap(),
            word_count: pocket_item.word_count,
            lang: pocket_item.lang,
            listen_duration_estimate: pocket_item.listen_duration_estimate,
            domain_metadata: pocket_item.domain_metadata,
            authors: pocket_item.authors,
            image: pocket_item.image,
            images: pocket_item.images,
            videos: pocket_item.videos,
        }
    }
}
