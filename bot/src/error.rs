use crate::{ports::search_index_repository, Message};
use std::{env::VarError, error::Error, fmt, sync::mpsc::SendError};

#[derive(Debug)]
pub struct ProcessingError {
    message: String,
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
impl From<search_index_repository::Error> for ProcessingError {
    fn from(err: search_index_repository::Error) -> Self {
        ProcessingError {
            message: format!("Search engine repo reported: {:?}", err),
        }
    }
}
impl From<VarError> for ProcessingError {
    fn from(err: VarError) -> Self {
        let hint = match err {
            VarError::NotPresent => "Did you export .env?".to_string(),
            VarError::NotUnicode(_) => "".to_string(),
        };
        ProcessingError {
            message: format!("Env var reported: {:?}. {}", err, hint),
        }
    }
}
