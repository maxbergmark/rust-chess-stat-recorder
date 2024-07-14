use std::fmt::Display;

use chrono::naive::{NaiveDate, NaiveTime};
use shakmaty::{Chess, Move, Position};

use crate::Result;

use super::{
    enums::{GameResult, Termination, TimeControl},
    game_player_data::RareMove,
    GamePlayerData, MoveType,
};

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct GameData {
    pub white_player: GamePlayerData,
    pub black_player: GamePlayerData,
    pub start_time: u32,
    pub move_variations: u32,
    pub game_link: [u8; 8],
    pub time_control: TimeControl,
    pub result: GameResult,
    pub termination: Termination,
    pub half_moves: u16,
}

pub struct RareMoveWithLink {
    pub game_link: String,
    pub san: String,
    pub ply: u16,
    pub move_type: MoveType,
}

impl Display for RareMoveWithLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let move_number_str = format!(
            "{:3}{}",
            self.ply / 2 + 1,
            if self.ply % 2 == 0 { ".  " } else { "..." }
        );
        let san_plus = format!("{}#", self.san);
        write!(
            f,
            "{move_number_str} {san_plus:7},{},{}",
            self.move_type, self.game_link
        )
    }
}

impl RareMoveWithLink {
    pub fn new(game_link: String, rare_move: &RareMove) -> Self {
        Self {
            game_link,
            san: rare_move.san.clone(),
            ply: rare_move.ply,
            move_type: rare_move.move_type.clone(),
        }
    }
}

impl GameData {
    pub fn analyze_position(&mut self, pos: &Chess, ply: usize, m: &Move, is_winner: bool) {
        self.check_move(pos, ply, m);
        self.check_possible_moves(pos, ply, m, is_winner);
    }

    pub fn get_rare_moves(&self) -> Vec<RareMoveWithLink> {
        self.white_player
            .rare_checkmates
            .iter()
            .chain(self.black_player.rare_checkmates.iter())
            .map(move |rare_move| {
                RareMoveWithLink::new(
                    self.get_formatted_game_link().unwrap_or_default(),
                    rare_move,
                )
            })
            .collect()
    }

    fn check_move(&mut self, position: &Chess, ply: usize, m: &Move) {
        let is_en_passant = m.is_en_passant();
        let mut board_copy = position.clone();
        board_copy.play_unchecked(m);
        let is_checkmate = board_copy.is_checkmate();
        let is_en_passant_mate = is_checkmate & is_en_passant;
        let player_data = self.get_player_data(ply);
        player_data.en_passant_mates += u8::from(is_en_passant_mate);
        player_data.en_passants += u8::from(is_en_passant);
        if is_checkmate {
            if let Some(rare_move) = GamePlayerData::check_rare_move(position.clone(), m, ply, true)
            {
                player_data.rare_checkmates.push(rare_move);
            }
        }
    }

    fn check_possible_moves(&mut self, pos: &Chess, ply: usize, m: &Move, is_winner: bool) {
        let mut board_copy = pos.clone();
        board_copy.play_unchecked(m);
        let is_checkmate = board_copy.is_checkmate();
        self.check_other_moves(pos, ply, m, is_winner, is_checkmate);
    }

    fn check_other_moves(
        &mut self,
        pos: &Chess,
        ply: usize,
        m: &Move,
        is_winner: bool,
        is_checkmate: bool,
    ) {
        let possible_moves = pos.legal_moves();
        self.move_variations += possible_moves.len() as u32;
        let player_data = self.get_player_data(ply);

        possible_moves
            .iter()
            .filter(|&possible_move| !possible_move.eq(m))
            .for_each(|possible_move| {
                player_data.check_other_move(
                    pos.clone(),
                    possible_move,
                    ply,
                    is_winner,
                    is_checkmate,
                );
                player_data.check_declined_en_passant(m, possible_move);
            });
    }

    #[allow(unused)]
    pub const fn is_en_passant_mate(&self) -> bool {
        self.white_player.en_passant_mates > 0 || self.black_player.en_passant_mates > 0
    }

    pub fn get_formatted_game_link(&self) -> Result<String> {
        Ok(format!(
            "https://lichess.org/{}",
            std::str::from_utf8(&self.game_link)?
        ))
    }

    pub fn get_player_data(&mut self, half_move_number: usize) -> &mut GamePlayerData {
        match half_move_number % 2 {
            0 => &mut self.white_player,
            _ => &mut self.black_player,
        }
    }

    pub fn parse_result(&mut self, value: &[u8]) -> Result<()> {
        let s = std::str::from_utf8(value)?;
        self.result = match s {
            "1-0" => GameResult::WhiteWin,
            "1/2-1/2" => GameResult::Draw,
            "0-1" => GameResult::BlackWin,
            "*" => GameResult::Unfinished, // some correspondence games take more than a month to complete
            _ => Err(value.to_vec())?,
        };
        Ok(())
    }

    pub fn parse_termination(&mut self, value: &[u8]) -> Result<()> {
        let s = std::str::from_utf8(value)?;
        self.termination = match s {
            "Normal" => Termination::Normal,
            "Time forfeit" => Termination::TimeForfeit,
            "Abandoned" => Termination::Abandoned,
            "Unterminated" => Termination::Unterminated,
            "Rules infraction" => Termination::RulesInfraction,
            _ => Err(value.to_vec())?,
        };
        Ok(())
    }

    pub fn parse_time_control(&mut self, value: &[u8]) -> Result<()> {
        let s = std::str::from_utf8(value)?;
        let l = s.split(' ').collect::<Vec<&str>>();

        self.time_control = match l[..] {
            ["Rated", speed, "game"] => Self::parse_rated_time_control(speed)?,
            ["Rated", speed, "tournament", _] => Self::parse_tournament_time_control(speed)?,
            [speed, "swiss", _] => Self::parse_tournament_time_control(speed)?,
            _ => Err(value.to_vec())?,
        };
        Ok(())
    }

    fn parse_rated_time_control(speed: &str) -> Result<TimeControl> {
        Ok(match speed {
            "Correspondence" => TimeControl::CorrespondenceGame,
            "Classical" => TimeControl::ClassicalGame,
            "Standard" => TimeControl::StandardGame,
            "Rapid" => TimeControl::RapidGame,
            "Blitz" => TimeControl::BlitzGame,
            "Bullet" => TimeControl::BulletGame,
            "UltraBullet" => TimeControl::UltraBulletGame,
            _ => Err(speed.to_string())?,
        })
    }

    fn parse_tournament_time_control(speed: &str) -> Result<TimeControl> {
        Ok(match speed {
            "Correspondence" => TimeControl::CorrespondenceTournament,
            "Classical" => TimeControl::ClassicalTournament,
            "Standard" => TimeControl::StandardTournament,
            "Rapid" => TimeControl::RapidTournament,
            "Blitz" => TimeControl::BlitzTournament,
            "Bullet" => TimeControl::BulletTournament,
            "UltraBullet" => TimeControl::UltraBulletTournament,
            _ => Err(speed.to_string())?,
        })
    }

    pub fn parse_site(&mut self, value: &[u8]) {
        let l = value.len();
        self.game_link[..8].clone_from_slice(&value[l - 8..l]);
    }

    pub fn parse_date(&mut self, value: &[u8]) -> Result<()> {
        let s = std::str::from_utf8(value)?;
        let date = NaiveDate::parse_from_str(s, "%Y.%m.%d")?;
        let datetime = date.and_hms_opt(0, 0, 0).ok_or_else(|| value.to_vec())?;
        self.start_time += datetime.and_utc().timestamp() as u32;
        Ok(())
    }

    pub fn parse_time(&mut self, value: &[u8]) -> Result<()> {
        let s = std::str::from_utf8(value)?;
        let time = NaiveTime::parse_from_str(s, "%H:%M:%S")?;
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).ok_or_else(|| value.to_vec())?;
        self.start_time += time.signed_duration_since(midnight).num_seconds() as u32;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::game_parser::enums::CheckType;

    use super::*;

    type Error = Box<dyn std::error::Error>;
    type Result<T> = std::result::Result<T, Error>;

    #[test]
    fn test_display_rare_move() -> Result<()> {
        let game_link = "https://lichess.org/123".to_string();
        let rare_move = RareMove {
            san: "Nf8e6".to_string(),
            ply: 10,
            move_type: MoveType::DoubleDisambiguationCheckmate {
                was_played: true,
                is_capture: false,
                checkmate_type: CheckType::Double,
            },
        };
        let rare_move_with_link = RareMoveWithLink::new(game_link, &rare_move);
        assert_eq!(
            rare_move_with_link.to_string(),
            "  6.   Nf8e6# ,DD 2 ,https://lichess.org/123"
        );
        Ok(())
    }

    #[test]
    fn test_display_rare_move_odd_ply() -> Result<()> {
        let game_link = "https://lichess.org/123".to_string();
        let rare_move = RareMove {
            san: "Nf8e6".to_string(),
            ply: 123,
            move_type: MoveType::DoubleDisambiguationCheckmate {
                was_played: true,
                is_capture: false,
                checkmate_type: CheckType::Double,
            },
        };
        let rare_move_with_link = RareMoveWithLink::new(game_link, &rare_move);
        assert_eq!(
            rare_move_with_link.to_string(),
            " 62... Nf8e6# ,DD 2 ,https://lichess.org/123"
        );
        Ok(())
    }
}
