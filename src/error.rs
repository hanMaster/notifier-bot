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
    // -- Mailer
    Mailer(mail_send::Error),
    // -- Askama
    Tmpl(askama::Error),
    // -- Chrono
    Time(chrono::OutOfRangeError),
    // -- Xlsx
    Xlsx(rust_xlsxwriter::XlsxError)
}

// region:    ---From

impl From<rust_xlsxwriter::XlsxError> for Error {
    fn from(value: rust_xlsxwriter::XlsxError) -> Self {
        Error::Xlsx(value)
    }
}

impl From<chrono::OutOfRangeError> for Error {
    fn from(e: chrono::OutOfRangeError) -> Self {
        Error::Time(e)
    }
}

impl From<askama::Error> for Error {
    fn from(e: askama::Error) -> Self {
        Error::Tmpl(e)
    }
}

impl From<mail_send::Error> for Error {
    fn from(e: mail_send::Error) -> Self {
        Error::Mailer(e)
    }
}

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
