use crate::validator::FirstMove;
use pgn_reader::{Role, San, SanPlus};
use std::collections::HashMap;

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

fn map_to_csv(map: HashMap<SanPlus, FirstMove>) -> String {
    map.into_iter()
        .map(|(k, v)| {
            format!(
                "{}, {}, {}, {}",
                k.to_string(),
                v.count,
                v.first_played,
                v.game_link
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

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
