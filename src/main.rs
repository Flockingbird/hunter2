use elefren::prelude::*;
use elefren::entities::event::Event;
use elefren::entities::status::Tag;

use std::env;
use std::error::Error;

struct Conf {
    base: String,
    client_id: String,
    client_secret: String,
    token: String,
}

impl Conf {
    fn from_env() -> Conf {
        Conf {
            base: env::var("HUNTER2_BASE").expect("HUNTER2_BASE"),
            client_id: env::var("HUNTER2_CLIENT_ID").expect("HUNTER2_CLIENT_ID"),
            client_secret: env::var("HUNTER2_CLIENT_SECRET").expect("HUNTER2_CLIENT_SECRET"),
            token: env::var("HUNTER2_TOKEN").expect("HUNTER2_TOKEN"),
        }
    }

    fn as_data(self) -> Data {
        Data {
          base: self.base.into(),
          client_id: self.client_id.into(),
          client_secret: self.client_secret.into(),
          redirect: "urn:ietf:wg:oauth:2.0:oob".into(),
          token: self.token.into(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let data = Conf::from_env().as_data();
    let client = Mastodon::from(data);

    welcome_msg();
    for event in client.streaming_public()? {
        match event {
            Event::Update(ref status) => {
                if has_job_related_tags(&status.tags) {
                    println!("{:#?}", status);
                }
            },
            Event::Notification(ref _notification) => { /* .. */ },
            Event::Delete(ref _id) => { /* .. */ },
            Event::FiltersChanged => { /* .. */ },
        }
    }
    Ok(())
}

fn welcome_msg() {
    println!("Hunting for jobs...");
}

fn has_job_related_tags(tags: &Vec<Tag>) -> bool {
    let job_tags = vec!["jobs", "jobsearch", "joboffer", "hiring", "vacancy"];

    // INK: debugging why checking for these tags does not work.
    // Probably best check first for a single tag?
    !tags.is_empty() &&
      tags.iter().map(|t| t.name.as_str() ).any(|e| job_tags.contains(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_job_related_tags_with_jobs_tag() {
        let tags = vec![Tag { url: "".to_string(), name: "jobs".to_string() }];
        assert!(has_job_related_tags(&tags))
    }

    #[test]
    fn test_has_job_related_tags_with_multiple_tags() {
        let tags = vec![
            Tag { url: "".to_string(), name: "jobs".to_string() },
            Tag { url: "".to_string(), name: "steve".to_string() },
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
        let tags = vec![Tag { url: "".to_string(), name: "steve".to_string() }];
        assert!(!has_job_related_tags(&tags))
    }
}
