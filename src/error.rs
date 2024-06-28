use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ChessError {
    #[allow(dead_code)]
    FileAlreadyParsed,
    NetworkError,
    FileError,
    #[allow(dead_code)]
    DecompressionError,
    InvalidMove,
    UIError,
}

impl Error for ChessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for ChessError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
