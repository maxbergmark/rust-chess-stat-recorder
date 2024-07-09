use derive_more::From;
use itertools::Itertools;
use rerun::RecordingStreamError;
use shakmaty::san::SanError;
use std::{fmt::Display, sync::PoisonError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    InvalidMove(SanError, String),
    InvalidFilename,
    #[from]
    ParseString(String),
    #[from]
    ParseBuffer(Vec<u8>),
    Mutex,
    #[from]
    Io(std::io::Error),
    #[from]
    Utf8(std::str::Utf8Error),
    ParseInt(std::num::ParseIntError),
    #[from]
    Reqwest(reqwest::Error),
    #[from]
    ParseDate(chrono::format::ParseError),
    #[from]
    Plotting(RecordingStreamError),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::Mutex
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMove(err, move_san) => {
                write!(f, "Invalid move: {err} for move {move_san}")
            }
            Self::InvalidFilename => write!(f, "Invalid filename"),
            Self::ParseString(s) => write!(f, "Parsing error: {s}"),
            Self::ParseBuffer(buffer) => {
                write!(f, "Parsing error: [{}]", comma_separated(buffer))
            }
            Self::Mutex => write!(f, "Mutex error"),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Utf8(e) => write!(f, "UTF-8 error: {e}"),
            Self::ParseInt(e) => write!(f, "Parse int error: {e}"),
            Self::Reqwest(e) => write!(f, "Reqwest error: {e}"),
            Self::ParseDate(e) => write!(f, "Date parse error: {e}"),
            Self::Plotting(e) => write!(f, "Plotting error: {e}"),
        }?;
        Ok(())
    }
}

fn comma_separated<T: Display>(v: &[T]) -> String {
    v.iter().map(ToString::to_string).join(", ")
}
