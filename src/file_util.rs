use std::{
    fs::File,
    io::{Read, Write},
    slice,
};

use pgn_reader::BufferedReader;
use tokio::io::AsyncRead;
use tokio_util::io::SyncIoBridge;

use crate::{game_data::GameData, Error, Result};

pub fn to_local_filename(url: &str) -> Result<String> {
    let filename = url.split('/').last().ok_or(Error::InvalidFilename)?;
    Ok(format!("./data/{filename}"))
}

async fn open_file(filename: &str) -> Result<tokio::fs::File> {
    Ok(tokio::fs::File::open(filename).await?)
}

pub async fn from_file(filename: &str) -> Result<BufferedReader<impl Read>> {
    open_file(filename)
        .await // this is AsyncRead
        .and_then(to_buffered_reader)
}

fn to_buffered_reader(reader: impl AsyncRead + Unpin) -> Result<BufferedReader<impl Read>> {
    let bridge = SyncIoBridge::new(reader); //  this is Read
    let decoder = zstd::Decoder::new(bridge)?;
    Ok(BufferedReader::new(decoder))
}

pub fn write_batch(file: &mut File, v: &[GameData]) -> Result<()> {
    let p = v.as_ptr().cast();
    let l = std::mem::size_of_val(v);
    let d = unsafe { slice::from_raw_parts(p, l) };
    Ok(file.write_all(d)?)
}
