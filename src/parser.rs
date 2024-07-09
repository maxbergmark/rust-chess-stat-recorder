use futures::{future::join_all, stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use pgn_reader::BufferedReader;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};

use crate::{
    file_util::{from_file, to_local_filename, write_batch},
    game::Game,
    game_data::GameData,
    lichess_util::{get_url_list, save_file},
    plotting::Plotter,
    ui::{UserInterface, UI},
    util::{get_output_file, AndThenErr, Progress},
    validator::Validator,
    Result,
};

fn validate_and_log(game: Game, plotter: &Arc<Plotter>) -> Result<GameData> {
    game.validate()
        .and_then(|game_data| {
            Plotter::add_samples(&game_data, plotter)?;
            Ok(game_data)
        })
        .and_then_err(|e| plotter.log_error(e))
}

fn parse_batch(
    chunk: impl Iterator<Item = Game>,
    output_file: &mut File,
    plotter: &Arc<Plotter>,
) -> Result<Progress> {
    let data = chunk
        .collect::<Vec<_>>()
        .into_par_iter()
        .flat_map(|game| validate_and_log(game, plotter))
        .collect::<Vec<_>>();

    write_batch(output_file, &data)?;
    Ok(data.into())
}

fn parse_all_games(
    filename: &str,
    game_stream: BufferedReader<impl Read>,
    ui: &Arc<Mutex<UI>>,
    plotter: &Arc<Plotter>,
) -> Result<()> {
    let mut validator = Validator::new();
    let mut progress = Progress::default();
    let mut output_file = get_output_file(filename)?;

    game_stream
        .into_iter(&mut validator)
        .flatten()
        .chunks(10000)
        .into_iter()
        .try_for_each(|chunk| {
            progress += parse_batch(chunk, &mut output_file, plotter)?;
            UI::update_progress(ui, filename, progress)?;
            plotter.update()
        })
        .or_else(|e| UI::set_error(ui, filename, &e))?;

    UI::complete_file(ui, filename, progress)
}

async fn parse_file(url: &str, ui: &Arc<Mutex<UI>>, plotter: &Arc<Plotter>) -> Result<()> {
    if !url.contains("2013-")
        && !url.contains("2014-")
        && !url.contains("2015-")
        && !url.contains("2016-")
        && !url.contains("2017-")
        && !url.contains("2018-")
        && !url.contains("2019-")
        && !url.contains("2020-")
    {
        return Ok(());
    }

    let filename = to_local_filename(url)?;
    let callback = |bytes: u64| {
        UI::set_downloading(ui, &filename)?;
        UI::update_progress(ui, &filename, Progress::from_bytes(bytes))
    };
    UI::add_file(ui, &filename)?;
    if !std::path::Path::new(&filename).exists() {
        save_file(url, &filename, callback).await?;
    }
    UI::set_processing(ui, &filename)?;
    let game_stream = from_file(&filename).await?;

    tokio::task::block_in_place(|| parse_all_games(&filename, game_stream, ui, plotter))
}

fn spawn_parse_file(
    url: String,
    ui: Arc<Mutex<UI>>,
    plotter: Arc<Plotter>,
) -> tokio::task::JoinHandle<Result<()>> {
    tokio::spawn(async move { parse_file(&url, &ui, &plotter).await })
}

async fn push_until_full(
    futures: &mut FuturesUnordered<tokio::task::JoinHandle<Result<()>>>,
    future: tokio::task::JoinHandle<Result<()>>,
) {
    futures.push(future);
    if futures.len() >= 10 {
        futures.next().await;
    }
}

async fn collect(
    futures: &mut FuturesUnordered<tokio::task::JoinHandle<Result<()>>>,
) -> Result<Vec<()>> {
    join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect::<Result<Vec<_>>>()
}

pub async fn run_all_files() -> Result<()> {
    let plotter = Plotter::new_arc()?;
    let ui = UI::new_arc()?;

    let mut futures = FuturesUnordered::new();

    for url in get_url_list().await? {
        let future = spawn_parse_file(url, ui.clone(), plotter.clone());
        push_until_full(&mut futures, future).await;
    }

    collect(&mut futures).await?;

    ui.lock()?.wait_for_exit()?;
    Ok(())
}
