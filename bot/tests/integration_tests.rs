use assert_cmd::prelude::*;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::search::SearchResults;
use predicates::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, process::Command};

#[derive(Serialize, Deserialize, Debug)]
struct TestVacancy {
    pub id: String,
    pub url: String,
    pub uri: String,
    pub content: String,
}

#[test]
fn env_vars_not_set() {
    let mut cmd = Command::cargo_bin("hunter2").unwrap();

    cmd.env_remove("BASE");
    cmd.env_remove("TAG_FILE");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("NotPresent"));
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Did you export .env?"));
}

#[tokio::test]
async fn delete_from_index() {
    let client = given_a_meilisearch_client();
    let index = given_a_clean_index().await;
    given_a_vacancy_in_index(
        &client,
        &index,
        TestVacancy {
            id: "42".to_string(),
            url: "https://example.com/@foo@example.com/1337".to_string(),
            uri: "https://example.com/users/foo/statuses/1337".to_string(),
            content: "We are hiring".to_string(),
        },
    )
    .await;

    let results = when_searched_for(&index, "hiring").await;
    assert!(results.hits.len() == 1);

    let mut cmd = Command::cargo_bin("hunter2").unwrap();
    cmd.arg("--delete")
        .arg("https://example.com/@foo@example.com/1337");

    cmd.assert().success();

    let results = when_searched_for(&index, "hiring").await;
    assert!(results.hits.len() == 0);
}

fn given_a_meilisearch_client() -> Client {
    let uri = env::var("MEILI_URI").expect("MEILI_URI");
    let key = env::var("MEILI_ADMIN_KEY").expect("MEILI_ADMIN_KEY");

    Client::new(uri.as_str(), key)
}

async fn given_a_clean_index() -> Index {
    let uri = env::var("MEILI_URI").expect("MEILI_URI");
    let key = env::var("MEILI_ADMIN_KEY").expect("MEILI_ADMIN_KEY");
    let client = Client::new(uri.as_str(), key);

    let task = client.create_index("vacancies", Some("id")).await.unwrap();
    task.wait_for_completion(&client, None, None).await.unwrap();

    let vacancies = client.index("vacancies");
    let filterable_attributes = ["tags", "language", "url"];
    vacancies
        .set_filterable_attributes(&filterable_attributes)
        .await
        .unwrap();

    let task = vacancies.delete_all_documents().await.unwrap();
    task.wait_for_completion(&client, None, None).await.unwrap();

    vacancies
}

async fn given_a_vacancy_in_index(client: &Client, index: &Index, vacancy: TestVacancy) {
    let task = index.add_documents(&[vacancy], Some("id")).await.unwrap();
    task.wait_for_completion(client, None, None).await.unwrap();
}

async fn when_searched_for(index: &Index, query: &str) -> SearchResults<TestVacancy> {
    index
        .search()
        .with_query(query)
        .execute::<TestVacancy>()
        .await
        .unwrap()
}
