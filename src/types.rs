use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct GenericError<T> {
    source: T,
    backtrace: Backtrace,
}

impl<T> From<T> for GenericError<T> {
    fn from(value: T) -> Self {
        Self {
            source: value,
            backtrace: Backtrace::capture(),
        }
    }
}

#[derive(Debug)]
pub enum MscpError {
    StdIo(GenericError<std::io::Error>),
    SetLogger(GenericError<log::SetLoggerError>),
}

impl From<std::io::Error> for MscpError {
    fn from(value: std::io::Error) -> Self {
        Self::StdIo(value.into())
    }
}

impl From<log::SetLoggerError> for MscpError {
    fn from(value: log::SetLoggerError) -> Self {
        Self::SetLogger(value.into())
    }
}

impl Display for MscpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.source())?;
        writeln!(f, "{}", self.backtrace())
    }
}

impl Error for MscpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source())
    }
}

impl MscpError {
    pub fn backtrace(&self) -> &Backtrace {
        match self {
            Self::StdIo(e) => &e.backtrace,
            Self::SetLogger(e) => &e.backtrace,
        }
    }

    pub fn source(&self) -> &(dyn Error + 'static) {
        match self {
            MscpError::StdIo(err) => &err.source,
            MscpError::SetLogger(err) => &err.source,
        }
    }
}

pub type Result<T> = std::result::Result<T, MscpError>;
