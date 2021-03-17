use elefren::prelude::*;
use elefren::entities::event::Event;
use elefren::entities::status::Tag;
use elefren::entities::account::Account;

use elefren::helpers::cli;
use elefren::helpers::env;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mastodon = if let Ok(data) = env::from_env() {
      Mastodon::from(data)
    } else {
        register()?
    };

    let you = mastodon.verify_credentials()?;

    welcome_msg(you);

    for event in mastodon.streaming_public()? {
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

fn register() -> Result<Mastodon, Box<dyn Error>> {
    let registration = Registration::new(std::env::var("BASE").expect("BASE"))
                                    .client_name("hunter2")
                                    .build()?;
    let mastodon = cli::authenticate(registration)?;

    // Print the ENV var to screen for copying into whatever we use (.env)
    println!("Save these env vars in e.g. .env\n");
    println!("export {}=\"{}\"", "BASE", &mastodon.data.base);
    println!("export {}=\"{}\"", "CLIENT_ID", &mastodon.data.client_id);
    println!("export {}=\"{}\"", "CLIENT_SECRET", &mastodon.data.client_secret);
    println!("export {}=\"{}\"", "REDIRECT", &mastodon.data.redirect);
    println!("export {}=\"{}\"\n", "TOKEN", &mastodon.data.token);

    Ok(mastodon)
}

fn welcome_msg(you: Account) {
    println!("We've sent out {} to hunt for jobs...", you.display_name);
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
