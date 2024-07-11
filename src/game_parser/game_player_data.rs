use pgn_reader::San;
use shakmaty::{Chess, Move, Position};
use std::cmp::min;

use crate::{util::is_double_disambiguation, Error};

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct GamePlayerData {
    pub name: [u8; 20],
    pub elo: i16,
    pub missed_mates: u16,
    pub missed_wins: u16,
    pub en_passant_mates: u8,
    pub missed_en_passant_mates: u8,
    pub en_passants: u8,
    pub declined_en_passants: u8,
    pub double_disambiguation_checkmates: u8,
}

impl GamePlayerData {
    pub fn check_other_move(
        &mut self,
        mut pos: Chess,
        possible_move: &Move,
        is_winner: bool,
        is_checkmate: bool,
    ) {
        pos.play_unchecked(possible_move);
        if pos.is_checkmate() {
            self.missed_mates += !u16::from(is_checkmate);
            self.missed_wins += !u16::from(is_winner);
            if possible_move.is_en_passant() {
                self.missed_en_passant_mates += 1;
            }
        }
    }

    pub fn check_declined_en_passant(&mut self, m: &Move, possible_move: &Move) {
        if possible_move.is_en_passant() && !m.is_en_passant() {
            self.declined_en_passants += 1;
        }
    }

    pub fn check_double_disambiguation(&mut self, position: &Chess, possible_move: &Move) {
        if Self::is_interesting_move(possible_move) {
            let san = San::from_move(position, possible_move);
            if is_double_disambiguation(&san) {
                let mut position = position.clone();
                position.play_unchecked(possible_move);
                if position.is_checkmate() {
                    self.double_disambiguation_checkmates += 1;
                }
            }
        }
    }

    fn is_interesting_move(m: &Move) -> bool {
        m.is_capture() && (m.role() == shakmaty::Role::Bishop || m.role() == shakmaty::Role::Knight)
    }

    pub fn set_elo(&mut self, value: &[u8]) {
        self.elo = std::str::from_utf8(value)
            .map_err(Error::Utf8)
            .and_then(|s| s.parse::<i16>().map_err(Error::ParseInt))
            .unwrap_or(0);
    }

    pub fn set_name(&mut self, value: &[u8]) {
        let l = min(value.len(), 20);
        self.name[..l].clone_from_slice(&value[..l]);
    }
}
