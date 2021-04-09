use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Account {
    id: String,
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
impl Account {}

#[derive(Debug, Deserialize, PartialEq)]
struct Tag {
    href: String,
    name: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Attachment {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Image {
    #[serde(rename = "mediaType")]
    media_type: String,
    url: String,
}
