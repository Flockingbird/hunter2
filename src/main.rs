use elefren::entities::event::Event;

use elefren::helpers::cli;
use elefren::helpers::env;
use elefren::prelude::*;
use elefren::Language;

use futures::executor::block_on;
use getopts::Options;
use regex::Regex;

use core::fmt::Debug;
use std::error::Error;
use std::{panic, thread};

#[macro_use]
extern crate lazy_static;

mod vacancy;
use vacancy::IntoMeili;

#[derive(Clone)]
struct Output {
    stdout: bool,
    meilisearch: bool,
}
impl Output {
    fn handle(&self, status: &elefren::entities::status::Status) {
        self.into_stdout(status);
        if self.meilisearch {
            Output::into_meilisearch(&status);
        }
    }
    fn progress(&self, message: String) {
        if self.stdout {
            eprintln!("{}", message);
        }
    }
    fn error(&self, message: String) {
        eprintln!("{}", message);
    }

    fn into_stdout<T: Debug>(&self, status: T) {
        if self.stdout {
            println!("{:#?}", &status)
        }
    }

    fn into_meilisearch(status: &elefren::entities::status::Status) {
        let uri = std::env::var("MEILI_URI").expect("MEILI_URI");
        let key = std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY");
        let vacancy = vacancy::Status::from(status);
        vacancy.into_meili(uri, key);

        block_on(async move {});
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
    opts.optflag("o", "out", "output to stdout");
    opts.optflag("m", "meili", "output to meilisearch");

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

    let output = Output {
        stdout: matches.opt_present("o"),
        meilisearch: matches.opt_present("m"),
    };

    let data = env::from_env().unwrap();
    let mastodon = Mastodon::from(data);

    if matches.opt_present("p") {
        // TODO: This method will return duplicates. So we should deduplicate
        for tag in job_tags() {
            for status in mastodon.get_tagged_timeline(tag, false)? {
                if has_job_related_tags(&status.tags) {
                    output.handle(&status);
                }
            }
        }
    }

    if matches.opt_present("f") {
        // Give every thread its own client and its own output.
        output.progress(String::from(" 📨 Listening for indexme requests"));
        let notifications_thread = capture_notifications(mastodon.clone(), output.clone());

        output.progress(String::from(" 📨 Listening for vacancies"));
        let updates_thread = capture_updates(mastodon, output);

        match notifications_thread.join() {
            Ok(_) => {}
            Err(e) => panic::resume_unwind(e),
        };
        match updates_thread.join() {
            Ok(_) => {}
            Err(e) => panic::resume_unwind(e),
        };
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
    !tags.is_empty()
        && tags
            .iter()
            .map(|t| t.name.to_owned())
            .any(|e| job_tags().contains(&e))
}

fn has_indexme_request(content: &String) -> bool {
    // Matches "... index me ...", "indexme" etc. But not "index like me" or "reindex meebo"
    lazy_static! {
        static ref RE: Regex = Regex::new("\\Windex\\s?me\\W").unwrap();
    };
    RE.is_match(content.as_str())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} TEMPLATE_FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn capture_notifications(mastodon: elefren::Mastodon, output: Output) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for event in mastodon.streaming_user().unwrap() {
            match event {
                Event::Update(ref _status) => { /* .. */ }
                Event::Notification(ref notification) => {
                    if let Some(status) = &notification.status {
                        if has_indexme_request(&status.content) {
                            output.into_stdout(&notification);
                        } else {
                            reply_dont_understand(&status, mastodon.clone(), output.clone());
                        }
                    }
                }
                Event::Delete(ref _id) => { /* .. */ }
                Event::FiltersChanged => { /* .. */ }
            }
        }
    })
}

fn capture_updates(mastodon: elefren::Mastodon, output: Output) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for event in mastodon.streaming_public().unwrap() {
            match event {
                Event::Update(ref status) => {
                    if has_job_related_tags(&status.tags) {
                        output.handle(&status);
                    }
                }
                Event::Notification(ref _notification) => { /* .. */ }
                Event::Delete(ref _id) => { /* .. */ }
                Event::FiltersChanged => { /* .. */ }
            }
        }
    })
}

fn reply_dont_understand(
    in_reply_to: &elefren::entities::status::Status,
    mastodon: elefren::Mastodon,
    output: Output,
) {
    // Duplicate in order to move into thread.
    let id = in_reply_to.id.clone();

    let thread = thread::spawn(move || {
        let reply = StatusBuilder::new()
            .status("I'm sorry, I don't understand that. I only understand requests to 'index me', did you forget that phrase?")
            .language(Language::Eng)
            .in_reply_to(id)
            .build();
        match reply {
            Ok(status) => {
                match mastodon.new_status(status) {
                    Ok(_) => output.into_stdout("Replied with instructions."),
                    Err(exception) => output.error(format!("{:?}", exception)),
                };
            }
            Err(exception) => output.error(format!("{:?}", exception)),
        };
    });

    // TODO: Handle thread errors centrally instead of thhe christmastree above
    match thread.join() {
        Ok(_) => {}
        Err(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elefren::entities::status::Tag;

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

    #[test]
    fn test_notification_has_request_to_index_with_phrase() {
        let content =
            String::from("<p>Hi there, @hunter2@example.com, please index me, if you will?<p>");
        assert!(has_indexme_request(&content))
    }

    #[test]
    fn test_notification_has_request_to_index_with_tag() {
        let content =
            String::from("<p>please <a href=\"\">#<span>vacancy</span>indexme<span></a>!<p>");
        assert!(has_indexme_request(&content))
    }

    #[test]
    fn test_notification_has_no_request_to_index_with_phrase() {
        let content = String::from("<p>are you a bot?<p>");
        assert!(!has_indexme_request(&content))
    }

    #[test]
    fn test_notification_has_no_request_to_index_with_stretched_phrase() {
        let content = String::from("<p>Where is the index? Could you tell me?<p>");
        assert!(!has_indexme_request(&content))
    }

    #[test]
    fn test_notification_has_no_request_to_index_with_partial_words() {
        let content = String::from("<p>reindex meebo<p>");
        assert!(!has_indexme_request(&content))
    }
}
