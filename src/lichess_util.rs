use std::io::{ErrorKind, Read};

use futures::{Stream, TryStreamExt};
use pgn_reader::BufferedReader;
use reqwest::{Client, Response};
use tokio_util::bytes::Bytes;

use crate::{error::ChessError, util::to_game_stream};

pub async fn get_file_list() -> Result<Vec<String>, ChessError> {
    let filename = "https://database.lichess.org/standard/list.txt";
    let body = reqwest::get(filename)
        .await
        .map_err(|_| ChessError::NetworkError)?
        .text()
        .await
        .map_err(|_| ChessError::NetworkError)?;

    let filenames = body.split('\n').map(str::to_string).collect();

    Ok(filenames)
}

#[allow(dead_code)]
async fn from_url(filename: &str) -> Result<BufferedReader<impl Read>, ChessError> {
    let client = Client::new();
    client
        .get(filename)
        .send()
        .await
        .map_err(|_| ChessError::NetworkError)
        .map(to_byte_stream)
        .and_then(to_game_stream)
}

fn to_byte_stream(response: Response) -> impl Stream<Item = Result<Bytes, std::io::Error>> {
    response.bytes_stream().map_err(convert_error)
}

fn convert_error(_err: reqwest::Error) -> std::io::Error {
    std::io::Error::new(ErrorKind::BrokenPipe, "network error")
}
