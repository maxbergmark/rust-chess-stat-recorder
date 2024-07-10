use derive_more::AddAssign;

use crate::game_data::GameData;

#[derive(Debug, Default, Copy, Clone, AddAssign)]
pub struct Progress {
    pub bytes: u64,
    pub games: u64,
    pub moves: u64,
    pub move_variations: u64,
}

impl Progress {
    pub fn from_bytes(bytes: u64) -> Self {
        Self {
            bytes,
            ..Default::default()
        }
    }
}

impl From<Vec<GameData>> for Progress {
    fn from(data: Vec<GameData>) -> Self {
        Self {
            games: data.len() as u64,
            moves: data.iter().map(|d| u64::from(d.half_moves)).sum(),
            move_variations: data.iter().map(|d| u64::from(d.move_variations)).sum(),
            ..Default::default()
        }
    }
}
