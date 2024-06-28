use crate::{error::ChessError, validator::FirstMove};
use futures::Stream;
use itertools::Itertools;
use pgn_reader::{BufferedReader, Role, San, SanPlus};
use std::{collections::HashMap, io::Read};
use tokio_util::{
    bytes::Bytes,
    io::{StreamReader, SyncIoBridge},
};

#[allow(unused)]
pub(crate) fn clean_sanplus(san_plus: &SanPlus) -> SanPlus {
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
pub(crate) fn save_move_map(moves: HashMap<SanPlus, FirstMove>, moves_filename: String) {
    println!("Number of unique moves: {}", moves.len());
    let raw_moves_filename = moves_filename.clone() + ".raw";

    let mut cleaned_moves = HashMap::new();
    for (k, v) in moves.iter() {
        let cleaned_key = clean_sanplus(k);
        cleaned_moves
            .entry(cleaned_key)
            .or_insert(FirstMove::new())
            .merge(v);
    }

    let move_counter_str = map_to_csv(cleaned_moves);
    let raw_move_counter_str = map_to_csv(moves);

    std::fs::write(moves_filename, move_counter_str).unwrap();
    std::fs::write(raw_moves_filename, raw_move_counter_str).unwrap();
}

pub(crate) fn to_game_stream(
    stream: impl Stream<Item = Result<Bytes, std::io::Error>> + Unpin,
) -> Result<BufferedReader<impl Read>, ChessError> {
    let reader = StreamReader::new(stream); // this is AsyncRead
    let bridge = SyncIoBridge::new(reader); //  this is Read
    zstd::Decoder::new(bridge)
        .map(BufferedReader::new)
        .map_err(|_| ChessError::DecompressionError)
}
