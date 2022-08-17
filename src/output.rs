mod meili;

use crate::vacancy::Vacancy;
use meili::IntoMeili;

use core::fmt::Debug;
use futures::executor::block_on;
use log::{debug, info};

use meilisearch_sdk::client::Client;
use meilisearch_sdk::document::Document;

#[derive(Clone)]
pub struct Output {
    meilisearch: bool,
}

impl Output {
    pub(crate) fn new(meilisearch: bool) -> Output {
        Output { meilisearch }
    }

    pub(crate) fn handle_vacancy(&self, vacancy: &Vacancy) {
        self.write_into_meili(vacancy);
    }

    fn write_into_meili<T>(&self, document: &T)
    where
        T: IntoMeili,
        T: Clone,
        T: Debug,
        T: std::fmt::Display,
    {
        if self.meilisearch {
            let uri = std::env::var("MEILI_URI").expect("MEILI_URI");
            let key = std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY");
            debug!("Writing to Meili {}: {:#?}", uri, document);
            info!("Writing to Meili {}: {}", uri, document);
            document.write_into_meili(uri, key);
        }
    }
}

impl Document for Vacancy {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
        &self.id
    }
}

impl std::fmt::Display for Vacancy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Status id=\"{}\" uri=\"{}\">", self.id, self.uri)
    }
}

impl IntoMeili for Vacancy {
    fn write_into_meili(&self, uri: String, key: String) {
        let client = Client::new(uri.as_str(), key.as_str());
        let document = self.clone();
        block_on(async move {
            let index = client.index("vacancies");
            index.add_documents(&[document], Some("id")).await.unwrap();
        });
    }
}
