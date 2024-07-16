use std::{
    fs::File,
    io::{Read, Write},
    slice,
    str::FromStr,
};

use pgn_reader::BufferedReader;
use tokio::io::AsyncRead;
use tokio_util::io::SyncIoBridge;

use crate::{
    game_parser::{GameData, RareMoveWithLink},
    Error, Result,
};

// counts: lichess_db_standard_rated_2013-01.pgn.zst 1
// list:   https://database.lichess.org/standard/lichess_db_standard_rated_2024-06.pgn.zst
// local:  ./data/lichess_db_standard_rated_2013-01.pgn.zst

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub url: String,
    pub filename: String,
    pub num_games: u64,
    pub year: u32,
    #[allow(unused)]
    pub month: u32,
}

impl FromStr for FileInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split(' ');

        let remote_filename = parts
            .next()
            .map(str::to_string)
            .ok_or(Error::InvalidFilename(s.to_string()))?;

        let url = format!("https://database.lichess.org/standard/{remote_filename}");

        let filename = to_local_filename(&remote_filename);

        let num_games = parts
            .next()
            .ok_or(Error::InvalidFilename(s.to_string()))?
            .parse()
            .map_err(Error::ParseInt)?;

        let year = filename[33..37].parse().map_err(Error::ParseInt)?;
        let month = filename[38..40].parse().map_err(Error::ParseInt)?;

        Ok(Self {
            url,
            filename,
            num_games,
            year,
            month,
        })
    }
}

pub fn to_local_filename(remote_filename: &str) -> String {
    format!("./data/{remote_filename}")
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

pub fn write_moves(file: &mut File, v: &[RareMoveWithLink]) -> Result<()> {
    v.iter().try_for_each(|rare_move| {
        writeln!(file, "{rare_move}")?;
        Ok(())
    })
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::*;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn test_year() -> Result<()> {
        let s = "lichess_db_standard_rated_2013-01.pgn.zst 123";
        let info: FileInfo = s.parse()?;
        assert_eq!(info.year, 2013);
        assert_eq!(info.month, 1);
        assert_eq!(info.num_games, 123);

        Ok(())
    }
}
