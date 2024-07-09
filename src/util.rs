use crate::{game_data::GameData, validator::FirstMove, Result};
use derive_more::AddAssign;
use futures::Stream;
use itertools::Itertools;
use pgn_reader::{BufferedReader, Role, San, SanPlus};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Read,
};
use tokio_util::{
    bytes::Bytes,
    io::{StreamReader, SyncIoBridge},
};

pub trait AndThenErr<U, E> {
    fn and_then_err(self, f: impl Fn(&E) -> std::result::Result<U, E>) -> Self;
}

impl<T, U, E> AndThenErr<U, E> for std::result::Result<T, E> {
    fn and_then_err(self, f: impl FnOnce(&E) -> std::result::Result<U, E>) -> Self {
        self.map_err(|e| f(&e).err().unwrap_or(e))
    }
}

#[derive(Debug, Default, Copy, Clone, AddAssign)]
pub struct Progress {
    pub bytes: u64,
    pub games: u64,
    pub moves: u64,
    pub move_variations: u64,
}

impl Progress {
    pub fn from_bytes(bytes: u64) -> Self {
        Self {
            bytes,
            ..Default::default()
        }
    }
}

impl From<Vec<GameData>> for Progress {
    fn from(data: Vec<GameData>) -> Self {
        Self {
            games: data.len() as u64,
            moves: data.iter().map(|d| u64::from(d.half_moves)).sum(),
            move_variations: data.iter().map(|d| u64::from(d.move_variations)).sum(),
            ..Default::default()
        }
    }
}

pub const fn is_double_disambiguation(san: &San) -> bool {
    match san {
        pgn_reader::San::Normal { file, rank, .. } => file.is_some() && rank.is_some(),
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
    stream: impl Stream<Item = std::result::Result<Bytes, std::io::Error>> + Unpin,
) -> Result<BufferedReader<impl Read>> {
    let reader = StreamReader::new(stream); // this is AsyncRead
    let bridge = SyncIoBridge::new(reader); //  this is Read
    Ok(zstd::Decoder::new(bridge).map(BufferedReader::new)?)
}

pub fn get_output_file(input_file: &str) -> std::result::Result<File, std::io::Error> {
    let raw_filename = input_file
        .split('/')
        .last()
        .and_then(|s| s.split('.').next())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No filename found"))?;
    let filename = format!("./output/{raw_filename}.bin");

    if std::path::Path::new(&filename).exists() {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .append(false)
            .open(&filename)
    } else {
        File::create(&filename)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use shakmaty::{fen::Fen, san::San, CastlingMode, Chess};

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

        let san = San::from_str(&correct_san)?;
        let m = san.to_move(&position)?;
        let san_plus = SanPlus::from_move(position, &m);

        // this fails, the actual value is "Nd5xf6#"
        assert_eq!(san_plus.to_string(), correct_san);
        Ok(())
    }

    #[test]
    fn test_parse_is_double_disambiguation2() -> Result<(), Box<dyn std::error::Error>> {
        let fen: Fen = "8/8/8/8/2n1n1p1/2n3P1/4nkP1/5n1K b - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;
        let correct_san = "N4xg3#";

        let san = San::from_str(&correct_san)?;
        let m = san.to_move(&position)?;
        let san_plus = SanPlus::from_move(position, &m);

        // this fails, the actual value is "Ne4xg3#"
        assert_eq!(san_plus.to_string(), correct_san);
        Ok(())
    }
}
