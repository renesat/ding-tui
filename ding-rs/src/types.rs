use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct TagRequest {
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct TagsRequest {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct BookmarksRequest {
    pub query: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct BookmarkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_names: Option<Vec<String>>,
}

impl BookmarkRequest {
    pub fn new(url: Url) -> Self {
        Self {
            url: Some(url),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TagsResponse {
    pub count: u64,
    pub next: Option<Url>,
    pub previous: Option<Url>,
    pub results: Vec<Tag>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    pub id: u64,
    pub name: String,
    pub date_added: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BookmarksResponse {
    pub count: u64,
    pub next: Option<Url>,
    pub previous: Option<Url>,
    pub results: Vec<Bookmark>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bookmark {
    pub id: u64,
    pub url: Url,
    pub title: String,
    pub description: String,
    pub notes: String,
    pub website_title: Option<String>,
    pub website_description: Option<String>,
    #[serde(deserialize_with = "empty_url")]
    pub web_archive_snapshot_url: Option<Url>,
    pub favicon_url: Option<Url>,
    pub preview_image_url: Option<Url>,
    pub is_archived: bool,
    pub unread: bool,
    pub shared: bool,
    pub tag_names: Vec<String>,
    pub date_added: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserProfile {
    pub theme: String,
    pub bookmark_date_display: String,
    pub bookmark_link_target: String,
    pub web_archive_integration: String,
    pub tag_search: String,
    pub enable_sharing: bool,
    pub enable_public_sharing: bool,
    pub enable_favicons: bool,
    pub display_url: bool,
    pub permanent_notes: bool,
    pub search_preferences: SearchPreferences,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SearchPreferences {
    pub sort: String,
    pub shared: String,
    pub unread: String,
}

fn empty_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        let url = Url::parse(s).map_err(D::Error::custom)?;
        Ok(Some(url))
    }
}

pub trait IterableRequest {
    fn limit(&self, limit: Option<u64>) -> Self;
    fn offset(&self, offset: Option<u64>) -> Self;
}

impl IterableRequest for TagsRequest {
    fn limit(&self, limit: Option<u64>) -> TagsRequest {
        TagsRequest {
            offset: self.offset,
            limit,
        }
    }

    fn offset(&self, offset: Option<u64>) -> TagsRequest {
        TagsRequest {
            offset,
            limit: self.limit,
        }
    }
}

impl IterableRequest for BookmarksRequest {
    fn limit(&self, limit: Option<u64>) -> BookmarksRequest {
        BookmarksRequest {
            query: self.query.clone(),
            offset: self.offset,
            limit,
        }
    }

    fn offset(&self, offset: Option<u64>) -> BookmarksRequest {
        BookmarksRequest {
            query: self.query.clone(),
            offset,
            limit: self.limit,
        }
    }
}

pub trait IterableResponse<T> {
    fn next(&self) -> Option<Url>;
    fn results(&self) -> Vec<T>;
}

impl IterableResponse<Tag> for TagsResponse {
    fn next(&self) -> Option<Url> {
        self.next.clone()
    }

    fn results(&self) -> Vec<Tag> {
        self.results.clone()
    }
}

impl IterableResponse<Bookmark> for BookmarksResponse {
    fn next(&self) -> Option<Url> {
        self.next.clone()
    }

    fn results(&self) -> Vec<Bookmark> {
        self.results.clone()
    }
}
