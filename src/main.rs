use elefren::entities::event::Event;
use elefren::helpers::cli;
use elefren::helpers::env;
use elefren::prelude::*;

use getopts::Options;
use log::{debug, info};

use core::fmt::Debug;
use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

mod job_tags;
mod may_index;
mod output;
mod vacancy;

use output::Output;

use crate::may_index::may_index;

// 5000 ms (5s) seems OK for a low-volume bot. The balance is to ensure we
// have enough time to process all events that came in during the sleep time on
// one hand and to keep the load low on the other hand.
const THREAD_SLEEP_DURATION: Duration = Duration::from_millis(5000);

#[derive(Debug)]
enum Message {
    Generic(String),
    Vacancy(elefren::entities::status::Status),
    Term,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("r", "register", "register hunter2 with your instance.");
    opts.optflag("f", "follow", "follow live updates.");
    opts.optflag("p", "past", "fetch past updates.");
    opts.optopt("o", "out", "output to filename as JSONL", "FILE");
    opts.optflag("m", "meili", "output to meilisearch");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => std::panic::panic_any(f.to_string()),
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

    let output = Output::new(matches.opt_str("o"), matches.opt_present("m"));
    env_logger::init();

    let data = match env::from_env() {
        Ok(data) => data,
        Err(err) => {
            panic!("Failed to load env var. Did you export .env?: {}", err)
        }
    };
    let mastodon = Mastodon::from(data);

    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    let messages_thread = handle_messages(rx, output);

    if matches.opt_present("p") {
        // TODO: This method will return duplicates. So we should deduplicate
        for tag in job_tags::tags(&std::env::var("TAG_FILE").unwrap()) {
            for status in mastodon.get_tagged_timeline(tag, false)? {
                if has_job_related_tags(&status.tags) {
                    tx.send(Message::Vacancy(status)).unwrap();
                }
            }
        }
    }

    if matches.opt_present("f") {
        tx.send(Message::Generic(String::from(
            " 📨 Listening for vacancies",
        )))
        .unwrap();
        let updates_thread = capture_updates(mastodon, tx);

        updates_thread.join().unwrap();
    } else {
        tx.send(Message::Term).unwrap();
    }

    messages_thread.join().unwrap();
    Ok(())
}

fn register() -> Result<Mastodon, Box<dyn Error>> {
    let registration = Registration::new(std::env::var("BASE").expect("BASE"))
        .client_name("hunter2")
        .build()?;
    let mastodon = cli::authenticate(registration)?;

    // Print the ENV var to screen for copying into whatever we use (.env)
    println!("Save these env vars in e.g. .env\n");
    println!("export BASE=\"{}\"", &mastodon.data.base);
    println!("export CLIENT_ID=\"{}\"", &mastodon.data.client_id);
    println!("export CLIENT_SECRET=\"{}\"", &mastodon.data.client_secret);
    println!("export REDIRECT=\"{}\"", &mastodon.data.redirect);
    println!("export TOKEN=\"{}\"\n", &mastodon.data.token);

    Ok(mastodon)
}

fn has_job_related_tags(tags: &[elefren::entities::status::Tag]) -> bool {
    !tags.is_empty()
        && tags
            .iter()
            .map(|t| t.name.to_owned())
            .any(|e| job_tags::tags(&std::env::var("TAG_FILE").unwrap()).contains(&e))
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} TEMPLATE_FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn capture_updates(mastodon: elefren::Mastodon, tx: Sender<Message>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for event in mastodon.streaming_public().unwrap() {
            match event {
                Event::Update(status) => {
                    if has_job_related_tags(&status.tags) {
                        debug!("Update {} is a vacancy", &status.id);
                        tx.send(Message::Vacancy(status)).unwrap();
                    }
                }
                Event::Notification(ref _notification) => { /* .. */ }
                Event::Delete(ref _id) => { /* .. */ }
                Event::FiltersChanged => { /* .. */ }
            }
        }
    })
}

fn handle_messages(
    rx: Receiver<Message>,
    output: Output,
) -> thread::JoinHandle<()> {
    debug!("opening message handler");
    thread::spawn(move || loop {
        if let Ok(received) = rx.try_recv() {
            info!("Handling: {:#?}", received);
            match received {
                Message::Vacancy(status) => {
                    if may_index(&status.account.url) {
                        debug!("Handling vacancy: {:#?}", status);
                        output.handle_vacancy(status.into());
                    }
                }
                Message::Generic(msg) => info!("{}", msg),
                Message::Term => {
                    debug!("closing message handler");
                    break;
                }
            }
        }
        thread::sleep(THREAD_SLEEP_DURATION);
    })
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
}
