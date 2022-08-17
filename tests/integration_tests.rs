use assert_cmd::prelude::*;
use dotenv;
use predicates::prelude::*;
use std::process::Command;
use std::panic;

#[test]
fn env_vars_not_set() {
    let mut cmd = Command::cargo_bin("hunter2").unwrap();

    cmd.env_remove("TAG_FILE");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Failed to load env var. Did you export .env?",
    ));
}

#[test]
fn delete_from_index() {
    run_test(|| {
        let mut cmd = Command::cargo_bin("hunter2").unwrap();
        cmd.arg("--delete")
            .arg("https://example.com/@foo@example.com/1337");

        cmd.assert().success();
    });
}

fn run_test<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    setup();

    let result = panic::catch_unwind(|| test());

    teardown();

    assert!(result.is_ok());
}

fn setup() {
    dotenv::from_filename(".env.test").expect("Attempt to load .env.test");
}

fn teardown() {
}
