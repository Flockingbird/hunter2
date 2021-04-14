use serde::{Deserialize, Serialize};

use futures::executor::block_on;
use meilisearch_sdk::client::*;
use meilisearch_sdk::document::*;

use crate::meili::IntoMeili;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Account {
    pub id: String,
    #[serde(default)]
    pub ap_id: String,
    #[serde(rename = "preferredUsername")]
    username: String,
    name: String,
    summary: String,
    url: String,
    tag: Vec<Tag>,
    attachment: Vec<Attachment>,
    icon: Option<Image>,
    image: Option<Image>,
}

impl Document for Account {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}

impl IntoMeili for Account {
    fn into_meili(&self, uri: String, key: String) {
        let client = Client::new(uri.as_str(), key.as_str());
        let document = self.clone();
        block_on(async move {
            let index = client.get_or_create("candidates").await.unwrap();
            index.add_documents(&[document], Some("id")).await.unwrap();
        });
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Tag {
    href: String,
    name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Attachment {
    name: String,
    value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Image {
    #[serde(rename = "mediaType")]
    media_type: String,
    url: String,
}
