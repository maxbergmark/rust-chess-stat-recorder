mod helpers;
mod enums;
mod game;
mod game_player_data;
mod game_data;
mod validator;
mod parallel_parser;
mod multichannel_parser;

use crate::multichannel_parser::MultiChannelParser;

fn main() {
    let parser = MultiChannelParser::new(2, 4);
    parser.start_consumer();
}
