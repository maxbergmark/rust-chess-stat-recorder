use std::{
    io::{ErrorKind, Read},
    time::Duration,
};

use backoff::{
    exponential::{ExponentialBackoff, ExponentialBackoffBuilder},
    SystemClock,
};
use futures::{Stream, TryFutureExt, TryStreamExt};
use pgn_reader::BufferedReader;
use reqwest::{Client, Response};
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::bytes::Bytes;

use crate::{util::to_game_stream, Result};

pub async fn get_url_list() -> Result<Vec<String>> {
    let filename = "https://database.lichess.org/standard/list.txt";

    let filenames = reqwest::get(filename)
        .and_then(Response::text)
        .map_ok(|s| split_lines(&s))
        .await?;

    Ok(filenames)
}

fn split_lines(s: &str) -> Vec<String> {
    s.split('\n').map(str::to_string).collect()
}

async fn download_file(url: &str) -> Result<Response> {
    Ok(Client::new()
        .get(url)
        .send()
        .await
        .and_then(Response::error_for_status)?)
}

async fn retry_download(url: &str) -> Result<Response> {
    let backoff: ExponentialBackoff<SystemClock> = ExponentialBackoffBuilder::new()
        .with_max_interval(Duration::from_secs(60))
        .with_max_elapsed_time(None)
        .build();
    backoff::future::retry(backoff, || async { Ok(download_file(url).await?) }).await
}

pub async fn save_file(
    url: &str,
    filename: &str,
    callback: impl Fn(u64) -> Result<()> + Send,
) -> Result<()> {
    let response = retry_download(url).await?;
    let mut progress = 0;
    let mut file = File::create(filename).await?;
    let mut stream = response.bytes_stream().map_err(convert_error);
    while let Some(chunk) = stream.try_next().await? {
        progress += chunk.len() as u64;
        callback(progress)?;
        file.write_all(&chunk).await?;
    }
    Ok(())
}

#[allow(unused)]
async fn from_url(url: &str) -> Result<BufferedReader<impl Read>> {
    download_file(url)
        .await
        .map(to_byte_stream)
        .and_then(to_game_stream)
}

fn to_byte_stream(
    response: Response,
) -> impl Stream<Item = std::result::Result<Bytes, std::io::Error>> {
    response.bytes_stream().map_err(convert_error)
}

// #[allow(clippy::needless_pass_by_value)]
fn convert_error(_: reqwest::Error) -> std::io::Error {
    std::io::Error::new(ErrorKind::BrokenPipe, "network error")
}
