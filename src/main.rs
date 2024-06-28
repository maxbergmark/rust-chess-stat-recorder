// #![allow(unused)]

use file_util::from_file;
use futures::future::join_all;
use itertools::Itertools;
use lichess_util::get_file_list;
use std::{
    io::Read,
    sync::{Arc, Mutex},
};
use ui::UI;

use error::ChessError;
use game::Game;
use pgn_reader::BufferedReader;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use validator::Validator;

mod enums;
mod error;
mod file_util;
mod game;
mod game_data;
mod game_player_data;
mod lichess_util;
mod ui;
mod util;
mod validator;

fn to_local_filename(url: &str) -> String {
    let parts = url.split('/').collect::<Vec<&str>>();
    let filename = parts.last().unwrap();
    format!("./data/{filename}")
}

fn parse_batch(
    chunk: impl Iterator<Item = Game>,
    filename: &String,
    progress: &mut u64,
    ui: Arc<Mutex<UI>>,
) {
    let games: Vec<Game> = chunk.collect();
    let data = games
        .into_par_iter()
        .flat_map(Game::validate)
        .collect::<Vec<_>>();
    *progress += data.len() as u64;
    if *progress % 10000 == 0 {
        ui.lock().unwrap().update_progress(filename, *progress);
        ui.lock().unwrap().update().unwrap();
    }
}

fn parse_all_games(filename: String, game_stream: BufferedReader<impl Read>, ui: Arc<Mutex<UI>>) {
    ui.lock().unwrap().add_file(&filename);
    ui.lock().unwrap().update().unwrap();
    let mut validator = Validator::new();
    let mut progress = 0;

    game_stream
        .into_iter(&mut validator)
        .flatten()
        .chunks(1000)
        .into_iter()
        .for_each(|chunk| parse_batch(chunk, &filename, &mut progress, ui.clone()));

    ui.lock().unwrap().complete_file(&filename, progress);
    ui.lock().unwrap().update().unwrap();
}

async fn parse_file(filename: &str, ui: Arc<Mutex<UI>>) -> Result<(), ChessError> {
    if !filename.contains("2013-") && !filename.contains("2014-") {
        return Ok(());
    }

    let filename = to_local_filename(filename);
    let game_stream = from_file(&filename).await?;

    tokio::task::block_in_place(|| {
        parse_all_games(filename, game_stream, ui);
    });
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ChessError> {
    let ui = UI::new().map_err(|_| ChessError::UIError)?;
    let ui_mutex = Arc::new(Mutex::new(ui));

    let futures = get_file_list().await?.into_iter().map(|filename| {
        let ui_mutex = ui_mutex.clone();
        tokio::spawn(async move {
            parse_file(&filename, ui_mutex).await.unwrap();
        })
    });
    join_all(futures).await;
    ui_mutex
        .lock()
        .unwrap()
        .exit()
        .map_err(|_| ChessError::UIError)?;
    Ok(())
}
