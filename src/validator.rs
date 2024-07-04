use crate::error::{Error, ToCrateError};
use crate::game::Game;
use crate::game_data::GameData;
use pgn_reader::{RawHeader, SanPlus, Skip, Visitor};
use shakmaty::Chess;
use std::collections::HashMap;
use std::mem;

pub struct FirstMove {
    pub count: u64,
    pub game_link: String,
    pub first_played: u32,
}

impl FirstMove {
    pub const fn new() -> Self {
        Self {
            count: 0,
            game_link: String::new(),
            first_played: u32::MAX,
        }
    }

    fn update(&mut self, game_link: [u8; 8], start_time: u32) -> Result<(), Error> {
        self.count += 1;
        if start_time < self.first_played {
            self.game_link = std::str::from_utf8(&game_link)
                .to_chess_error(Error::ParsingError(game_link.to_vec()))?
                .to_string();
            self.first_played = start_time;
        }
        Ok(())
    }

    pub fn merge(&mut self, other: &Self) {
        self.count += other.count;
        if other.first_played < self.first_played {
            self.game_link.clone_from(&other.game_link);
            self.first_played = other.first_played;
        }
    }
}

pub struct Validator {
    games: i64,
    // pub move_counter: HashMap<SanPlus, u64>,
    pub move_counter: HashMap<SanPlus, FirstMove>,
    game: Game,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            games: 0,
            move_counter: HashMap::new(),
            game: Game {
                // index: 0,
                position: Chess::default(),
                sans: Vec::new(),
                success: true,
                data: GameData::new(),
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
        let game_data = &mut self.game.data;
        let v = value.as_bytes();
        match key {
            b"WhiteElo" => game_data.white_player.set_elo(v),
            b"BlackElo" => game_data.black_player.set_elo(v),
            b"White" => game_data.white_player.set_name(v),
            b"Black" => game_data.black_player.set_name(v),
            b"Event" => game_data
                .parse_time_control(v)
                .unwrap_or_else(|_| self.game.success = false),
            b"Result" => game_data
                .parse_result(v)
                .unwrap_or_else(|_| self.game.success = false),
            b"Termination" => game_data
                .parse_termination(v)
                .unwrap_or_else(|_| self.game.success = false),
            b"Site" => game_data.parse_site(v),
            b"UTCDate" => game_data
                .parse_date(v)
                .unwrap_or_else(|_| self.game.success = false),
            b"UTCTime" => game_data
                .parse_time(v)
                .unwrap_or_else(|_| self.game.success = false),
            _ => {}
        }
    }

    fn end_headers(&mut self) -> Skip {
        Skip(!self.game.success)
    }

    fn san(&mut self, san_plus: SanPlus) {
        if self.game.success {
            // *self.move_counter.entry(san_plus.clone()).or_insert(0) += 1;
            self.move_counter
                .entry(san_plus.clone())
                .or_insert(FirstMove::new())
                .update(self.game.data.game_link, self.game.data.start_time)
                .unwrap_or_else(|_| self.game.success = false);
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
                // index: self.games,
                position: Chess::default(),
                sans: Vec::with_capacity(80),
                success: true,
                data: GameData::new(),
            },
        )
    }
}
