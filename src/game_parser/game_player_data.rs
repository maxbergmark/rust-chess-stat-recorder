use pgn_reader::San;
use shakmaty::{Chess, Move, Position, Role};
use std::cmp::min;

use crate::{util::is_double_disambiguation, Error};

use super::enums::{CheckType, MoveType};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
    pub double_disambiguation_capture_checkmates: u8,
    pub rare_checkmates: Vec<RareMove>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RareMove {
    pub san: String,
    pub ply: u16,
    pub move_type: MoveType,
}

impl GamePlayerData {
    pub fn check_other_move(
        &mut self,
        mut position: Chess,
        possible_move: &Move,
        ply: usize,
        is_winner: bool,
        is_checkmate: bool,
    ) {
        let board_copy = position.clone();
        position.play_unchecked(possible_move);

        if position.is_checkmate() {
            self.missed_mates += u16::from(!is_checkmate);
            self.missed_wins += u16::from(!is_winner);
            if possible_move.is_en_passant() {
                self.missed_en_passant_mates += 1;
            }

            if let Some(rare_move) = Self::check_rare_move(board_copy, possible_move, ply, false) {
                self.rare_checkmates.push(rare_move);
            }
        }
    }

    pub fn check_declined_en_passant(&mut self, m: &Move, possible_move: &Move) {
        if possible_move.is_en_passant() && !m.is_en_passant() {
            self.declined_en_passants += 1;
        }
    }

    pub fn check_rare_move(
        position: Chess,
        m: &Move,
        ply: usize,
        was_played: bool,
    ) -> Option<RareMove> {
        if Self::is_interesting_move(m) {
            let san = San::from_move(&position, m);
            if is_double_disambiguation(&san) {
                Some(RareMove {
                    san: san.to_string(),
                    ply: ply as u16,
                    move_type: MoveType::DoubleDisambiguationCheckmate {
                        is_capture: m.is_capture(),
                        was_played,
                        checkmate_type: Self::is_discovered_check(position, m),
                    },
                })
            } else if m.role() == Role::King && m.is_capture() {
                Some(RareMove {
                    san: san.to_string(),
                    ply: ply as u16,
                    move_type: MoveType::KingCheckmate {
                        is_capture: true,
                        was_played,
                    },
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_discovered_check(mut position: Chess, m: &Move) -> CheckType {
        let sq = m.to();
        position.play_unchecked(m);
        let checkers = position.checkers();
        match (checkers.contains(sq), checkers.count() > 1) {
            (true, true) => CheckType::Double,
            (true, false) => CheckType::Normal,
            (false, _) => CheckType::Discovered,
        }
    }

    fn is_interesting_move(m: &Move) -> bool {
        m.role() == Role::Bishop || m.role() == Role::Knight || m.role() == Role::King
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

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    extern crate test;
    use std::str::FromStr;

    use super::*;
    use pgn_reader::SanPlus;
    use shakmaty::{fen::Fen, san::San, CastlingMode, Chess};

    type Error = Box<dyn std::error::Error>;
    type Result<T> = std::result::Result<T, Error>;

    #[test]
    fn test_is_double_disambiguation() {
        let san = San::Normal {
            role: Role::Pawn,
            file: Some(shakmaty::File::A),
            rank: Some(shakmaty::Rank::First),
            capture: false,
            to: shakmaty::Square::A2,
            promotion: None,
        };
        assert!(is_double_disambiguation(&san));
    }

    #[test]
    fn test_parse_is_double_disambiguation() -> Result<()> {
        let fen: Fen = "N2NQN1k/7p/7B/7R/5N2/7P/6B1/6K1 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;

        let san = San::from_str("Nf8e6#")?;
        let m = san.to_move(&position)?;
        let san_plus = SanPlus::from_move(position, &m);
        let is_double_disambiguation = is_double_disambiguation(&san_plus.san);

        assert!(is_double_disambiguation);
        Ok(())
    }

    #[test]
    fn test_is_discovered_mate() -> Result<()> {
        let fen: Fen = "N2NQN1k/7p/7B/7R/5N2/7P/6B1/6K1 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;

        let san = San::from_str("Nf8e6#")?;
        let m = san.to_move(&position)?;
        let check_type = GamePlayerData::is_discovered_check(position, &m);

        assert_eq!(check_type, CheckType::Discovered);
        Ok(())
    }

    #[test]
    fn test_is_not_discovered_mate() -> Result<()> {
        let fen: Fen = "3NQN1k/4N1Np/7B/5B1R/7P/8/8/6K1 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;

        let san = San::from_str("Nf7#")?;
        let m = san.to_move(&position)?;
        let check_type = GamePlayerData::is_discovered_check(position, &m);

        assert_eq!(check_type, CheckType::Normal);
        Ok(())
    }

    #[test]
    fn test_is_discovered_double_mate() -> Result<()> {
        let fen: Fen = "3NQN1k/7p/4N2B/3N3R/7P/8/6B1/6K1 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;

        let san = San::from_str("Ng6#")?;
        let m = san.to_move(&position)?;
        let check_type = GamePlayerData::is_discovered_check(position, &m);

        assert_eq!(check_type, CheckType::Double);
        Ok(())
    }

    #[test]
    fn test_is_normal_double_disambiguation_mate() -> Result<()> {
        let fen: Fen = "1k6/2N1N3/1K6/NNN1N3/8/8/8/8 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;

        let san = San::from_str("Ne5c6#")?;
        let m = san.to_move(&position)?;
        let check_type = GamePlayerData::is_discovered_check(position, &m);

        assert_eq!(check_type, CheckType::Normal);
        Ok(())
    }

    #[test]
    fn test_check_rare_move() -> Result<()> {
        let fen: Fen = "1k6/2N1N3/1K6/NNN1N3/8/8/8/8 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;

        let san = San::from_str("Ne5c6#")?;
        let m = san.to_move(&position)?;
        let rare_move = GamePlayerData::check_rare_move(position, &m, 123, true);
        let expected = RareMove {
            san: "Ne5c6".to_string(),
            ply: 123,
            move_type: MoveType::DoubleDisambiguationCheckmate {
                was_played: true,
                is_capture: false,
                checkmate_type: CheckType::Normal,
            },
        };

        assert_eq!(rare_move, Some(expected));
        Ok(())
    }

    #[bench]
    fn bench_check_other_move(b: &mut test::Bencher) {
        b.iter(|| {
            let mut game_player_data = GamePlayerData::default();
            let position = Chess::default();
            // game has average of 2000 move variations, 20 legal moves in starting position
            for _ in 0..100 {
                for m in position.legal_moves() {
                    game_player_data.check_other_move(position.clone(), &m, 0, false, false);
                }
            }
        });
    }

    #[bench]
    fn bench_check_rare_move(b: &mut test::Bencher) {
        b.iter(|| {
            let position = Chess::default();
            // game has average of 2000 move variations, 20 legal moves in starting position
            for _ in 0..100 {
                for m in position.legal_moves() {
                    GamePlayerData::check_rare_move(position.clone(), &m, 0, false);
                }
            }
        });
    }

    #[bench]
    fn bench_check_rare_move_single_none(b: &mut test::Bencher) {
        let position = Chess::default();
        let m = position.legal_moves()[0].clone();
        b.iter(|| {
            GamePlayerData::check_rare_move(position.clone(), &m, 0, false);
        });
    }

    #[bench]
    fn bench_check_rare_move_single_knight(b: &mut test::Bencher) -> Result<()> {
        let position = Chess::default();

        let san = San::from_str("Nc3")?;
        let m = san.to_move(&position)?;

        b.iter(|| {
            GamePlayerData::check_rare_move(position.clone(), &m, 0, false);
        });
        Ok(())
    }

    #[bench]
    fn bench_check_rare_move_single_some(b: &mut test::Bencher) -> Result<()> {
        let fen: Fen = "N2NQN1k/7p/7B/7R/5N2/7P/6B1/6K1 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;

        let san = San::from_str("Nf8e6#")?;
        let m = san.to_move(&position)?;

        b.iter(|| {
            GamePlayerData::check_rare_move(position.clone(), &m, 0, false);
        });
        Ok(())
    }

    #[bench]
    fn bench_copy_board(b: &mut test::Bencher) -> Result<()> {
        let fen: Fen = "8/2KN1p2/5p2/3N1B1k/5PNp/7P/7P/8 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;

        b.iter(|| position.clone());
        Ok(())
    }

    #[bench]
    fn bench_parse_san(b: &mut test::Bencher) -> Result<()> {
        let position = Chess::default();

        let san = San::from_str("Nc3")?;
        let m = san.to_move(&position)?;

        b.iter(|| San::from_move(&position, &m));
        Ok(())
    }
    #[bench]
    fn bench_parse_san_double_disambiguation(b: &mut test::Bencher) -> Result<()> {
        let fen: Fen = "N2NQN1k/7p/7B/7R/5N2/7P/6B1/6K1 w - -".parse()?;
        let position: Chess = fen.into_position(CastlingMode::Standard)?;

        let san = San::from_str("Nf8e6#")?;
        let m = san.to_move(&position)?;

        b.iter(|| San::from_move(&position, &m));
        Ok(())
    }
}
