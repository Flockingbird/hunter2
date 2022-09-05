use elefren::entities::event::Event;
use elefren::entities::notification::Notification;
use elefren::entities::status::Status;
use elefren::helpers::env;
use elefren::prelude::*;
use regex::Regex;

use futures::executor::block_on;
use lazy_static::lazy_static;
use log::{debug, error, info};
use meilisearch_sdk::client::Client;

use core::fmt::Debug;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

mod cli_options;
mod error;
mod hunter2;
mod job_tags;
mod ports;

use cli_options::CliOptions;
use error::ProcessingError;
use ports::search_index_port::SearchIndexPort;

use hunter2::may_index::may_index;
use hunter2::vacancy::Vacancy;

// 5000 ms (5s) seems OK for a low-volume bot. The balance is to ensure we
// have enough time to process all events that came in during the sleep time on
// one hand and to keep the load low on the other hand.
const THREAD_SLEEP_DURATION: Duration = Duration::from_millis(5000);

#[derive(Debug)]
enum Message {
    Generic(String),
    Vacancy(Status),
    Term,
}

fn main() -> Result<(), ProcessingError> {
    let cli_opts = CliOptions::new();

    // print help when requested
    if cli_opts.help {
        cli_opts.print_usage();
        return Ok(());
    }

    // register this client over OAUTH
    if cli_opts.register {
        cli_opts.register();
        return Ok(());
    }

    // Delete a toot from index
    if let Some(toot_uri) = cli_opts.delete {
        return block_on(async move {
            println!("Deleting {}", toot_uri);
            return delete(toot_uri).await;
        });
    }

    let search_index_port = SearchIndexPort::new(cli_opts.meilisearch);
    env_logger::init();

    let data = match env::from_env() {
        Ok(data) => data,
        Err(err) => {
            panic!("Failed to load env var. Did you export .env?: {}", err)
        }
    };
    let mastodon = Mastodon::from(data);

    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    let messages_thread = handle_messages(rx, search_index_port, mastodon.clone());

    if cli_opts.past {
        // TODO: This method will return duplicates. So we should deduplicate
        for tag in job_tags::tags(&std::env::var("TAG_FILE").unwrap()) {
            for status in mastodon.get_tagged_timeline(tag, false)? {
                if has_job_related_tags(&status.tags) {
                    tx.send(Message::Vacancy(status)).unwrap();
                }
            }
        }
    }

    if cli_opts.follow {
        tx.send(Message::Generic(String::from("📨 Listening for vacancies")))
            .unwrap();
        let updates_thread = capture_updates(mastodon, tx);

        updates_thread.join().unwrap();
    } else {
        tx.send(Message::Term).unwrap();
    }

    messages_thread.join().unwrap();
    Ok(())
}

fn has_job_related_tags(tags: &[elefren::entities::status::Tag]) -> bool {
    !tags.is_empty()
        && tags
            .iter()
            .map(|t| t.name.to_owned())
            .any(|e| job_tags::tags(&std::env::var("TAG_FILE").unwrap()).contains(&e))
}

fn is_in_reply_to(mastodon: &elefren::Mastodon, notification: &Notification) -> Option<Status> {
    if let Some(status) = &notification.status {
        if let Some(reply_to_id) = &status.in_reply_to_id {
            let vacancy = mastodon.get_status(reply_to_id).unwrap();
            Some(vacancy)
        } else {
            None
        }
    } else {
        None
    }
}

fn has_indexme_request(content: &str) -> bool {
    // Matches "... index me ...", "indexme" etc.
    // But not "index like me" or "reindex meebo"
    lazy_static! {
        static ref RE: Regex = Regex::new("\\Windex\\s?me\\W").unwrap();
    };
    RE.is_match(content)
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
                Event::Notification(ref notification) => {
                    if let Some(status) = is_in_reply_to(&mastodon, notification) {
                        if has_indexme_request(&status.content) {
                            tx.send(Message::Vacancy(status)).unwrap();
                        }
                    }
                }
                Event::Delete(ref _id) => { /* .. */ }
                Event::FiltersChanged => { /* .. */ }
            }
        }
    })
}

fn handle_messages(
    rx: Receiver<Message>,
    search_index_port: SearchIndexPort,
    client: Mastodon,
) -> thread::JoinHandle<()> {
    debug!("opening message handler");
    thread::spawn(move || loop {
        if let Ok(received) = rx.try_recv() {
            info!("Handling: {:#?}", received);
            match received {
                Message::Vacancy(status) => {
                    if may_index(&status.account.url) {
                        debug!("Handling vacancy: {:#?}", status);
                        search_index_port.handle_vacancy(&status.clone().into());
                        client.favourite(&status.id).map_or_else(
                            |_| info!("Favourited {}", &status.id),
                            |err| error!("Could not favourite {}: {:#?}", &status.id, err),
                        );
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

async fn delete(toot_uri: String) -> Result<(), ProcessingError> {
    let uri = std::env::var("MEILI_URI").expect("MEILI_URI");
    let key = std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY");
    let client = Client::new(uri.as_str(), key.as_str());
    let index = client.index("vacancies");

    let results = index
        .search()
        .with_filter(format!("url = '{}'", toot_uri).as_str())
        .execute::<Vacancy>()
        .await?;

    if results.nb_hits == 0 {
        Err(ProcessingError::new(format!(
            "could not find a vacancy with url: {}",
            toot_uri
        )))
    } else {
        for hit in results.hits.iter() {
            let task = index.delete_document(&hit.result.id).await?;
            task.wait_for_completion(&client, None, None).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elefren::entities::status::Tag;
    use std::env::set_var;

    #[test]
    fn test_has_job_related_tags_with_jobs_tag() {
        set_tags_file_env();
        let tags = vec![Tag {
            url: "".to_string(),
            name: "jobs".to_string(),
        }];
        assert!(has_job_related_tags(&tags))
    }

    #[test]
    fn test_has_job_related_tags_with_multiple_tags() {
        set_tags_file_env();
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
        set_tags_file_env();
        let tags = vec![];
        assert!(!has_job_related_tags(&tags))
    }

    #[test]
    fn test_has_no_job_related_tags_without_allowed_tags() {
        set_tags_file_env();
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

    fn set_tags_file_env() {
        set_var("TAG_FILE", "./job_tags.txt");
    }
}
