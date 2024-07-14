use futures::{future::join_all, stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use pgn_reader::BufferedReader;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    config::Config,
    game_parser::{Game, GameData, Validator},
    plotter::Plotter,
    ui::{UserInterface, UI},
    util::{
        from_file, get_data_output_file, get_file_list, get_move_output_file, save_file,
        write_batch, write_moves, AndThenErr, FileInfo, Progress,
    },
    Result,
};

fn validate_and_log(game: Game, plotter: &Arc<Plotter>) -> Result<GameData> {
    game.validate()
        .inspect(|game_data| {
            Plotter::add_samples(game_data, plotter);
        })
        .and_then_err(|e| plotter.log_error(e))
}

fn parse_batch(
    chunk: impl Iterator<Item = Game>,
    data_output_file: &mut Option<File>,
    move_output_file: &mut Option<File>,
    plotter: &Arc<Plotter>,
) -> Result<Progress> {
    let data = chunk
        .collect::<Vec<_>>()
        .into_par_iter()
        .flat_map(|game| validate_and_log(game, plotter))
        .collect::<Vec<_>>();

    let rare_moves = data.iter().flat_map(GameData::get_rare_moves).collect_vec();

    rare_moves
        .iter()
        .try_for_each(|rare_move| Plotter::log_rare_move(plotter, rare_move))?;

    if let Some(data_output_file) = data_output_file {
        write_batch(data_output_file, &data)?;
    }
    if let Some(move_output_file) = move_output_file {
        write_moves(move_output_file, &rare_moves)?;
    }
    Ok(data.into())
}

fn parse_all_games(
    filename: &str,
    game_stream: BufferedReader<impl Read>,
    ui: &Arc<Mutex<UI>>,
    plotter: &Arc<Plotter>,
    config: &Config,
) -> Result<()> {
    let mut validator = Validator::new();
    let mut progress = Progress::default();

    let mut data_output_file = if config.output.data {
        Some(get_data_output_file(filename)?)
    } else {
        None
    };

    let mut move_output_file = if config.output.rare_moves {
        Some(get_move_output_file(filename)?)
    } else {
        None
    };

    game_stream
        .into_iter(&mut validator)
        .flatten()
        .chunks(10000)
        .into_iter()
        .try_for_each(|chunk| {
            progress += parse_batch(chunk, &mut data_output_file, &mut move_output_file, plotter)?;
            UI::update_progress(ui, filename, progress)?;
            plotter.update()
        })
        .or_else(|e| UI::set_error(ui, filename, &e))?;

    UI::complete_file(ui, filename, progress)
}

async fn parse_file(
    file_info: FileInfo,
    ui: &Arc<Mutex<UI>>,
    plotter: &Arc<Plotter>,
    config: &Config,
) -> Result<()> {
    if !config.years.contains(&file_info.year) {
        return Ok(());
    }

    let filename = file_info.filename.clone();
    let init = |file_size| UI::set_downloading(ui, &filename, file_size);
    let callback = |bytes: u64| UI::update_progress(ui, &filename, Progress::from_bytes(bytes));

    UI::add_file(ui, &file_info)?;
    if !Path::new(&filename).exists() {
        save_file(&file_info.url, &filename, init, callback).await?;
    }

    UI::set_processing(ui, &filename)?;
    let game_stream = from_file(&filename).await?;

    tokio::task::block_in_place(|| parse_all_games(&filename, game_stream, ui, plotter, config))
}

fn spawn_parse_file(
    file_info: FileInfo,
    ui: Arc<Mutex<UI>>,
    plotter: Arc<Plotter>,
    config: Config,
) -> tokio::task::JoinHandle<Result<()>> {
    tokio::spawn(async move { parse_file(file_info, &ui, &plotter, &config).await })
}

async fn push_until_full(
    futures: &mut FuturesUnordered<tokio::task::JoinHandle<Result<()>>>,
    future: tokio::task::JoinHandle<Result<()>>,
) {
    futures.push(future);
    if futures.len() >= 12 {
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
    let config = Config::from_file()?;
    let plotter = Plotter::new_arc(&config)?;
    let ui = UI::new_arc()?;

    let mut futures = FuturesUnordered::new();

    for file_info in get_file_list().await? {
        let future = spawn_parse_file(file_info, ui.clone(), plotter.clone(), config.clone());
        push_until_full(&mut futures, future).await;
    }

    collect(&mut futures).await?;

    ui.lock()?.wait_for_exit()?;
    Ok(())
}
