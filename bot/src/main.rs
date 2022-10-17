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

use hunter2::may_index::may_index;

// 5000 ms (5s) seems OK for a low-volume bot. The balance is to ensure we
// have enough time to process all events that came in during the sleep time on
// one hand and to keep the load low on the other hand.
const THREAD_SLEEP_DURATION: Duration = Duration::from_millis(5000);

#[derive(Debug)]
pub enum Message {
    Generic(String),
    NewMessage(NewStatus),
    Vacancy(Status),
    Term,
}
impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Generic(content) => write!(f, "Generic: {}", content),
            Message::NewMessage(_) => write!(f, "NewMessage"),
            Message::Vacancy(status) => write!(f, "Vacancy: {}", status.uri),
            Message::Term => write!(f, "Term"),
        }
    }
}

fn main() -> Result<(), ProcessingError> {
    let cli_opts = CliOptions::new();
    let search_index_repository = SearchIndexRepository::new();
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
                    if may_index(&status.account.url) {
                        debug!("Handling vacancy: {:#?}", status);
                        search_index_repository.add(&status.clone().into());
                        client.favourite(&status.id).map_or_else(
                            |_| info!("Favourited {}", &status.uri),
                            |_| error!("Could not favourite {}", &status.uri),
                        );
                    }
                }
                Message::Generic(msg) => info!("{}", msg),
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
    })
}
