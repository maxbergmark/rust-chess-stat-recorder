use std::{
    fs::File,
    io::{Read, Write},
    slice,
};

use pgn_reader::BufferedReader;
use tokio::io::AsyncRead;
use tokio_util::io::SyncIoBridge;

use crate::{
    error::{Error, ToCrateError},
    game_data::GameData,
};

pub fn to_local_filename(url: &str) -> Result<String, Error> {
    let parts = url.split('/').collect::<Vec<&str>>();
    let filename = parts.last().ok_or(Error::InvalidFilename)?;
    Ok(format!("./data/{filename}"))
}

pub async fn from_file(filename: &str) -> Result<BufferedReader<impl Read>, Error> {
    tokio::fs::File::open(filename)
        .await // this is AsyncRead
        .map_err(Error::FileError)
        // .to_chess_error(ChessError::FileError)
        .and_then(to_buffered_reader)
}

fn to_buffered_reader(reader: impl AsyncRead + Unpin) -> Result<BufferedReader<impl Read>, Error> {
    let bridge = SyncIoBridge::new(reader); //  this is Read
    let decoder = zstd::Decoder::new(bridge).to_chess_error(Error::DecompressionError)?;
    Ok(BufferedReader::new(decoder))
}

pub fn write_batch(file: &mut File, v: &[GameData]) -> Result<(), Error> {
    let p = v.as_ptr().cast();
    let l = std::mem::size_of_val(v);
    let d = unsafe { slice::from_raw_parts(p, l) };
    file.write_all(d).map_err(Error::FileError)
}
