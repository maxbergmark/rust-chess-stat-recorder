use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Read,
};

use crate::{game_parser::FirstMove, Result};

use futures::Stream;
use itertools::Itertools;
use pgn_reader::BufferedReader;
use shakmaty::Role;
use shakmaty::{san::San, san::SanPlus};
use tokio_util::{
    bytes::Bytes,
    io::{StreamReader, SyncIoBridge},
};

pub const fn is_double_disambiguation(san: &San) -> bool {
    match san {
        San::Normal { file, rank, .. } => file.is_some() && rank.is_some(),
        _ => false,
    }
}

#[allow(unused)]
pub fn clean_sanplus(san_plus: &SanPlus) -> SanPlus {
    let cleaned_san = match &san_plus.san {
        San::Normal {
            role: Role::Pawn,
            file,
            rank,
            capture,
            to,
            promotion,
        } => San::Normal {
            role: Role::Pawn,
            file: *file,
            rank: *rank,
            capture: *capture,
            to: *to,
            promotion: *promotion,
        },
        San::Normal {
            role,
            file: _,
            rank: _,
            capture,
            to,
            promotion,
        } => San::Normal {
            role: *role,
            file: None,
            rank: None,
            capture: *capture,
            to: *to,
            promotion: *promotion,
        },
        other => other.clone(),
    };
    SanPlus {
        san: cleaned_san,
        suffix: san_plus.suffix,
    }
}

#[allow(unused)]
fn map_to_csv(map: HashMap<SanPlus, FirstMove>) -> String {
    map.into_iter()
        .map(|(san, first_move)| {
            format!(
                "{}, {}, {}, {}",
                san, first_move.count, first_move.first_played, first_move.game_link
            )
        })
        .join("\n")
}

#[allow(unused)]
pub fn save_move_map(
    moves: HashMap<SanPlus, FirstMove>,
    moves_filename: String,
) -> std::io::Result<()> {
    let raw_moves_filename = format!("{moves_filename}.raw");

    let mut cleaned_moves = HashMap::new();
    for (k, v) in &moves {
        let cleaned_key = clean_sanplus(k);
        cleaned_moves
            .entry(cleaned_key)
            .or_insert(FirstMove::new())
            .merge(v);
    }

    let move_counter_str = map_to_csv(cleaned_moves);
    let raw_move_counter_str = map_to_csv(moves);

    std::fs::write(moves_filename, move_counter_str)?;
    std::fs::write(raw_moves_filename, raw_move_counter_str)
}

pub fn to_game_stream(
    stream: impl Stream<Item = std::io::Result<Bytes>> + Unpin,
) -> Result<BufferedReader<impl Read>> {
    let reader = StreamReader::new(stream); // this is AsyncRead
    let bridge = SyncIoBridge::new(reader); //  this is Read
    Ok(zstd::Decoder::new(bridge).map(BufferedReader::new)?)
}

fn raw_file_name(input_file: &str) -> Result<&str> {
    Ok(input_file
        .split('/')
        .last()
        .and_then(|s| s.split('.').next())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No filename found"))?)
}

fn open_or_create_file(filename: &str) -> Result<File> {
    let result = if std::path::Path::new(filename).exists() {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .append(false)
            .open(filename)
    } else {
        File::create(filename)
    };
    Ok(result?)
}

pub fn get_data_output_file(input_file: &str) -> Result<File> {
    let raw_filename = raw_file_name(input_file)?;
    let filename = format!("./output/{raw_filename}.bin");
    open_or_create_file(&filename)
}

pub fn get_move_output_file(input_file: &str) -> Result<File> {
    let raw_filename = raw_file_name(input_file)?;
    let filename = format!("./output/{raw_filename}.moves");
    open_or_create_file(&filename)
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use shakmaty::{fen::Fen, san::San, CastlingMode, Chess, Role};

    type Result<T, E> = std::result::Result<T, E>;

    #[test]
    fn test_is_double_disambiguation() {
        let san = San::Normal {
            role: Role::Pawn,
            file: Some(shakmaty::File::A),
            rank: Some(shakmaty::Rank::First),
            capture: false,
            to: shakmaty::Square::A2,
            promotion: None,
        };
        assert!(is_double_disambiguation(&san));
    }

    #[test]
    fn test_parse_is_double_disambiguation() -> Result<(), Box<dyn std::error::Error>> {
        let fen: Fen = "8/2KN1p2/5p2/3N1B1k/5PNp/7P/7P/8 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;
        let correct_san = "N5xf6#";

        let san = San::from_str(correct_san)?;
        let m = san.to_move(&position)?;
        let san_plus = SanPlus::from_move(position, &m);

        assert_eq!(san_plus.to_string(), correct_san);
        Ok(())
    }

    #[test]
    fn test_parse_is_double_disambiguation2() -> Result<(), Box<dyn std::error::Error>> {
        let fen: Fen = "8/8/8/8/2n1n1p1/2n3P1/4nkP1/5n1K b - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;
        let correct_san = "N4xg3#";

        let san = San::from_str(correct_san)?;
        let m = san.to_move(&position)?;
        let san_plus = SanPlus::from_move(position, &m);

        assert_eq!(san_plus.to_string(), correct_san);
        Ok(())
    }
}
