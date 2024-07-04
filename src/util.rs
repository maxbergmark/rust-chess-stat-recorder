use crate::{error::Error, validator::FirstMove};
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
        .map(|(k, v)| format!("{}, {}, {}, {}", k, v.count, v.first_played, v.game_link))
        // join iterator of strings with newline
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
    stream: impl Stream<Item = Result<Bytes, std::io::Error>> + Unpin,
) -> Result<BufferedReader<impl Read>, Error> {
    let reader = StreamReader::new(stream); // this is AsyncRead
    let bridge = SyncIoBridge::new(reader); //  this is Read
    zstd::Decoder::new(bridge)
        .map(BufferedReader::new)
        .map_err(|_| Error::DecompressionError)
}

pub fn get_output_file(input_file: &str) -> Result<File, std::io::Error> {
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
