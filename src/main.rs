mod enums;
mod game;
mod game_data;
mod game_player_data;
mod helpers;
mod multichannel_parser;
mod parallel_parser;
mod rabbitmq_handler;
mod validator;

use crate::multichannel_parser::MultiChannelParser;

fn main() {
    let parser = MultiChannelParser::new(2, 4);
    parser.start_consumer();
}
