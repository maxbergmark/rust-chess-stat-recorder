use std::collections::HashMap;
use pgn_reader::{Role, San, SanPlus};

pub (crate) fn clean_sanplus(san_plus: &SanPlus) ->  SanPlus {
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

pub (crate) fn save_move_map(moves: HashMap<SanPlus, u64>, moves_filename: &str) {
    println!("Number of unique moves: {}", moves.len());

    let mut cleaned_moves = HashMap::new();
    for (k, v) in moves.iter() {
        let cleaned_key = clean_sanplus(k);
        *cleaned_moves.entry(cleaned_key).or_insert(0) += v;
    }

    let move_counter_str  = cleaned_moves.into_iter()
        .map(|(k, v)| {format!("{}: {}", k.to_string(), v)})
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(moves_filename, move_counter_str).unwrap();
}