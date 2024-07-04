use crate::enums::GameResult;
use crate::error::Error;
use crate::game_data::GameData;
use shakmaty::san::{San, SanError};
use shakmaty::{Chess, Position};

#[derive(Debug)]
pub struct Game {
    pub position: Chess,
    pub sans: Vec<San>,
    pub success: bool,
    pub data: GameData,
}

impl Game {
    pub fn validate(mut self) -> Result<GameData, Error> {
        self.sans
            .iter()
            .enumerate()
            .try_for_each(|(half_move_number, san)| {
                Self::parse_move(&mut self.position, &mut self.data, half_move_number, san)
            })?;
        self.data.half_moves = self.sans.len() as u16;
        Ok(self.data)
    }

    fn parse_move(
        position: &mut Chess,
        game_data: &mut GameData,
        half_move_number: usize,
        san: &San,
    ) -> Result<(), Error> {
        let m = san
            .to_move(position)
            .map_err(|err| to_error(game_data, san, err))?;
        let is_winner = Self::check_is_winner(game_data.result, half_move_number);
        let player_data = game_data.get_player_data(half_move_number);

        player_data.analyze_position(position, &m, is_winner);
        position.play_unchecked(&m);
        Ok(())
    }

    fn check_is_winner(result: GameResult, ply: usize) -> bool {
        match ply % 2 {
            0 => result == GameResult::WhiteWin,
            _ => result == GameResult::BlackWin,
        }
    }
}

fn to_error(game_data: &GameData, san: &San, err: SanError) -> Error {
    let game_link = game_data.get_formatted_game_link();
    match game_link {
        Ok(game_link) => {
            let message = format!("Invalid move: {san:?} in game {game_link}");
            Error::InvalidMove(err, message)
        }
        Err(e) => e,
    }
}
