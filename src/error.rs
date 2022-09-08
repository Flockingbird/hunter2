use crate::Message;
use std::{error::Error, fmt, sync::mpsc::SendError};

#[derive(Debug)]
pub struct ProcessingError {
    message: String,
}

impl ProcessingError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}
impl Error for ProcessingError {}
impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while processing command: {}", &self.message)
    }
}
impl From<meilisearch_sdk::errors::Error> for ProcessingError {
    fn from(err: meilisearch_sdk::errors::Error) -> Self {
        ProcessingError {
            message: format!("Meilisearch reported: {:?}", err),
        }
    }
}
impl From<elefren::Error> for ProcessingError {
    fn from(err: elefren::Error) -> Self {
        ProcessingError {
            message: format!("Elefren reported: {:?}", err),
        }
    }
}
impl From<SendError<Message>> for ProcessingError {
    fn from(err: SendError<Message>) -> Self {
        ProcessingError {
            message: format!("Message queue reported: {:?}", err),
        }
    }
}
