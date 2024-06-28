use std::io::Read;

use pgn_reader::BufferedReader;
use tokio::{fs::File, io::AsyncRead};
use tokio_util::io::SyncIoBridge;

use crate::error::ChessError;

pub async fn from_file(filename: &str) -> Result<BufferedReader<impl Read>, ChessError> {
    File::open(filename)
        .await // this is AsyncRead
        .map(to_buffered_reader)
        .map_err(|_| ChessError::FileError)
}

fn to_buffered_reader(reader: impl AsyncRead + Unpin) -> BufferedReader<impl Read> {
    let bridge = SyncIoBridge::new(reader); //  this is Read
    let decoder = zstd::Decoder::new(bridge).expect("zst decoder");
    BufferedReader::new(decoder)
}
