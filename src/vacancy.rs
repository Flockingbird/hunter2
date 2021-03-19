use chrono::prelude::*;
use serde::Serialize;

use elefren::entities::*;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Status {
    pub id: String,
    pub uri: String,
    pub url: Option<String>,
    pub account: Account,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub media_attachments: Vec<Attachment>,
    pub tags: Vec<Tag>,
    pub card: Option<Card>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Account {
    pub acct: String,
    pub avatar: String,
    pub avatar_static: String,
    pub display_name: String,
    pub url: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Attachment {
    pub id: String,
    #[serde(rename = "type")]
    pub media_type: MediaType,
    pub url: String,
    pub remote_url: Option<String>,
    pub preview_url: String,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Tag {
    name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
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
            account: Account::from(owned_status.account),
            content: owned_status.content,
            created_at: owned_status.created_at,
            media_attachments: Attachment::from_vec(owned_status.media_attachments),
            tags: Tag::from_vec(owned_status.tags),
            card: match owned_status.card {
                Some(card) => Some(Card::from(card)),
                None => None,
            },
            language: owned_status.language,
        }
    }
}

impl Account {
    pub fn from(account: account::Account) -> Self {
        Account {
            acct: account.acct,
            avatar: account.avatar,
            avatar_static: account.avatar_static,
            display_name: account.display_name,
            url: account.url,
            username: account.username,
        }
    }
}

impl Attachment {
    pub fn from_vec(attachments: Vec<attachment::Attachment>) -> Vec<Self> {
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
                    media_type: media_type,
                    url: a.url,
                    remote_url: a.remote_url,
                    preview_url: a.preview_url,
                }
            })
            .collect()
    }
}

impl Tag {
    pub fn from_vec(tags: Vec<status::Tag>) -> Vec<Self> {
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
