# hunter2

Hunter2 is a job hunt bot that finds jobs on the fediverse and makes those avaiable on a *job search page*.

## Bot

The bot is found in `/bot`. The readme there describes the bot and the tech.

## Search page

The search website is found `/web`.

## Quickstart

Requirements:

* Rust compiler and cargo: https://doc.rust-lang.org/cargo/getting-started/installation.html

### Run

After installing the requirements, on the development machine, run

```bash
cd bot
cargo run -- --help
```

This builds and runs the bot locally, and passes the flag `--help` to the bot.

### Test

After installing the requirements, on the development machine, run

```bash
cd bot
cargo test
```

This builds and runs the tests locally.

### Release and deploy.

Any commit on main, will be checked, tested and when those pass, a release will be built and pushed to production.

Any pull-request against `main` will be checked and tested. When merged into main, a release will be built and pushed to production.

