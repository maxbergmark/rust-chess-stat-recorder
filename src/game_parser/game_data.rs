use chrono::naive::{NaiveDate, NaiveTime};
use shakmaty::{Chess, Move, Position};

use crate::error::Error;

use super::{
    enums::{GameResult, Termination, TimeControl},
    GamePlayerData,
};

#[derive(Debug, Copy, Clone, Default)]
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

impl GameData {
    pub fn analyze_position(&mut self, pos: &Chess, ply: usize, m: &Move, is_winner: bool) {
        self.check_move(pos, ply, m);
        self.check_possible_moves(pos, ply, m, is_winner);
    }

    fn check_move(&mut self, pos: &Chess, ply: usize, m: &Move) {
        let is_en_passant = m.is_en_passant();
        let mut board_copy = pos.clone();
        board_copy.play_unchecked(m);
        let is_en_passant_mate = board_copy.is_checkmate() & is_en_passant;
        let player_data = self.get_player_data(ply);
        player_data.en_passant_mates += u8::from(is_en_passant_mate);
        player_data.en_passants += u8::from(is_en_passant);
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
                player_data.check_other_move(pos.clone(), possible_move, is_winner, is_checkmate);
                player_data.check_declined_en_passant(m, possible_move);
                player_data.check_double_disambiguation(pos, possible_move);
            });
    }

    #[allow(unused)]
    pub const fn is_en_passant_mate(&self) -> bool {
        self.white_player.en_passant_mates > 0 || self.black_player.en_passant_mates > 0
    }

    pub const fn has_double_disambiguation(&self) -> bool {
        self.white_player.double_disambiguation_checkmates > 0
            || self.black_player.double_disambiguation_checkmates > 0
    }

    pub fn get_formatted_game_link(&self) -> Result<String, Error> {
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

    pub fn parse_result(&mut self, value: &[u8]) -> Result<(), Error> {
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

    pub fn parse_termination(&mut self, value: &[u8]) -> Result<(), Error> {
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

    pub fn parse_time_control(&mut self, value: &[u8]) -> Result<(), Error> {
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

    fn parse_rated_time_control(speed: &str) -> Result<TimeControl, Error> {
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

    fn parse_tournament_time_control(speed: &str) -> Result<TimeControl, Error> {
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

    pub fn parse_date(&mut self, value: &[u8]) -> Result<(), Error> {
        let s = std::str::from_utf8(value)?;
        let date = NaiveDate::parse_from_str(s, "%Y.%m.%d")?;
        let datetime = date.and_hms_opt(0, 0, 0).ok_or_else(|| value.to_vec())?;
        self.start_time += datetime.and_utc().timestamp() as u32;
        Ok(())
    }

    pub fn parse_time(&mut self, value: &[u8]) -> Result<(), Error> {
        let s = std::str::from_utf8(value)?;
        let time = NaiveTime::parse_from_str(s, "%H:%M:%S")?;
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).ok_or_else(|| value.to_vec())?;
        self.start_time += time.signed_duration_since(midnight).num_seconds() as u32;
        Ok(())
    }
}
