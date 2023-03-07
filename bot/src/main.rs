use elefren::entities::status::Status;
use elefren::helpers::env;
use elefren::Mastodon;
use elefren::MastodonClient;
use elefren::NewStatus;

use core::fmt::Debug;
use log::{debug, error, info};
use std::fmt::Display;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

mod cli;
mod cli_options;
mod error;
mod hunter2;
mod ports;

use cli::socket_client::{PublicStreamClient, StreamClient, UserStreamClient};

use cli::tag_fetcher::TagFetcher;
use cli_options::CliOptions;
use error::ProcessingError;

use ports::job_tags_repository::{JobTagsFileRepository, JobTagsRepository};
use ports::search_index_repository::SearchIndexRepository;

use hunter2::may_index::{may_index, is_stale, is_reply};

// 5000 ms (5s) seems OK for a low-volume bot. The balance is to ensure we
// have enough time to process all events that came in during the sleep time on
// one hand and to keep the load low on the other hand.
const THREAD_SLEEP_DURATION: Duration = Duration::from_millis(5000);

#[derive(Debug)]
pub enum Message {
    NewMessage(NewStatus),
    Vacancy(Status),
    Term,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::NewMessage(_) => write!(f, "NewMessage"),
            Message::Vacancy(status) => {
                write!(f, "Vacancy: {} - {}", status.uri, status.created_at)
            }
            Message::Term => write!(f, "Term"),
        }
    }
}

fn main() -> Result<(), ProcessingError> {
    let cli_opts = CliOptions::new();

    let search_index_repository = SearchIndexRepository::new(
        std::env::var("MEILI_URI")?,
        std::env::var("MEILI_ADMIN_KEY")?,
    );
    let job_tags_repository = JobTagsFileRepository::new(std::env::var("TAG_FILE")?);

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
        println!("Deleting {}", toot_uri);
        return search_index_repository
            .delete_all(toot_uri)
            .map_err(|e| e.into());
    }

    let mastodon = Mastodon::from(env::from_env()?);

    if let Some(toot_uri) = cli_opts.add {
        println!("Adding {}", toot_uri);
        let id = id_from_uri(toot_uri).expect("Attempt to extract an ID from the URI");
        let vacancy = mastodon.get_status(&id).unwrap().into();
        search_index_repository.add(&vacancy);
    }

    env_logger::init();

    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    let messages_thread = handle_messages(rx, search_index_repository, mastodon.clone());

    if cli_opts.past {
        TagFetcher::new(job_tags_repository.tags(), mastodon.clone(), tx.clone())
            .run_once()
            .expect("Fetching statuses for all job-tags");
    }

    if cli_opts.follow {
        PublicStreamClient::new(mastodon.clone(), tx.clone()).run()?;
        UserStreamClient::new(mastodon, tx).run()?;
    } else {
        tx.send(Message::Term)?;
    }

    messages_thread.join().unwrap();
    Ok(())
}

fn handle_messages(
    rx: Receiver<Message>,
    search_index_repository: SearchIndexRepository,
    client: Mastodon,
) -> thread::JoinHandle<()> {
    debug!("opening message handler");
    thread::spawn(move || loop {
        if let Ok(received) = rx.try_recv() {
            info!("Handling: {}", received);
            match received {
                Message::Vacancy(status) => {
                    maybe_index(status, &search_index_repository, &client);
                }
                Message::NewMessage(new_status) => {
                    debug!("sending new status");
                    client.new_status(new_status).expect("sending new message");
                }
                Message::Term => {
                    debug!("closing message handler");
                    break;
                }
            }
        }
        thread::sleep(THREAD_SLEEP_DURATION);
        debug!("Message loop next iteration");
    })
}

fn id_from_uri(uri: String) -> Option<String> {
    match uri.split('/').last() {
        Some(id) => {
            if id.chars().all(char::is_numeric) {
                Some(id.to_string())
            } else {
                None
            }
        }
        None => None,
    }
}

fn maybe_index(status: Status, search_index_repository: &SearchIndexRepository, client: &Mastodon) {
    if search_index_repository.exists(&status.id) {
        info!("Skipping existing vacancy: {}", status.uri);
        return;
    }

    if is_stale(&status.created_at) {
        info!("Skipping because too old");
        return;
    }

    if is_reply(&status.in_reply_to_id) {
        info!("Skipping because a reply to another post");
        return;
    }

    if !may_index(&status.account.url) {
        info!("Skipping because account does not allow indexing");
        return;
    }

    debug!("Handling vacancy: {:#?}", status);
    search_index_repository.add(&status.clone().into());
    client.favourite(&status.id).map_or_else(
        |_| info!("Favourited {}", &status.uri),
        |_| error!("Could not favourite {}", &status.uri),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_from_uri_with_proper_uri() {
        let uri = "https://example.com/@foo@example.com/1337".to_string();
        assert_eq!(Some("1337".to_string()), id_from_uri(uri));
    }

    #[test]
    fn test_id_from_uri_without_id_at_end() {
        let uri = "https://example.com/@foo@example.com/1337fail".to_string();
        assert_eq!(None, id_from_uri(uri));
    }

    #[test]
    fn test_id_from_uri_with_only_id() {
        let uri = "1337".to_string();
        assert_eq!(Some("1337".to_string()), id_from_uri(uri));
    }
}
