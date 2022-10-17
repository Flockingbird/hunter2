use futures::executor::block_on;
use log::{debug, info};

use crate::hunter2::vacancy::Vacancy;
use std::fmt::Display;

use meilisearch_sdk::client::Client;
use meilisearch_sdk::search::SearchResults;

#[derive(Clone)]
pub struct SearchIndexRepository {
    client: Client,
}

impl SearchIndexRepository {
    pub(crate) fn new() -> Self {
        let uri = std::env::var("MEILI_URI").expect("MEILI_URI");
        let key = std::env::var("MEILI_ADMIN_KEY").expect("MEILI_ADMIN_KEY");
        let client = Client::new(uri.as_str(), key.as_str());
        Self { client }
    }

    pub(crate) fn add(&self, vacancy: &Vacancy) {
        debug!("Adding document to search index: {:#?}", vacancy);
        info!("Adding document to search index: {}", vacancy.id);

        block_on(async move {
            self.client
                .index("vacancies")
                .add_documents(&[vacancy], Some("id"))
                .await
                .unwrap();
        });
    }

    pub(crate) fn delete_all(&self, toot_uri: String) -> Result<(), Error> {
        block_on(async move {
            let results = &self.get_all(toot_uri).await?;
            for hit in results.hits.iter() {
                self.delete(&hit.result.id).await?;
            }
            Ok(())
        })
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        let task = self.client.index("vacancies").delete_document(id).await?;
        task.wait_for_completion(&self.client, None, None).await?;
        Ok(())
    }

    async fn get_all(&self, toot_uri: String) -> Result<SearchResults<Vacancy>, Error> {
        self.client
            .index("vacancies")
            .search()
            .with_filter(format!("url = '{}'", toot_uri).as_str())
            .execute::<Vacancy>()
            .await
            .map_err(|e| e.into())
    }
}

#[derive(Debug)]
pub struct Error(meilisearch_sdk::errors::Error);
impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Repository error: {}", self.0)
    }
}

impl From<meilisearch_sdk::errors::Error> for Error {
    fn from(err: meilisearch_sdk::errors::Error) -> Self {
        Self(err)
    }
}
