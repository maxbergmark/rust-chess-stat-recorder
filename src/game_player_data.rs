use shakmaty::{Chess, Move, Position};
use std::cmp::min;

#[derive(Debug, Copy, Clone)]
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
}

#[allow(unused)]
impl GamePlayerData {
    pub const fn new() -> Self {
        Self {
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

    pub fn analyze_position(&mut self, pos: &Chess, m: &Move, is_winner: bool) {
        self.check_move(pos, m);
        self.check_possible_moves(pos, m, is_winner);
    }

    fn check_move(&mut self, pos: &Chess, m: &Move) {
        let is_en_passant = m.is_en_passant();
        let mut board_copy = pos.clone();
        board_copy.play_unchecked(m);
        let is_en_passant_mate = board_copy.is_checkmate() & is_en_passant;
        self.en_passant_mates += u8::from(is_en_passant_mate);
        self.en_passants += u8::from(is_en_passant);
    }

    fn check_possible_moves(&mut self, pos: &Chess, m: &Move, is_winner: bool) {
        let mut board_copy = pos.clone();
        board_copy.play_unchecked(m);
        let is_checkmate = board_copy.is_checkmate();
        self.check_other_moves(pos, m, is_winner, is_checkmate);
    }

    fn check_other_moves(&mut self, pos: &Chess, m: &Move, is_winner: bool, is_checkmate: bool) {
        pos.legal_moves()
            .iter()
            .filter(|&possible_move| !possible_move.eq(m))
            .for_each(|possible_move| {
                self.check_other_move(pos.clone(), possible_move, is_winner, is_checkmate);
                self.check_declined_en_passant(m, possible_move);
            });
    }

    fn check_other_move(
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

    fn check_declined_en_passant(&mut self, m: &Move, possible_move: &Move) {
        if possible_move.is_en_passant() && !m.is_en_passant() {
            self.declined_en_passants += 1;
        }
    }

    pub fn set_elo(&mut self, value: &[u8]) {
        self.elo = std::str::from_utf8(value)
            .map_err(|_| ())
            .and_then(|s| s.parse::<i16>().map_err(|_| ()))
            .unwrap_or(0);
    }

    pub fn set_name(&mut self, value: &[u8]) {
        let l = min(value.len(), 20);
        self.name[..l].clone_from_slice(&value[..l]);
    }
}
