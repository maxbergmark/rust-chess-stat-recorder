use std::collections::HashMap;
use std::mem;
use pgn_reader::{RawHeader, Skip, Visitor};
use shakmaty::Chess;
use shakmaty::san::SanPlus;
use crate::game::Game;
use crate::game_data::GameData;

pub(crate) struct Validator {
    games: i64,
    pub(crate) move_counter: HashMap<SanPlus, u64>,
    game: Game,
}

impl Validator {
    pub(crate) fn new() -> Validator {
        Validator {
            games: 0,
            move_counter: HashMap::new(),
            game: Game {
                index: 0,
                pos: Chess::default(),
                sans: Vec::new(),
                success: true,
                game_data: GameData::new(),
            },
        }
    }
}

impl Visitor for Validator {
    type Result = Game;

    fn begin_game(&mut self) {
        self.games += 1;
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        match key {
            b"WhiteElo" => self.game.game_data.white_player.set_elo(&value.as_bytes()),
            b"BlackElo" => self.game.game_data.black_player.set_elo(&value.as_bytes()),
            b"White" => self.game.game_data.white_player.set_name(&value.as_bytes()),
            b"Black" => self.game.game_data.black_player.set_name(&value.as_bytes()),
            b"Event" => self.game.game_data.parse_time_control(&value.as_bytes()),
            b"Result" => self.game.game_data.parse_result(&value.as_bytes()),
            b"Termination" => self.game.game_data.parse_termination(&value.as_bytes()),
            b"Site" => self.game.game_data.parse_site(&value.as_bytes()),
            b"UTCDate" => self.game.game_data.parse_date(&value.as_bytes()),
            b"UTCTime" => self.game.game_data.parse_time(&value.as_bytes()),
            _ => {}
        }
    }

    fn end_headers(&mut self) -> Skip {
        Skip(!self.game.success)
    }

    fn san(&mut self, san_plus: SanPlus) {
        if self.game.success {
            *self.move_counter.entry(san_plus.clone()).or_insert(0) += 1;
            self.game.sans.push(san_plus.san);
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn end_game(&mut self) -> Self::Result {
        mem::replace(
            &mut self.game,
            Game {
                index: self.games,
                pos: Chess::default(),
                sans: Vec::with_capacity(80),
                success: true,
                game_data: GameData::new(),
            },
        )
    }
}