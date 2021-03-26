use elefren::entities::event::Event;

use elefren::helpers::cli;
use elefren::helpers::env;
use elefren::prelude::*;

use meilisearch_sdk::{client::*};

use getopts::Options;
use futures::executor::block_on;

use std::error::Error;

mod vacancy;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("r", "register", "register hunter2 with your instance.");
    opts.optflag("f", "follow", "follow live updates.");
    opts.optflag("p", "past", "fetch past updates.");
    opts.optflag("o", "out", "output to stdout instead of to meilisearch");

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
                    if matches.opt_present("o") {
                        out(format!("{:#?}", status));
                    } else {
                        into_meilisearch(&status);
                    }
                }
            }
        }
    }

    if matches.opt_present("f") {
        for event in mastodon.streaming_public()? {
            match event {
                Event::Update(ref status) => {
                    if has_job_related_tags(&status.tags) {
                        out(format!("{:#?}", status));
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

fn welcome_msg(you: elefren::entities::account::Account) -> String {
    //format!("We've sent out {} to hunt for jobs...", you.display_name)
    "".to_string()
}

// TODO: implement some -q or -o to pipe to other parts and pieces and whatnot
fn out(message: String) {
    println!("{}", message);
}

fn into_meilisearch(status: &elefren::entities::status::Status) {
    let uri = std::env::var("MEILI_URI").expect("MEILI_URI");
    let key = std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY");

    block_on(async move {
        let vacancy = vacancy::Status::from(status);
        let client = Client::new(uri.as_str(), key.as_str());
        let vacancies = client.get_or_create("vacancies").await.unwrap();
        // TODO: rewrite to accept a list and not single documents.
        // requires re-thinking how to deal with streaming api.
        vacancies.add_documents(&[vacancy], Some("id")).await.unwrap();
    })
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

fn has_job_related_tags(tags: &Vec<elefren::entities::status::Tag>) -> bool {
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
