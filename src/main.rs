use elefren::entities::event::Event;

use elefren::helpers::cli;
use elefren::helpers::env;
use elefren::prelude::*;
use elefren::Language;

use getopts::Options;
use log::{debug, error, info};
use regex::Regex;
use reqwest::{header::ACCEPT, Client};
use serde::Serialize;
use uuid::Uuid;

use core::fmt::Debug;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[macro_use]
extern crate lazy_static;

mod meili;
use meili::IntoMeili;
mod candidate;
mod may_index;
mod vacancy;

// 5000 ms (5s) seems OK for a low-volume bot. The balance is to ensure we
// have enough time to process all events that came in during the sleep time on
// one hand and to keep the load low on the other hand.
const THREAD_SLEEP_DURATION: Duration = Duration::from_millis(5000);

const CONTACT_HUMAN_MSG:&str = "I am a bot. So please reach out to @flockingbird@fosstodon.org if you want to contact a human.";

#[derive(Debug)]
enum Message {
    Generic(String),
    Vacancy(elefren::entities::status::Status),
    IndexMe(elefren::entities::status::Status),
    ReplyUnderstood(elefren::entities::status::Status),
    ReplyDontUnderstand(elefren::entities::status::Status),
    Term,
}

#[derive(Clone)]
struct Output {
    file_name: Option<String>,
    meilisearch: bool,
}
impl Output {
    fn new(file_name: Option<String>, meilisearch: bool) -> Output {
        match &file_name {
            Some(file_name) => {
                File::create(file_name).unwrap();
            }
            None => {}
        };

        Output {
            file_name,
            meilisearch,
        }
    }

    fn handle_vacancy(&self, status: &elefren::entities::status::Status) {
        let vacancy = vacancy::Status::from(status);
        debug!("Handling vacancy: {:#?}", vacancy);
        if may_index::may_index(&vacancy.account.url) {
            self.into_file(&vacancy);
            self.into_meili(vacancy);
        }
    }
    fn handle_indexme(&self, account: &elefren::entities::account::Account) {
        debug!("Handling indexme: {:#?}", account);
        // Indexme is always indexed, regardless of users' indexing preferences.
        if let Ok(rich_account) = fetch_rich_account(&account.acct) {
            debug!("Fetched rich account: {:#?}", rich_account);
            self.into_file(&rich_account);
            self.into_meili(rich_account);
        }
    }
    fn into_file<T>(&self, status: T)
    where
        T: Serialize,
        T: Debug,
        T: std::fmt::Display,
    {
        match &self.file_name {
            Some(file_name) => {
                debug!("Writing to {}: {:#?}", file_name, status);
                info!("Writing to {}: {}", file_name, status);
                let mut file = OpenOptions::new().append(true).open(file_name).unwrap();
                let json = serde_json::to_string(&status).unwrap();
                file.write_all(json.as_bytes()).unwrap();
            }
            None => {}
        }
    }

    fn into_meili<T>(&self, document: T)
    where
        T: IntoMeili,
        T: Clone,
        T: Debug,
        T: std::fmt::Display,
    {
        if self.meilisearch {
            let uri = std::env::var("MEILI_URI").expect("MEILI_URI");
            let key = std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY");
            let owned_doc = document;
            debug!("Writing to Meili {}: {:#?}", uri, owned_doc);
            info!("Writing to Meili {}: {}", uri, owned_doc);
            owned_doc.into_meili(uri, key);
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
    let messages_thread = handle_messages(mastodon.clone(), rx, output);

    if matches.opt_present("p") {
        // TODO: This method will return duplicates. So we should deduplicate
        for tag in job_tags() {
            for status in mastodon.get_tagged_timeline(tag, false)? {
                if has_job_related_tags(&status.tags) {
                    tx.send(Message::Vacancy(status)).unwrap();
                }
            }
        }
    }

    if matches.opt_present("f") {
        tx.send(Message::Generic(String::from(
            " 📨 Listening for indexme requests",
        )))
        .unwrap();
        // Give every thread its own client and its own output.
        let notifications_thread = capture_notifications(mastodon.clone(), tx.clone());

        tx.send(Message::Generic(String::from(
            " 📨 Listening for vacancies",
        )))
        .unwrap();
        let updates_thread = capture_updates(mastodon, tx);

        notifications_thread.join().unwrap();
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

fn job_tags() -> Vec<String> {
    vec![
        "job".to_string(),
        "jobs".to_string(),
        "jobsearch".to_string(),
        "joboffer".to_string(),
        "hiring".to_string(),
        "vacancy".to_string(),
        "offredemploi".to_string(),
        "emploi".to_string(),
        "jobangebot".to_string(),
    ]
}

fn has_job_related_tags(tags: &[elefren::entities::status::Tag]) -> bool {
    !tags.is_empty()
        && tags
            .iter()
            .map(|t| t.name.to_owned())
            .any(|e| job_tags().contains(&e))
}

fn has_indexme_request(content: &str) -> bool {
    // Matches "... index me ...", "indexme" etc.
    // But not "index like me" or "reindex meebo"
    lazy_static! {
        static ref RE: Regex = Regex::new("\\Windex\\s?me\\W").unwrap();
    };
    RE.is_match(content)
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} TEMPLATE_FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn capture_notifications(
    mastodon: elefren::Mastodon,
    tx: Sender<Message>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for event in mastodon.streaming_user().unwrap() {
            match event {
                Event::Update(ref _status) => { /* .. */ }
                Event::Notification(notification) => {
                    debug!("Handling notification: {:#?}", notification);
                    if let Some(status) = notification.status {
                        debug!("Notification from {}: {}", status.account.acct, status.uri);
                        if has_indexme_request(&status.content) {
                            debug!("Notification {} is an indexme request", &status.id);
                            tx.send(Message::IndexMe(status.clone())).unwrap();
                            tx.send(Message::ReplyUnderstood(status)).unwrap();
                        } else {
                            debug!("Notification {} is not an indexme request", &status.id);
                            tx.send(Message::ReplyDontUnderstand(status)).unwrap();
                        }
                    }
                }
                Event::Delete(ref _id) => { /* .. */ }
                Event::FiltersChanged => { /* .. */ }
            }
        }
    })
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
    mastodon: elefren::Mastodon,
    rx: Receiver<Message>,
    output: Output,
) -> thread::JoinHandle<()> {
    debug!("opening message handler");
    thread::spawn(move || loop {
        if let Ok(received) = rx.try_recv() {
            info!("Handling: {:#?}", received);
            match received {
                Message::Vacancy(status) => {
                    output.handle_vacancy(&status);
                    mark_favorite(&status, mastodon.clone());
                }
                Message::IndexMe(status) => output.handle_indexme(&status.account),
                Message::ReplyUnderstood(status) => {
                    reply_understood(&status, mastodon.clone());
                }
                Message::ReplyDontUnderstand(status) => {
                    reply_dont_understand(&status, mastodon.clone());
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

fn fetch_rich_account(acct: &str) -> Result<candidate::Account, core::fmt::Error> {
    // TODO: handle errors!
    let res = webfinger::resolve(format!("acct:{}", acct), true).unwrap();
    let profile_link = res
        .links
        .into_iter()
        .find(|link| link.rel == "self")
        .unwrap();

    if let Some(href) = profile_link.href {
        let mut account = Client::new()
            .get(&href)
            .header(ACCEPT, profile_link.mime_type.unwrap())
            .send()
            .unwrap()
            .json::<candidate::Account>()
            .unwrap();

        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, &account.ap_id.as_bytes());
        account.ap_id = account.id;
        account.id = uuid.to_hyphenated().to_string();

        Ok(account)
    } else {
        Err(std::fmt::Error)
    }
}

fn mark_favorite(status: &elefren::entities::status::Status, mastodon: elefren::Mastodon) {
    let id = &status.id;

    match mastodon.favourite(id) {
        Ok(status) => info!("Favorited status '{}'", status.id),
        Err(err) => error!("Favoriting failed: {}", err),
    };
}

fn reply_understood(in_reply_to: &elefren::entities::status::Status, mastodon: elefren::Mastodon) {
    let id = &in_reply_to.id;

    let reply = StatusBuilder::new()
        .status(format!("Your account is being indexed and should show up when you search on https://search.flockingbird.social/candidates/ in a few minutes. - {}", CONTACT_HUMAN_MSG))
        .language(Language::Eng)
        .in_reply_to(id)
        .build().unwrap();

    match mastodon.new_status(reply) {
        Ok(status) => info!("Replying status with id '{}' with 'understood'", status.id),
        Err(err) => error!("Replying to {} failed: {}", id, err),
    };
}

fn reply_dont_understand(
    in_reply_to: &elefren::entities::status::Status,
    mastodon: elefren::Mastodon,
) {
    let id = &in_reply_to.id;

    let reply = StatusBuilder::new()
        .status(format!("I'm sorry, I don't understand that. I only understand requests to 'index me', did you forget that phrase? - {}", CONTACT_HUMAN_MSG))
        .language(Language::Eng)
        .in_reply_to(id)
        .build().unwrap();

    match mastodon.new_status(reply) {
        Ok(status) => info!(
            "Replying status with id '{}' with 'dont understand'",
            status.id
        ),
        Err(err) => error!("Replying to {} failed: {}", id, err),
    };
}

#[cfg(test)]
mod tests {
    // Don't forget to add assert_ne when used. For pretty output.
    use pretty_assertions::assert_eq;

    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

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

    #[test]
    fn test_fetch_rich_account_returns_account() -> Result<(), std::io::Error> {
        let acct = String::from("testing_hunter2@mastodon.online");
        let path = Path::new("./test/fixtures/hunter2_ap.json");
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let actual_account = fetch_rich_account(&acct).unwrap();

        let mut expected_account: candidate::Account = serde_json::from_reader(reader)?;
        expected_account.ap_id = expected_account.id;
        expected_account.id = actual_account.id.clone();

        assert_eq!(actual_account, expected_account);
        Ok(())
    }
}
