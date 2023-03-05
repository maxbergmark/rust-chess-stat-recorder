// Validates moves in PGNs.
// Usage: cargo run --release --example validate -- [PGN]...

mod helpers;
mod enums;
mod game;
mod game_player_data;
mod game_data;
mod validator;
mod parallel_parser;


use std::env;
use crate::parallel_parser::ParallelParser;

fn main() {
    let mut complete_success = true;

    for arg in env::args().skip(1) {
        let parser = ParallelParser::new(&arg, 8);
        let success = parser.process_file();
        complete_success &= success;
    }

    if !complete_success {
        std::process::exit(1);
    }
}
