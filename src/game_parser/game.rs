use crate::{Error, Result};
use shakmaty::san::{San, SanError};
use shakmaty::{Chess, Position};

use super::enums::GameResult;
use super::GameData;

#[derive(Debug, Default, Clone)]
pub struct Game {
    pub sans: Vec<San>,
    pub success: bool,
    pub data: GameData,
}

impl Game {
    pub fn validate(mut self) -> Result<GameData> {
        let mut position = Chess::default();
        self.sans
            .iter()
            .enumerate()
            .try_for_each(|(ply, san)| Self::check_move(&mut position, &mut self.data, ply, san))?;
        self.data.half_moves = self.sans.len() as u16;
        Ok(self.data)
    }

    fn check_move(
        position: &mut Chess,
        game_data: &mut GameData,
        ply: usize,
        san: &San,
    ) -> Result<()> {
        let m = san
            .to_move(position)
            .map_err(|err| to_error(game_data, san, err))?;
        let is_winner = Self::check_is_winner(game_data.result, ply);

        game_data.analyze_position(position, ply, &m, is_winner);
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

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    extern crate test;

    use super::*;

    type Error = Box<dyn std::error::Error>;

    #[bench]
    fn bench_check_move(b: &mut test::Bencher) {
        b.iter(|| {
            let mut game_data = GameData::default();
            let position = Chess::default();
            // game has average of 60 moves, 20 legal moves in starting position
            for _ in 0..3 {
                for m in position.legal_moves() {
                    let san = San::from_move(&position, &m);
                    let mut board_copy = position.clone();
                    Game::check_move(&mut board_copy, &mut game_data, 0, &san)?;
                }
            }
            Ok::<(), Error>(())
        });
    }
}
