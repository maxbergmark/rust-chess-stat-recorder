// Validates moves in PGNs.
// Usage: cargo run --release --example validate -- [PGN]...

mod helpers;
mod enums;
mod game;
mod game_player_data;
mod game_data;
mod validator;
mod parallel_parser;
mod multichannel_parser;

// #[macro_use(c)]
extern crate cute;

use crate::multichannel_parser::MultiChannelParser;

fn main() {
    let parser = MultiChannelParser::new(2, 4);
    parser.start_consumer();
}
