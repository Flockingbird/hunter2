use crate::hunter2::vacancy::Vacancy;

use core::fmt::Debug;
use futures::executor::block_on;
use log::{debug, info};

use meilisearch_sdk::client::Client;
use serde::Serialize;

#[derive(Clone)]
pub struct SearchIndexPort {
    meilisearch: bool,
}

impl SearchIndexPort {
    pub(crate) fn new(meilisearch: bool) -> Self {
        Self { meilisearch }
    }

    pub(crate) fn handle_vacancy(&self, vacancy: &Vacancy) {
        self.write_into_meili(vacancy);
    }

    fn write_into_meili<T>(&self, document: &T)
    where
        T: Serialize,
        T: Debug,
        T: std::fmt::Display,
    {
        if self.meilisearch {
            let uri = std::env::var("MEILI_URI").expect("MEILI_URI");
            let key = std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY");
            let index = Client::new(uri.as_str(), key.as_str()).index("vacancies");

            debug!("Writing to Meili {}: {:#?}", uri, document);
            info!("Writing to Meili {}: {}", uri, document);

            block_on(async move {
                index.add_documents(&[document], Some("id")).await.unwrap();
            });
        }
    }
}

impl std::fmt::Display for Vacancy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Status id=\"{}\" uri=\"{}\">", self.id, self.uri)
    }
}
