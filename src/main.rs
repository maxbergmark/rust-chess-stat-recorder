#![warn(
    // missing_docs,
    // unreachable_pub,
    keyword_idents,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    future_incompatible,
    nonstandard_style,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
)]

use file_util::{from_file, to_local_filename, write_batch};
use futures::{future::join_all, stream::FuturesUnordered, StreamExt, TryFutureExt};
use game_data::GameData;
use itertools::Itertools;
use lichess_util::{get_url_list, save_file};
use plotting::Plotter;
use std::{
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};

use error::{Error, ToCrateError};
use game::Game;
use pgn_reader::BufferedReader;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use ui::{UserInterface, UI};
use validator::Validator;

mod enums;
mod error;
mod file_util;
mod game;
mod game_data;
mod game_player_data;
mod lichess_util;
mod plotting;
mod ui;
mod util;
mod validator;

fn validate_and_log(game: Game, plotter: &Arc<Plotter>) -> Result<GameData, Error> {
    let game_data = game.validate();
    match game_data {
        Ok(game_data) => {
            Plotter::add_samples(&game_data, plotter)?;
            Ok(game_data)
        }
        Err(e) => {
            plotter.log_error(&format!("{e}"))?;
            Err(Error::GameError)
        }
    }
}

fn parse_batch(
    chunk: impl Iterator<Item = Game>,
    output_file: &mut File,
    plotter: &Arc<Plotter>,
) -> Result<u64, Error> {
    let games: Vec<Game> = chunk.collect();
    let data = games
        .into_par_iter()
        .flat_map(|game| validate_and_log(game, plotter))
        .collect::<Vec<_>>();

    write_batch(output_file, &data)?;
    Ok(data.len() as u64)
}

fn parse_all_games(
    filename: &str,
    game_stream: BufferedReader<impl Read>,
    ui: &Arc<Mutex<UI>>,
    plotter: &Arc<Plotter>,
) -> Result<(), Error> {
    let mut validator = Validator::new();
    let mut progress = 0;
    let mut output_file = util::get_output_file(filename).map_err(Error::FileError)?;

    game_stream
        .into_iter(&mut validator)
        .flatten()
        .chunks(10000)
        .into_iter()
        .try_for_each(|chunk| {
            progress += parse_batch(chunk, &mut output_file, plotter)?;
            UI::update_progress(ui, filename, progress)?;
            plotter.update().to_chess_error(Error::PlottingError)
        })
        .inspect_err(|e| UI::set_error(ui, filename, e))?;

    UI::complete_file(ui, filename, progress)
}

async fn parse_file(url: &str, ui: &Arc<Mutex<UI>>, plotter: &Arc<Plotter>) -> Result<(), Error> {
    if !url.contains("2013-")
        && !url.contains("2014-")
        && !url.contains("2015-")
        && !url.contains("2016-")
        && !url.contains("2017-")
        && !url.contains("2018-")
        && !url.contains("2019-")
    {
        return Ok(());
    }

    let filename = to_local_filename(url)?;
    let callback = |progress: u64| {
        UI::set_downloading(ui, &filename)?;
        UI::update_progress(ui, &filename, progress)
    };
    UI::add_file(ui, &filename)?;
    if !std::path::Path::new(&filename).exists() {
        save_file(url, &filename, callback).await?;
    }
    UI::set_processing(ui, &filename)?;
    let game_stream = from_file(&filename).await?;

    tokio::task::block_in_place(|| parse_all_games(&filename, game_stream, ui, plotter))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let plotter = Plotter::new_arc()?;
    let ui = UI::new_arc()?;

    let mut futures = FuturesUnordered::new();

    for url in get_url_list().await? {
        let ui = ui.clone();
        let plotter = plotter.clone();
        let future = tokio::spawn(async move { parse_file(&url, &ui, &plotter).await })
            .map_err(|_| Error::TokioError);

        futures.push(future);
        if futures.len() >= 10 {
            futures.next().await;
        }
    }
    join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect::<Result<Vec<_>, _>>()?;

    ui.lock()
        .map_err(|_| Error::UIError)?
        .wait_for_exit()
        .map_err(|_| Error::UIError)?;
    Ok(())
}
