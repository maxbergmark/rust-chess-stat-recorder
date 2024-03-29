use chrono::naive::{NaiveDate, NaiveTime};

use crate::enums::{GameResult, Termination, TimeControl};
use crate::game_player_data::GamePlayerData;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct GameData {
    pub(crate) white_player: GamePlayerData,
    pub(crate) black_player: GamePlayerData,
    pub(crate) start_time: u32,
    pub(crate) game_link: [u8; 8],
    pub(crate) time_control: TimeControl,
    pub(crate) result: GameResult,
    pub(crate) termination: Termination,
    pub(crate) half_moves: u16,
}

impl GameData {
    pub(crate) fn new() -> GameData {
        GameData {
            white_player: GamePlayerData::new(),
            black_player: GamePlayerData::new(),
            start_time: 0,
            game_link: [0; 8],
            time_control: TimeControl::RatedStandardGame,
            result: GameResult::Unfinished,
            termination: Termination::Unterminated,
            half_moves: 0,
        }
    }

    pub(crate) fn parse_result(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        self.result = match s {
            "1-0" => GameResult::WhiteWin,
            "1/2-1/2" => GameResult::Draw,
            "0-1" => GameResult::BlackWin,
            "*" => GameResult::Unfinished, // some correspondence games take more than a month to complete
            _ => unimplemented!("Result: {}", s),
        }
    }

    pub(crate) fn parse_termination(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        self.termination = match s {
            "Normal" => Termination::Normal,
            "Time forfeit" => Termination::TimeForfeit,
            "Abandoned" => Termination::Abandoned,
            "Unterminated" => Termination::Unterminated,
            "Rules infraction" => Termination::RulesInfraction,
            _ => unimplemented!("Termination: {}", s),
        }
    }

    pub(crate) fn parse_time_control(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        let l = s.split(" ").collect::<Vec<&str>>();

        self.time_control = match l[..] {
            ["Rated", speed, "game"] => match speed {
                "Correspondence" => TimeControl::RatedCorrespondenceGame,
                "Classical" => TimeControl::RatedClassicalGame,
                "Standard" => TimeControl::RatedStandardGame,
                "Rapid" => TimeControl::RatedRapidGame,
                "Blitz" => TimeControl::RatedBlitzGame,
                "Bullet" => TimeControl::RatedBulletGame,
                "UltraBullet" => TimeControl::RatedUltraBulletGame,
                _ => unimplemented!(),
            },
            ["Rated", speed, "tournament", _] => match speed {
                "Correspondence" => TimeControl::RatedCorrespondenceTournament,
                "Classical" => TimeControl::RatedClassicalTournament,
                "Standard" => TimeControl::RatedStandardTournament,
                "Rapid" => TimeControl::RatedRapidTournament,
                "Blitz" => TimeControl::RatedBlitzTournament,
                "Bullet" => TimeControl::RatedBulletTournament,
                "UltraBullet" => TimeControl::RatedUltraBulletTournament,
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        };
    }

    pub(crate) fn parse_site(&mut self, value: &[u8]) {
        let l = value.len();
        self.game_link[..8].clone_from_slice(&value[l - 8..l]);
    }

    pub(crate) fn parse_date(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        let date = NaiveDate::parse_from_str(s, "%Y.%m.%d").expect(s);
        let datetime = date.and_hms_opt(0, 0, 0).unwrap();
        self.start_time += datetime.timestamp() as u32;
    }

    pub(crate) fn parse_time(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        let time = NaiveTime::parse_from_str(s, "%H:%M:%S").unwrap();
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        self.start_time += time.signed_duration_since(midnight).num_seconds() as u32;
    }
}
