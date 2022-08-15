use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn env_vars_not_set() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hunter2")?;

    cmd.env_remove("TAG_FILE");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Failed to load env var. Did you export .env?",
    ));

    Ok(())
}
