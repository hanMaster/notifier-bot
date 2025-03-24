use std::fmt::{Display, Formatter};
use crate::adapters::{amo, profit};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // -- Config
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),

    Sqlx(sqlx::Error),
    AmoCRM(amo::Error),
    Profitbase(profit::Error),
    Request(teloxide::RequestError),
}

// region:    ---From

impl From<amo::Error> for Error {
    fn from(e: amo::Error) -> Error {
        Error::AmoCRM(e)
    }
}

impl From<profit::Error> for Error {
    fn from(e: profit::Error) -> Error {
        Error::Profitbase(e)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Sqlx(value)
    }
}

impl From<teloxide::RequestError> for Error {
    fn from(value: teloxide::RequestError) -> Self {
        Error::Request(value)
    }
}

// endregion: ---From

// region:    --- Error boilerplate
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
// endregion: --- Error boilerplate
