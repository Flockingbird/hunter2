use elefren::entities::account::Account;
use elefren::entities::event::Event;
use elefren::entities::status::Status;
use elefren::entities::card::Card;
use elefren::entities::attachment::Attachment;
use elefren::entities::status::Tag;
use elefren::prelude::*;
use elefren::helpers::cli;
use elefren::helpers::env;

use chrono::prelude::*;
use serde_json;
use serde::Serialize;
use getopts::Options;

use std::error::Error;

#[derive(Debug, Clone, Serialize, PartialEq)]
struct VacancyStatus {
    pub id: String,
    pub uri: String,
    pub url: Option<String>,
    pub account: VacancyAccount,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub media_attachments: Vec<VacancyAttachment>,
    pub tags: Vec<VacancyTag>,
    pub card: Option<VacancyCard>,
    pub language: Option<String>,
}
#[derive(Debug, Clone, Serialize, PartialEq)]
struct VacancyAccount {
    pub acct: String,
    pub avatar: String,
    pub avatar_static: String,
    pub display_name: String,
    pub url: String,
    pub username: String,
}
#[derive(Debug, Clone, Serialize, PartialEq)]
struct VacancyAttachment {
    pub id: String,
    #[serde(rename = "type")]
    pub media_type: MediaType,
    pub url: String,
    pub remote_url: Option<String>,
    pub preview_url: String,
}
#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
enum MediaType {
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
struct VacancyTag {
    name: String,
}
#[derive(Debug, Clone, Serialize, PartialEq)]
struct VacancyCard {
    url: String,
    title: String,
    description: String,
    image: Option<String>,
}

impl VacancyStatus {
    pub fn from(status: &Status) -> Self {
        let owned_status = status.to_owned();
        VacancyStatus {
            id: owned_status.id,
            uri: owned_status.uri,
            url: owned_status.url,
            account: VacancyAccount::from(owned_status.account),
            content: owned_status.content,
            created_at: owned_status.created_at,
            media_attachments: VacancyAttachment::from_vec(owned_status.media_attachments),
            tags: VacancyTag::from_vec(owned_status.tags),
            card: match owned_status.card {
                Some(card) => Some(VacancyCard::from(card)),
                None => None,
            },
            language: owned_status.language,
        }
    }
}
impl VacancyAccount {
    pub fn from(account: Account) -> Self {
        VacancyAccount {
            acct: account.acct,
            avatar: account.avatar,
            avatar_static: account.avatar_static,
            display_name: account.display_name,
            url: account.url,
            username: account.username,
        }
    }
}
impl VacancyAttachment {
    pub fn from_vec(attachments: Vec<Attachment>) -> Vec<Self> {
        attachments.into_iter().map(|a| {
            let media_type = match a.media_type {
                Image => MediaType::Image,
                Video => MediaType::Video,
                Gifv => MediaType::Gifv,
                Unknown => MediaType::Unknown,
            };

            VacancyAttachment {
                id: a.id,
                media_type: media_type,
                url: a.url,
                remote_url: a.remote_url,
                preview_url: a.preview_url,
            }
        }).collect()
    }
}
impl VacancyTag {
    pub fn from_vec(tags: Vec<Tag>) -> Vec<Self> {
        tags.into_iter().map(|t| VacancyTag { name: t.name }).collect()
    }
}
impl VacancyCard {
    pub fn from(card: Card) -> Self {
        VacancyCard {
            url: card.url,
            title: card.title,
            description: card.description,
            image: card.image,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("r", "register", "register hunter2 with your instance.");
    opts.optflag("f", "follow", "follow live updates.");
    opts.optflag("p", "past", "fetch past updates.");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };

    // print help when requested
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }

    if matches.opt_present("r") {
        register()?;
        return Ok(());
    }

    let data = env::from_env().unwrap();
    let mastodon = Mastodon::from(data);

    let you = mastodon.verify_credentials()?;

    out(welcome_msg(you));

    if matches.opt_present("p") {
        // TODO: This method will return duplicates. So we should deduplicate
        for tag in job_tags() {
            for status in mastodon.get_tagged_timeline(tag, false)? {
                if has_job_related_tags(&status.tags) {
                    out(publish(&status));
                }
            }
        }
    }

    if matches.opt_present("f") {
        for event in mastodon.streaming_public()? {
            match event {
                Event::Update(ref status) => {
                    if has_job_related_tags(&status.tags) {
                        out(publish(status));
                    }
                }
                Event::Notification(ref _notification) => { /* .. */ }
                Event::Delete(ref _id) => { /* .. */ }
                Event::FiltersChanged => { /* .. */ }
            }
        }
    }

    Ok(())
}

fn register() -> Result<Mastodon, Box<dyn Error>> {
    let registration = Registration::new(std::env::var("BASE").expect("BASE"))
        .client_name("hunter2")
        .build()?;
    let mastodon = cli::authenticate(registration)?;

    // Print the ENV var to screen for copying into whatever we use (.env)
    println!("Save these env vars in e.g. .env\n");
    println!("export {}=\"{}\"", "BASE", &mastodon.data.base);
    println!("export {}=\"{}\"", "CLIENT_ID", &mastodon.data.client_id);
    println!(
        "export {}=\"{}\"",
        "CLIENT_SECRET", &mastodon.data.client_secret
    );
    println!("export {}=\"{}\"", "REDIRECT", &mastodon.data.redirect);
    println!("export {}=\"{}\"\n", "TOKEN", &mastodon.data.token);

    Ok(mastodon)
}

fn welcome_msg(you: Account) -> String {
    //format!("We've sent out {} to hunt for jobs...", you.display_name)
    "".to_string()
}

fn publish(status: &Status) -> String {
    let vacancy = VacancyStatus::from(status);
    format!("{}", serde_json::to_string(&vacancy).unwrap())
}

// TODO: implement some -q or -o to pipe to other parts and pieces and whatnot
fn out(message: String) {
    println!("{}", message);
}

fn job_tags() -> Vec<String> {
    vec![
        "jobs".to_string(),
        "jobsearch".to_string(),
        "joboffer".to_string(),
        "hiring".to_string(),
        "vacancy".to_string(),
    ]
}

fn has_job_related_tags(tags: &Vec<Tag>) -> bool {
    // INK: debugging why checking for these tags does not work.
    // Probably best check first for a single tag?
    !tags.is_empty()
        && tags
            .iter()
            .map(|t| t.name.to_owned())
            .any(|e| job_tags().contains(&e))
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} TEMPLATE_FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_job_related_tags_with_jobs_tag() {
        let tags = vec![Tag {
            url: "".to_string(),
            name: "jobs".to_string(),
        }];
        assert!(has_job_related_tags(&tags))
    }

    #[test]
    fn test_has_job_related_tags_with_multiple_tags() {
        let tags = vec![
            Tag {
                url: "".to_string(),
                name: "jobs".to_string(),
            },
            Tag {
                url: "".to_string(),
                name: "steve".to_string(),
            },
        ];
        assert!(has_job_related_tags(&tags))
    }

    #[test]
    fn test_has_no_job_related_tags_without_tags() {
        let tags = vec![];
        assert!(!has_job_related_tags(&tags))
    }

    #[test]
    fn test_has_no_job_related_tags_without_allowed_tags() {
        let tags = vec![Tag {
            url: "".to_string(),
            name: "steve".to_string(),
        }];
        assert!(!has_job_related_tags(&tags))
    }
}
