use shakmaty::{Chess, Move, Position};
use std::cmp::min;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct GamePlayerData {
    pub(crate) name: [u8; 20],
    pub(crate) elo: i16,
    pub(crate) missed_mates: u16,
    pub(crate) missed_wins: u16,
    pub(crate) en_passant_mates: u8,
    pub(crate) missed_en_passant_mates: u8,
    pub(crate) en_passants: u8,
    pub(crate) declined_en_passants: u8,
}

#[allow(unused)]
impl GamePlayerData {
    pub(crate) fn new() -> GamePlayerData {
        GamePlayerData {
            name: [0; 20],
            elo: 0,
            missed_mates: 0,
            missed_wins: 0,
            en_passant_mates: 0,
            missed_en_passant_mates: 0,
            en_passants: 0,
            declined_en_passants: 0,
        }
    }

    pub(crate) fn analyze_position(&mut self, pos: &Chess, m: &Move, is_winner: bool) {
        self.check_move(pos, m);
        self.check_possible_moves(pos, m, is_winner);
    }

    fn check_move(&mut self, pos: &Chess, m: &Move) {
        let is_en_passant = m.is_en_passant();
        let mut board_copy = pos.clone();
        board_copy.play_unchecked(m);
        let is_en_passant_mate = board_copy.is_checkmate() & is_en_passant;
        self.en_passant_mates += is_en_passant_mate as u8;
        self.en_passants += is_en_passant as u8;
    }

    fn check_possible_moves(&mut self, pos: &Chess, m: &Move, is_winner: bool) {
        let mut board_copy = pos.clone();
        board_copy.play_unchecked(m);
        let is_checkmate = board_copy.is_checkmate();

        for possible_move in pos.legal_moves() {
            if possible_move.eq(m) {
                continue;
            }
            let mut board_copy = pos.clone();
            board_copy.play_unchecked(&possible_move);
            if board_copy.is_checkmate() {
                self.missed_mates += !is_checkmate as u16;
                self.missed_wins += !is_winner as u16;
                if possible_move.is_en_passant() {
                    self.missed_en_passant_mates += 1;
                }
            }

            if possible_move.is_en_passant() && !m.is_en_passant() {
                self.declined_en_passants += 1;
            }
        }
    }

    pub(crate) fn set_elo(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        self.elo = if s.len() == 1 {
            0
        } else {
            s.parse::<i16>().unwrap()
        };
    }

    pub(crate) fn set_name(&mut self, value: &[u8]) {
        let l = min(value.len(), 20);
        self.name[..l].clone_from_slice(&value[..l]);
    }
}
