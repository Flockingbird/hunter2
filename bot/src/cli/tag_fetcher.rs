use crate::Message;
use elefren::prelude::*;
use std::error::Error;
use std::sync::mpsc::Sender;

pub struct TagFetcher {
    tags: Vec<String>,
    mastodon: Mastodon,
    tx: Sender<Message>,
}

impl TagFetcher {
    pub fn new(tags: Vec<String>, mastodon: Mastodon, tx: Sender<Message>) -> Self {
        Self { tags, mastodon, tx }
    }

    pub fn run_once(&self) -> Result<(), Box<dyn Error>> {
        for tag in &self.tags {
            let page = self.mastodon.get_hashtag_timeline(tag, false)?;

            for status in page.items_iter() {
                self.tx.send(Message::Vacancy(status))?;
            }
        }
        Ok(())
    }
}
