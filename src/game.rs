use std::mem;
use shakmaty::{Chess, Position};
use shakmaty::san::San;
use crate::game_data::GameData;
use crate::enums::GameResult;

pub (crate) struct Game {
    pub(crate) index: i64,
    pub(crate) pos: Chess,
    pub(crate) sans: Vec<San>,
    pub(crate) success: bool,
    pub(crate) game_data: GameData,

}

impl Game {

    pub(crate) fn validate(&mut self) -> bool {
        for (i, san) in self.sans.iter().enumerate() {
            let m = match san.to_move(&self.pos) {
                Ok(m) => m,
                Err(_) => return false,
            };
            let player_data = match i % 2 {
                0 => &mut self.game_data.white_player,
                _ => &mut self.game_data.black_player
            };
            let is_winner = match i % 2 {
                0 => matches!(self.game_data.result, GameResult::WhiteWin),
                _ => matches!(self.game_data.result, GameResult::BlackWin)
            };
            player_data.analyze_position(&self.pos, &m, is_winner);
            self.pos.play_unchecked(&m);
        }
        self.game_data.half_moves = self.sans.len() as u16;
        true
    }

    pub(crate) fn get_game_data(&mut self) -> GameData {
        mem::replace(
            &mut self.game_data,
            GameData::new()
        )
    }

}