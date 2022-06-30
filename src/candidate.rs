use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Candidate {
    pub id: String,
    #[serde(default)]
    pub ap_id: String,
    #[serde(rename = "preferredUsername")]
    pub username: String,
    name: String,
    summary: String,
    pub url: String,
    tag: Vec<Tag>,
    attachment: Vec<Attachment>,
    icon: Option<Image>,
    image: Option<Image>,
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
