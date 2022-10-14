use crate::error::ProcessingError;
use crate::ports::job_tags_repository::{JobTagsFileRepository, JobTagsRepository};
use crate::Message;
use elefren::entities::event::Event;
use elefren::entities::prelude::Notification;
use elefren::entities::status::Status;
use elefren::prelude::*;
use elefren::Mastodon;
use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use std::sync::mpsc::Sender;
use std::thread;

pub trait StreamClient {
    fn new(mastodon: Mastodon, tx: Sender<Message>) -> Self;
    fn run(&self) -> Result<thread::JoinHandle<()>, ProcessingError>;
}

pub struct PublicStreamClient {
    mastodon: Mastodon,
    tx: Sender<Message>,
}

impl StreamClient for PublicStreamClient {
    fn new(mastodon: Mastodon, tx: Sender<Message>) -> Self {
        Self { mastodon, tx }
    }
    fn run(&self) -> Result<thread::JoinHandle<()>, ProcessingError> {
        self.tx
            .send(Message::Generic("📨 Listening for vacancies".to_string()))?;
        Ok(capture_updates(
            self.mastodon.to_owned(),
            self.tx.to_owned(),
        ))
    }
}

pub struct UserStreamClient {
    mastodon: Mastodon,
    tx: Sender<Message>,
}

impl StreamClient for UserStreamClient {
    fn new(mastodon: Mastodon, tx: Sender<Message>) -> Self {
        Self { mastodon, tx }
    }

    fn run(&self) -> Result<thread::JoinHandle<()>, ProcessingError> {
        self.tx.send(Message::Generic(
            "📨 Listening for notifications".to_string(),
        ))?;
        Ok(capture_notifications(
            self.mastodon.to_owned(),
            self.tx.to_owned(),
        ))
    }
}

fn capture_updates(mastodon: elefren::Mastodon, tx: Sender<Message>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let job_tags_repository = JobTagsFileRepository::new(std::env::var("TAG_FILE").unwrap());
        for event in mastodon.streaming_public().unwrap() {
            if let Event::Update(status) = event {
                if has_job_related_tags(&status.tags, &job_tags_repository) {
                    debug!("Update {} is a vacancy", &status.id);
                    tx.send(Message::Vacancy(status)).unwrap();
                }
            }
        }
    })
}

fn has_job_related_tags<T: JobTagsRepository>(
    tags: &[elefren::entities::status::Tag],
    job_tags_repository: &T,
) -> bool {
    !tags.is_empty()
        && tags
            .iter()
            .map(|t| t.name.to_owned())
            .any(|e| job_tags_repository.tags().contains(&e))
}

fn capture_notifications(
    mastodon: elefren::Mastodon,
    tx: Sender<Message>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for event in mastodon.streaming_user().unwrap() {
            if let Event::Notification(notification) = event {
                debug!(
                    "Recieved a notification: {:#?}",
                    &notification.notification_type
                );
                if let Some(notification_status) = &notification.status {
                    if has_indexme_request(&notification_status.content) {
                        debug!(
                            "Notification has an indexme request: {}",
                            &notification_status.content
                        );
                        if let Some(status) = is_in_reply_to(&mastodon, &notification) {
                            debug!("Notification is a reply to: {}", &status.id);
                            tx.send(Message::Vacancy(status.clone())).unwrap();
                            let reply = StatusBuilder::new()
                                .status("Thanks for the request! The job posting can now be found at https://search.flockingbird.social/")
                                .in_reply_to(&status.id)
                                .build()
                                .unwrap();
                            tx.send(Message::NewMessage(reply))
                                .expect("Attempt to send a new message");
                        } else {
                            debug!("Notification is not a reply");
                        }
                    } else {
                        debug!(
                            "Notification has no indexme request: {}",
                            &notification_status.content
                        );
                        let reply = StatusBuilder::new()
                            .status("I did not understand the request. Did you include the phrase \"index this\"?")
                            .in_reply_to(&notification_status.id)
                            .build()
                            .unwrap();
                        tx.send(Message::NewMessage(reply))
                            .expect("Attempt to send a new message");
                    }
                }
            }
        }
    })
}

fn has_indexme_request(content: &str) -> bool {
    // Matches "... index this ...", "indexthis" etc.
    // But not "index like this" or "reindex thistle"
    lazy_static! {
        static ref RE: Regex = Regex::new("\\Windex\\s?this\\W").unwrap();
    };
    RE.is_match(content)
}

fn is_in_reply_to(mastodon: &elefren::Mastodon, notification: &Notification) -> Option<Status> {
    if let Some(status) = &notification.status {
        if let Some(reply_to_id) = &status.in_reply_to_id {
            let vacancy = mastodon.get_status(reply_to_id).unwrap();
            Some(vacancy)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::ports::job_tags_repository::{JobTagsMemoryRepository, JobTagsRepository};

    use super::*;
    use elefren::entities::status::Tag;

    #[test]
    fn test_has_job_related_tags_with_jobs_tag() {
        let tags = vec![Tag {
            url: "".to_string(),
            name: "jobs".to_string(),
        }];
        assert!(has_job_related_tags(&tags, &job_tags_repository()))
    }

    #[test]
    fn test_has_job_related_tags_with_multiple_tags() {
        let tags = vec![
            Tag {
                url: "".to_string(),
                name: "jobs".to_string(),
            },
            Tag {
                url: "".to_string(),
                name: "steve".to_string(),
            },
        ];
        assert!(has_job_related_tags(&tags, &job_tags_repository()))
    }

    #[test]
    fn test_has_no_job_related_tags_without_tags() {
        let tags = vec![];
        assert!(!has_job_related_tags(&tags, &job_tags_repository()))
    }

    #[test]
    fn test_has_no_job_related_tags_without_allowed_tags() {
        let tags = vec![Tag {
            url: "".to_string(),
            name: "steve".to_string(),
        }];
        assert!(!has_job_related_tags(&tags, &job_tags_repository()))
    }

    fn job_tags_repository() -> impl JobTagsRepository {
        JobTagsMemoryRepository {
            tags: vec!["jobs".to_string()],
        }
    }

    #[test]
    fn test_notification_has_request_to_index_with_phrase() {
        let content =
            String::from("<p>Hi there, @hunter2@example.com, please index this, if you will?<p>");
        assert!(has_indexme_request(&content))
    }

    #[test]
    fn test_notification_has_request_to_index_with_word() {
        let content = String::from("<p>indexthis<p>");
        assert!(has_indexme_request(&content))
    }

    #[test]
    fn test_notification_has_request_to_index_with_tag() {
        let content = String::from("<p>please <a href=\"\">#indexthis</a>!<p>");
        assert!(has_indexme_request(&content))
    }

    #[test]
    fn test_notification_has_no_request_to_index_with_phrase() {
        let content = String::from("<p>are you a bot?<p>");
        assert!(!has_indexme_request(&content))
    }

    #[test]
    fn test_notification_has_no_request_to_index_with_stretched_phrase() {
        let content = String::from("<p>Where is the index? Could you tell me?<p>");
        assert!(!has_indexme_request(&content))
    }

    #[test]
    fn test_notification_has_no_request_to_index_with_partial_words() {
        let content = String::from("<p>reindex thistle<p>");
        assert!(!has_indexme_request(&content))
    }
}
