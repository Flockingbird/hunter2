use chrono::prelude::*;
use elefren::entities::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Vacancy {
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

impl From<status::Status> for Vacancy {
    fn from(status: status::Status) -> Self {
        Vacancy {
            id: status.id,
            uri: status.uri,
            url: status.url,
            account: Account::from(&status.account),
            content: status.content,
            created_at: status.created_at,
            created_at_ts: status.created_at.timestamp_millis(),
            media_attachments: Attachment::from(status.media_attachments),
            tags: Tag::from(status.tags),
            card: status.card.map(Card::from),
            language: status.language,
        }
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
