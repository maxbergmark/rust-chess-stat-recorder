use chrono::naive::{NaiveDate, NaiveTime};

use crate::enums::{GameResult, Termination, TimeControl};
use crate::error::{Error, ToCrateError};
use crate::game_player_data::GamePlayerData;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct GameData {
    pub white_player: GamePlayerData,
    pub black_player: GamePlayerData,
    pub start_time: u32,
    pub game_link: [u8; 8],
    pub time_control: TimeControl,
    pub result: GameResult,
    pub termination: Termination,
    pub half_moves: u16,
}

impl GameData {
    pub const fn new() -> Self {
        Self {
            white_player: GamePlayerData::new(),
            black_player: GamePlayerData::new(),
            start_time: 0,
            game_link: [0; 8],
            time_control: TimeControl::StandardGame,
            result: GameResult::Unfinished,
            termination: Termination::Unterminated,
            half_moves: 0,
        }
    }

    pub const fn is_en_passant_mate(&self) -> bool {
        self.white_player.en_passant_mates > 0 || self.black_player.en_passant_mates > 0
    }

    pub fn get_formatted_game_link(&self) -> Result<String, Error> {
        Ok(format!(
            "https://lichess.org/{}",
            std::str::from_utf8(&self.game_link)
                .to_chess_error(Error::ParsingError(self.game_link.to_vec()))?
        ))
    }

    pub fn get_player_data(&mut self, half_move_number: usize) -> &mut GamePlayerData {
        match half_move_number % 2 {
            0 => &mut self.white_player,
            _ => &mut self.black_player,
        }
    }

    pub fn parse_result(&mut self, value: &[u8]) -> Result<(), Error> {
        let s = std::str::from_utf8(value).to_chess_error(Error::ParsingError(value.to_vec()))?;
        self.result = match s {
            "1-0" => GameResult::WhiteWin,
            "1/2-1/2" => GameResult::Draw,
            "0-1" => GameResult::BlackWin,
            "*" => GameResult::Unfinished, // some correspondence games take more than a month to complete
            _ => Err(Error::ParsingError(value.to_vec()))?,
        };
        Ok(())
    }

    pub fn parse_termination(&mut self, value: &[u8]) -> Result<(), Error> {
        let s = std::str::from_utf8(value).to_chess_error(Error::ParsingError(value.to_vec()))?;
        self.termination = match s {
            "Normal" => Termination::Normal,
            "Time forfeit" => Termination::TimeForfeit,
            "Abandoned" => Termination::Abandoned,
            "Unterminated" => Termination::Unterminated,
            "Rules infraction" => Termination::RulesInfraction,
            _ => Err(Error::ParsingError(value.to_vec()))?,
        };
        Ok(())
    }

    pub fn parse_time_control(&mut self, value: &[u8]) -> Result<(), Error> {
        let s = std::str::from_utf8(value).to_chess_error(Error::ParsingError(value.to_vec()))?;
        let l = s.split(' ').collect::<Vec<&str>>();

        self.time_control = match l[..] {
            ["Rated", speed, "game"] => Self::parse_rated_time_control(speed)?,
            ["Rated", speed, "tournament", _] => Self::parse_tournament_time_control(speed)?,
            [speed, "swiss", _] => Self::parse_tournament_time_control(speed)?,
            _ => Err(Error::ParsingError(value.to_vec()))?,
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
            _ => Err(Error::ParsingError(speed.as_bytes().to_vec()))?,
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
            _ => Err(Error::ParsingError(speed.as_bytes().to_vec()))?,
        })
    }

    pub fn parse_site(&mut self, value: &[u8]) {
        let l = value.len();
        self.game_link[..8].clone_from_slice(&value[l - 8..l]);
    }

    pub fn parse_date(&mut self, value: &[u8]) -> Result<(), Error> {
        let s = std::str::from_utf8(value).to_chess_error(Error::ParsingError(value.to_vec()))?;
        let date = NaiveDate::parse_from_str(s, "%Y.%m.%d")
            .to_chess_error(Error::ParsingError(value.to_vec()))?;
        let datetime = date
            .and_hms_opt(0, 0, 0)
            .ok_or(Error::ParsingError(value.to_vec()))?;
        self.start_time += datetime.and_utc().timestamp() as u32;
        Ok(())
    }

    pub fn parse_time(&mut self, value: &[u8]) -> Result<(), Error> {
        let s = std::str::from_utf8(value).to_chess_error(Error::ParsingError(value.to_vec()))?;
        let time = NaiveTime::parse_from_str(s, "%H:%M:%S")
            .to_chess_error(Error::ParsingError(value.to_vec()))?;
        let midnight =
            NaiveTime::from_hms_opt(0, 0, 0).ok_or(Error::ParsingError(value.to_vec()))?;
        self.start_time += time.signed_duration_since(midnight).num_seconds() as u32;
        Ok(())
    }
}
