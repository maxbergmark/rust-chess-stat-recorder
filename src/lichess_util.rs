use std::{
    io::{ErrorKind, Read},
    time::Duration,
};

use backoff::{
    exponential::{ExponentialBackoff, ExponentialBackoffBuilder},
    SystemClock,
};
use futures::{Stream, TryStreamExt};
use pgn_reader::BufferedReader;
use reqwest::{Client, Response};
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::bytes::Bytes;

use crate::{
    error::{Error, ToCrateError},
    util::to_game_stream,
};

pub async fn get_url_list() -> Result<Vec<String>, Error> {
    let filename = "https://database.lichess.org/standard/list.txt";
    let body = reqwest::get(filename)
        .await
        .map_err(|_| Error::NetworkError)?
        .text()
        .await
        .map_err(|_| Error::NetworkError)?;

    let filenames = body.split('\n').map(str::to_string).collect();

    Ok(filenames)
}

async fn download_file(url: &str) -> Result<Response, Error> {
    let client = Client::new();
    client
        .get(url)
        .send()
        .await
        .and_then(Response::error_for_status)
        .to_chess_error(Error::NetworkError)
}

async fn retry_download(url: &str) -> Result<Response, Error> {
    let backoff: ExponentialBackoff<SystemClock> = ExponentialBackoffBuilder::new()
        .with_max_interval(Duration::from_secs(60))
        .with_max_elapsed_time(None)
        .build();
    backoff::future::retry(backoff, || async { Ok(download_file(url).await?) }).await
}

pub async fn save_file(
    url: &str,
    filename: &str,
    callback: impl Fn(u64) -> Result<(), Error> + Send,
) -> Result<(), Error> {
    let response = retry_download(url).await?;
    let mut progress = 0;
    let mut file = File::create(filename).await.map_err(Error::FileError)?;
    let mut stream = response.bytes_stream().map_err(convert_error);
    while let Some(chunk) = stream.try_next().await.map_err(Error::FileError)? {
        progress += chunk.len() as u64;
        callback(progress)?;
        file.write_all(&chunk).await.map_err(Error::FileError)?;
    }
    Ok(())
}

#[allow(dead_code)]
async fn from_url(url: &str) -> Result<BufferedReader<impl Read>, Error> {
    download_file(url)
        .await
        .map(to_byte_stream)
        .and_then(to_game_stream)
}

fn to_byte_stream(response: Response) -> impl Stream<Item = Result<Bytes, std::io::Error>> {
    response.bytes_stream().map_err(convert_error)
}

#[allow(clippy::needless_pass_by_value)]
fn convert_error(_: reqwest::Error) -> std::io::Error {
    std::io::Error::new(ErrorKind::BrokenPipe, "network error")
}
