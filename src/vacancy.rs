use chrono::prelude::*;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};

use elefren::entities::*;

use meilisearch_sdk::client::*;
use meilisearch_sdk::document::*;

use crate::meili::IntoMeili;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Status {
    pub id: String,
    pub uri: String,
    pub url: Option<String>,
    pub account: Account,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub created_at_ts: i64,
    pub media_attachments: Vec<Attachment>,
    pub tags: Vec<Tag>,
    pub card: Option<Card>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Account {
    pub id: String,
    pub acct: String,
    pub avatar: String,
    pub avatar_static: String,
    pub display_name: String,
    pub url: String,
    pub username: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Attachment {
    pub id: String,
    #[serde(rename = "type")]
    pub media_type: MediaType,
    pub url: String,
    pub remote_url: Option<String>,
    pub preview_url: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum MediaType {
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "gifv")]
    Gifv,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Tag {
    name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Card {
    url: String,
    title: String,
    description: String,
    image: Option<String>,
}

impl Status {
    pub fn from(status: &status::Status) -> Self {
        let owned_status = status.to_owned();
        Status {
            id: owned_status.id,
            uri: owned_status.uri,
            url: owned_status.url,
            account: Account::from(&owned_status.account),
            content: owned_status.content,
            created_at: owned_status.created_at,
            created_at_ts: owned_status.created_at.timestamp_millis(),
            media_attachments: Attachment::from(owned_status.media_attachments),
            tags: Tag::from(owned_status.tags),
            card: owned_status.card.map(Card::from),
            language: owned_status.language,
        }
    }
}

impl Document for Status {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}

impl IntoMeili for Status {
    fn write_into_meili(&self, uri: String, key: String) {
        let client = Client::new(uri.as_str(), key.as_str());
        let document = self.clone();
        block_on(async move {
            let index = client.index("vacancies");
            index.add_documents(&[document], Some("id")).await.unwrap();
        });
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Status id=\"{}\" uri=\"{}\">", self.id, self.uri)
    }
}

impl Account {
    pub fn from(account: &account::Account) -> Self {
        let owned_account = account.to_owned();
        Account {
            id: owned_account.id,
            acct: owned_account.acct,
            avatar: owned_account.avatar,
            avatar_static: owned_account.avatar_static,
            display_name: owned_account.display_name,
            url: owned_account.url,
            username: owned_account.username,
        }
    }
}

impl Attachment {
    pub fn from(attachments: Vec<attachment::Attachment>) -> Vec<Self> {
        attachments
            .into_iter()
            .map(|a| {
                let media_type = match a.media_type {
                    elefren::entities::attachment::MediaType::Image => MediaType::Image,
                    elefren::entities::attachment::MediaType::Video => MediaType::Video,
                    elefren::entities::attachment::MediaType::Gifv => MediaType::Gifv,
                    elefren::entities::attachment::MediaType::Unknown => MediaType::Unknown,
                };

                Attachment {
                    id: a.id,
                    media_type,
                    url: a.url,
                    remote_url: a.remote_url,
                    preview_url: a.preview_url,
                }
            })
            .collect()
    }
}

impl Tag {
    pub fn from(tags: Vec<status::Tag>) -> Vec<Self> {
        tags.into_iter().map(|t| Tag { name: t.name }).collect()
    }
}

impl Card {
    pub fn from(card: card::Card) -> Self {
        Card {
            url: card.url,
            title: card.title,
            description: card.description,
            image: card.image,
        }
    }
}
