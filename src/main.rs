use elefren::entities::status::Status;
use elefren::helpers::env;
use elefren::Mastodon;
use elefren::MastodonClient;

use futures::executor::block_on;
use log::{debug, error, info};
use meilisearch_sdk::client::Client;

use core::fmt::Debug;
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
use hunter2::vacancy::Vacancy;

// 5000 ms (5s) seems OK for a low-volume bot. The balance is to ensure we
// have enough time to process all events that came in during the sleep time on
// one hand and to keep the load low on the other hand.
const THREAD_SLEEP_DURATION: Duration = Duration::from_millis(5000);

#[derive(Debug)]
pub enum Message {
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

    let data = match env::from_env() {
        Ok(data) => data,
        Err(err) => {
            panic!("Failed to load env var. Did you export .env?: {}", err)
        }
    };
    let mastodon = Mastodon::from(data);

    let search_index_repository = SearchIndexRepository::new(cli_opts.meilisearch);
    let job_tags_repository = JobTagsFileRepository::new(std::env::var("TAG_FILE").unwrap());
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
            info!("Handling: {:#?}", received);
            match received {
                Message::Vacancy(status) => {
                    if may_index(&status.account.url) {
                        debug!("Handling vacancy: {:#?}", status);
                        search_index_repository.add(&status.clone().into());
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
}
