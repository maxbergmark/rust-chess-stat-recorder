use std::fmt::Display;

use itertools::Itertools;
use shakmaty::san::SanError;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[allow(dead_code)]
    FileAlreadyParsed,
    NetworkError,
    #[allow(dead_code)]
    FileError(std::io::Error),
    #[allow(dead_code)]
    DecompressionError,
    #[allow(dead_code)]
    InvalidMove(SanError, String),
    UIError,
    InvalidFilename,
    #[allow(dead_code)]
    ParsingError(Vec<u8>),
    PlottingError,
    GameError,
    TokioError,
}

#[allow(clippy::module_name_repetitions)]
pub trait ToCrateError<T> {
    fn to_chess_error(self, err: Error) -> Result<T, Error>;
}

impl<T, E> ToCrateError<T> for Result<T, E> {
    fn to_chess_error(self, err: Error) -> Result<T, Error> {
        self.map_err(|_| err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileError(e) => write!(f, "File error: {e}"),
            Self::FileAlreadyParsed => write!(f, "File already parsed"),
            Self::NetworkError => write!(f, "Network error"),
            Self::DecompressionError => write!(f, "Decompression error"),
            Self::InvalidMove(err, move_san) => {
                write!(f, "Invalid move: {err} for move {move_san}")
            }
            Self::UIError => write!(f, "UI error"),
            Self::InvalidFilename => write!(f, "Invalid filename"),
            Self::ParsingError(buffer) => {
                write!(f, "Parsing error: [{}]", comma_separated(buffer))
            }
            Self::PlottingError => write!(f, "Plotting error"),
            Self::GameError => write!(f, "Game error"),
            Self::TokioError => write!(f, "Tokio error"),
        }?;
        Ok(())
    }
}

fn comma_separated<T: Display>(v: &[T]) -> String {
    v.iter().map(ToString::to_string).join(", ")
}
