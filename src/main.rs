use elefren::prelude::*;
use elefren::entities::event::Event;
use elefren::entities::status::Status;

use std::env;
use std::error::Error;

struct Conf {
    base: String,
    client_id: String,
    client_secret: String,
    token: String,
}

impl Conf {
    fn from_env() -> Conf {
        Conf {
            base: env::var("HUNTER2_BASE").expect("HUNTER2_BASE"),
            client_id: env::var("HUNTER2_CLIENT_ID").expect("HUNTER2_CLIENT_ID"),
            client_secret: env::var("HUNTER2_CLIENT_SECRET").expect("HUNTER2_CLIENT_SECRET"),
            token: env::var("HUNTER2_TOKEN").expect("HUNTER2_TOKEN"),
        }
    }

    fn as_data(self) -> Data {
        Data {
          base: self.base.into(),
          client_id: self.client_id.into(),
          client_secret: self.client_secret.into(),
          redirect: "urn:ietf:wg:oauth:2.0:oob".into(),
          token: self.token.into(),
        }
    }
}

fn main() -> Result<(), Box<Error>> {
    let data = Conf::from_env().as_data();
    let client = Mastodon::from(data);

    for event in client.streaming_public()? {
        match event {
            Event::Update(ref status) => {
                if is_job_related(status) {
                    println!("{:#?}", status);
                }
            },
            Event::Notification(ref _notification) => { /* .. */ },
            Event::Delete(ref _id) => { /* .. */ },
            Event::FiltersChanged => { /* .. */ },
        }
    }
    Ok(())
}

fn is_job_related(status: &Status) -> bool {
    let job_tags = vec!["jobs", "jobsearch", "joboffer", "hiring", "vacancy"];

    !status.tags.is_empty() &&
      status.tags.iter().map(|t| t.name.as_str() ).any(|e| job_tags.contains(&e))
}
