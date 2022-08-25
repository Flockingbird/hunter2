use assert_cmd::prelude::*;
use dotenv;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::tasks::Task;
use predicates::prelude::*;
use serde::{Deserialize, Serialize};
use std::process::Command;

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

    cmd.assert().failure().stderr(predicate::str::contains(
        "Failed to load env var. Did you export .env?",
    ));
}

#[tokio::test]
async fn delete_from_index() {
    env();
    let uri = std::env::var("MEILI_URI").expect("MEILI_URI");
    let key = std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY");

    let client = Client::new(uri.as_str(), key);
    let task = client.create_index("vacancies", Some("id")).await.unwrap();
    task.wait_for_completion(&client, None, None).await.unwrap();

    let vacancies = client.index("vacancies");
    let filterable_attributes = ["tags", "language", "url"];
    vacancies
        .set_filterable_attributes(&filterable_attributes)
        .await
        .unwrap();

    let task: Task = vacancies.delete_all_documents().await.unwrap();
    task.wait_for_completion(&client, None, None).await.unwrap();

    let task = vacancies
        .add_documents(
            &[TestVacancy {
                id: "42".to_string(),
                url: "https://example.com/@foo@example.com/1337".to_string(),
                uri: "https://example.com/users/foo/statuses/1337".to_string(),
                content: "We are hiring".to_string(),
            }],
            Some("id"),
        )
        .await
        .unwrap();
    task.wait_for_completion(&client, None, None).await.unwrap();

    let results = vacancies
        .search()
        .with_query("hiring")
        .with_limit(5)
        .execute::<TestVacancy>()
        .await
        .unwrap();
    assert!(results.hits.len() == 1);

    let mut cmd = Command::cargo_bin("hunter2").unwrap();
    cmd.arg("--delete")
        .arg("https://example.com/@foo@example.com/1337");

    cmd.assert().success();

    let results = vacancies
        .search()
        .with_query("hiring")
        .with_limit(5)
        .execute::<TestVacancy>()
        .await
        .unwrap();
    assert!(results.hits.len() == 0);
}

fn env() {
    dotenv::from_filename(".env.test").expect("Attempt to load .env.test");
}
