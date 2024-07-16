mod enums;
mod game;
mod game_data;
mod game_player_data;
mod validator;

pub use enums::MoveType;
pub use game::Game;
pub use game_data::{GameData, RareMoveWithLink};
pub use game_player_data::GamePlayerData;
pub use validator::{FirstMove, Validator};

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    extern crate test;
    use enums::{CheckType, GameResult, Termination, TimeControl};
    use pgn_reader::BufferedReader;

    use super::*;

    type Error = Box<dyn std::error::Error>;
    type Result<T> = std::result::Result<T, Error>;

    const DOUBLE_DISAMBIGUATION_GAME: &str = r#"[Event "Rated Rapid game"]
[Site "https://lichess.org/UcZZx10k"]
[Date "2024.06.09"]
[White "Questforrarestmove"]
[Black "Questforraremove"]
[Result "1-0"]
[UTCDate "2024.06.09"]
[UTCTime "07:51:44"]
[WhiteElo "1500"]
[BlackElo "1500"]
[WhiteRatingDiff "+245"]
[BlackRatingDiff "-242"]
[Variant "Standard"]
[TimeControl "600+0"]
[ECO "B01"]
[Opening "Scandinavian Defense: Blackburne-Kloosterboer Gambit"]
[Termination "Normal"]
[Annotator "lichess.org"]

1. e4 d5 2. exd5 c6 { B01 Scandinavian Defense: Blackburne-Kloosterboer Gambit } 3. dxc6 Qc7 4. cxb7 Qb6 5. bxa8=B Qxb2 6. Nc3 Qxc2 7. f4 Na6 8. f5 Qxc1 9. Bxa6 Qxa1 10. Bxc8 Qxa2 11. f6 Qxd2+ 12. Kf1 Qxc3 13. fxg7 Bxg7 14. Nf3 Qxf3+ 15. Qxf3 a5 16. g4 a4 17. g5 a3 18. Qxa3 Nf6 19. g6 Ne4 20. gxh7 Rg8 21. hxg8=B Ng3+ 22. Ke1 Nxh1 23. h4 Bc3+ 24. Qxc3 f5 25. Qf3 Kf8 26. Qxh1 Ke8 27. h5 f4 28. h6 f3 29. h7 f2+ 30. Kxf2 e5 31. h8=Q Ke7 32. Ke2 Kd6 33. Kd2 Kc5 34. Kc1 Kd4 35. Kb1 Ke3 36. Ka1 e4 37. Q1h4 Kf3 38. Qe5 Kg2 39. Qa5 Kf3 40. Qae1 Kg2 41. Bce6 Kf3 42. Bb3 Kg2 43. Bc2 Kf3 44. Bh7 Kg2 45. Bg6 Kf3 46. Bc6 Kg2 47. Bd5 Kf3 48. Bb7 Kg2 49. Bc6 Kf3 50. Bc6xe4# { White wins by checkmate. } 1-0

"#;

    const MINIMAL_GAME: &str = r#"[Event "Rated Rapid game"]
[Site "https://lichess.org/UcZZx10k"]
[Date "2024.06.09"]
[White "Questforrarestmove"]
[Black "Questforraremove"]
[Result "1-0"]
[UTCDate "2024.06.09"]
[UTCTime "07:51:44"]
[WhiteElo "1500"]
[BlackElo "1500"]
[WhiteRatingDiff "+245"]
[BlackRatingDiff "-242"]
[Variant "Standard"]
[TimeControl "600+0"]
[ECO "B01"]
[Opening "Scandinavian Defense: Blackburne-Kloosterboer Gambit"]
[Termination "Normal"]
[Annotator "lichess.org"]

1. f3 e6 2. g4 Qh4 1-0

"#;

    #[test]
    fn test_parser_double_disambiguation() -> Result<()> {
        let reader = BufferedReader::new(DOUBLE_DISAMBIGUATION_GAME.as_bytes());
        let mut validator = Validator::new();
        let game = reader
            .into_iter(&mut validator)
            .next()
            .ok_or("No game found")??;

        let mut expected = GameData {
            white_player: GamePlayerData::default(),
            black_player: GamePlayerData::default(),
            result: GameResult::WhiteWin,
            start_time: 1_717_919_504,
            half_moves: 99,
            move_variations: 2701,
            game_link: *b"UcZZx10k",
            time_control: TimeControl::RapidGame,
            termination: Termination::Normal,
        };

        let expected_rare_moves = vec![
            RareMoveWithLink {
                game_link: "https://lichess.org/UcZZx10k".to_string(),
                san: "Bc6xe4".to_string(),
                ply: 92,
                move_type: MoveType::DoubleDisambiguationCheckmate {
                    was_played: false,
                    is_capture: true,
                    checkmate_type: CheckType::Normal,
                },
            },
            RareMoveWithLink {
                game_link: "https://lichess.org/UcZZx10k".to_string(),
                san: "Bc6xe4".to_string(),
                ply: 98,
                move_type: MoveType::DoubleDisambiguationCheckmate {
                    was_played: true,
                    is_capture: true,
                    checkmate_type: CheckType::Normal,
                },
            },
        ];

        let result = game.validate()?;
        // not testing for these here
        expected.white_player = result.white_player.clone();
        expected.black_player = result.black_player.clone();

        assert_eq!(result, expected);
        assert_eq!(result.get_rare_moves(), expected_rare_moves);
        Ok(())
    }

    #[bench]
    fn bench_parser_game_validate(b: &mut test::Bencher) {
        let reader = BufferedReader::new(DOUBLE_DISAMBIGUATION_GAME.as_bytes());
        let mut validator = Validator::new();
        let games = reader
            .into_iter(&mut validator)
            .flatten()
            .collect::<Vec<_>>();
        b.iter(|| {
            let results = games.clone().into_iter().map(super::game::Game::validate);
            assert_eq!(results.count(), 1);
        });
    }

    #[bench]
    fn bench_parser_pgn_validate(b: &mut test::Bencher) {
        b.iter(|| {
            let reader = BufferedReader::new(DOUBLE_DISAMBIGUATION_GAME.as_bytes());
            let mut validator = Validator::new();
            let res = reader.into_iter(&mut validator).flatten();

            assert_eq!(res.count(), 1);
        });
    }

    #[bench]
    fn bench_parser_game_validate_minimal(b: &mut test::Bencher) {
        let reader = BufferedReader::new(MINIMAL_GAME.as_bytes());
        let mut validator = Validator::new();
        let games = reader
            .into_iter(&mut validator)
            .flatten()
            .collect::<Vec<_>>();
        b.iter(|| {
            let results = games.clone().into_iter().map(super::game::Game::validate);
            assert_eq!(results.count(), 1);
        });
    }

    #[bench]
    fn bench_parser_pgn_validate_minimal(b: &mut test::Bencher) {
        b.iter(|| {
            let reader = BufferedReader::new(MINIMAL_GAME.as_bytes());
            let mut validator = Validator::new();
            let res = reader.into_iter(&mut validator).flatten();

            assert_eq!(res.count(), 1);
        });
    }
}
