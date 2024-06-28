use crate::enums::GameResult;
use crate::error::ChessError;
use crate::game_data::GameData;
use shakmaty::san::San;
use shakmaty::{Chess, Position};
// use std::mem;

#[derive(Debug)]
pub(crate) struct Game {
    // pub(crate) index: i64,
    pub(crate) position: Chess,
    pub(crate) sans: Vec<San>,
    pub(crate) success: bool,
    pub(crate) game_data: GameData,
}

impl Game {
    pub(crate) fn validate(mut self) -> Result<GameData, ChessError> {
        self.sans
            .iter()
            .enumerate()
            .try_for_each(|(half_move_number, san)| {
                Game::parse_move(
                    &mut self.position,
                    &mut self.game_data,
                    half_move_number,
                    san,
                )
            })?;
        // for (half_move_number, san) in self.sans.iter().enumerate() {
        //     let m = san
        //         .to_move(&self.position)
        //         .map_err(|_| ChessError::InvalidMove)?;
        //     // let is_winner = self.check_is_winner(half_move_number);
        //     let player_data = self.game_data.get_player_data(half_move_number);

        //     // player_data.analyze_position(&self.position, &m, is_winner);
        //     self.position.play_unchecked(&m);
        // }
        self.game_data.half_moves = self.sans.len() as u16;
        Ok(self.game_data)
    }

    fn parse_move(
        position: &mut Chess,
        game_data: &mut GameData,
        half_move_number: usize,
        san: &San,
    ) -> Result<(), ChessError> {
        let m = san.to_move(position).map_err(|_| ChessError::InvalidMove)?;
        let is_winner = Game::check_is_winner(&game_data.result, half_move_number);
        let player_data = game_data.get_player_data(half_move_number);

        player_data.analyze_position(position, &m, is_winner);
        position.play_unchecked(&m);
        Ok(())
    }

    fn check_is_winner(result: &GameResult, half_move_number: usize) -> bool {
        match half_move_number % 2 {
            0 => matches!(result, GameResult::WhiteWin),
            _ => matches!(result, GameResult::BlackWin),
        }
    }

    // pub(crate) fn get_game_data(&mut self) -> GameData {
    // mem::replace(&mut self.game_data, GameData::new())
    // }
}
