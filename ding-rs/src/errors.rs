use thiserror::Error;

#[derive(Debug, Error)]
pub enum DingError {
    #[error(transparent)]
    Url {
        #[from]
        source: url::ParseError,
    },

    #[error(transparent)]
    Request {
        #[from]
        source: reqwest::Error,
    },
}
