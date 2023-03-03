// Validates moves in PGNs.
// Usage: cargo run --release --example validate -- [PGN]...

mod helpers;

use std::{env, fs::File, io, mem, slice, sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
}};

use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
use std::time::Instant;

use pgn_reader::{BufferedReader, RawHeader, San, SanPlus, Skip, Visitor};
use shakmaty::{Chess, Position, Move};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct GamePlayerData {
    elo: i16,
    missed_mates: u16,
    missed_wins: u16, // TODO: implement
    en_passant_mates: u8,
    missed_en_passant_mates: u8,
    en_passants: u8,
    declined_en_passants: u8,
}

impl GamePlayerData {
    fn new() -> GamePlayerData {
        GamePlayerData{
            elo: 0,
            missed_mates: 0,
            missed_wins: 0,
            en_passant_mates: 0,
            missed_en_passant_mates: 0,
            en_passants: 0,
            declined_en_passants: 0,
        }
    }

    fn analyze_position(&mut self, pos: &Chess, m: &Move) {
        self.check_move(pos, m);
        self.check_possible_moves(pos, m);
    }

    fn check_move(&mut self, pos: &Chess, m: &Move) {
        let is_en_passant = m.is_en_passant();
        let mut board_copy = pos.clone();
        board_copy.play_unchecked(&m);
        let is_en_passant_mate = board_copy.is_checkmate() & is_en_passant;
        self.en_passant_mates += is_en_passant_mate as u8;
        self.en_passants += is_en_passant as u8;

    }

    fn check_possible_moves(&mut self, pos: &Chess, m: &Move) {
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
                if possible_move.is_en_passant() {
                    self.missed_en_passant_mates += 1;
                }
            }

            if possible_move.is_en_passant() && !m.is_en_passant() {
                self.declined_en_passants += 1;
            }
        }
    }

    fn set_elo(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        self.elo = if s.len() == 1 {0} else {s.parse::<i16>().unwrap()};
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct GameData {
    white_player: GamePlayerData,
    black_player: GamePlayerData,
    time_control: u8,
    result: u8,
    termination: u8,
}

impl GameData {
    fn new() -> GameData {
        GameData {
            white_player: GamePlayerData::new(),
            black_player: GamePlayerData::new(),
            time_control: 0,
            result: 0,
            termination: 0,
        }
    }

    fn parse_result(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        self.result = match s {
            "1-0" => 1,
            "1/2-1/2" => 2,
            "0-1" => 3,
            "*" => 4,
            _ => unimplemented!("Result: {}", s)
        }
    }

    fn parse_termination(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        self.termination = match s {
            "Normal" => 1,
            "Time forfeit" => 2,
            "Abandoned" => 3,
            "Unterminated" => 4,
            "Rules infraction" => 5,
            _ => unimplemented!("Termination: {}", s)
        }
    }


    fn parse_time_control(&mut self, value: &[u8]) {
        let s = std::str::from_utf8(value).unwrap();
        let l = s.split(" ").collect::<Vec<&str>>();

        self.time_control = match l[..] {
            ["Rated", speed, "game"] => match speed {
                "Correspondence" => 1,
                "Classical" => 2,
                "Standard" => 3,
                "Rapid" => 4,
                "Blitz" => 5,
                "Bullet" => 6,
                "UltraBullet" => 7,
                _ => unimplemented!(),
            },
            ["Rated", speed, "tournament", _] => match speed {
                "Correspondence" => 10,
                "Classical" => 11,
                "Standard" => 12,
                "Rapid" => 13,
                "Blitz" => 14,
                "Bullet" => 15,
                "UltraBullet" => 16,
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        };
    }
}

struct Game {
    index: i64,
    pos: Chess,
    sans: Vec<San>,
    success: bool,
    game_data: GameData,

}

impl Game {

    fn validate(&mut self) -> bool {
        self.success && {
            for (i, san) in self.sans.iter().enumerate() {
                let m = match san.to_move(&self.pos) {
                    Ok(m) => m,
                    Err(_) => return false,
                };
                if i % 2 == 0 {
                    self.game_data.white_player.analyze_position(&self.pos, &m);
                } else {
                    self.game_data.black_player.analyze_position(&self.pos, &m);
                }
                self.pos.play_unchecked(&m);
            }
            true
        }
    }

    fn get_game_data(&mut self) -> GameData {
        mem::replace(
            &mut self.game_data,
            GameData::new()
        )
    }

}

struct Validator {
    games: i64,
    move_counter: HashMap<SanPlus, u64>,
    game: Game,
}

impl Validator {
    fn new() -> Validator {
        Validator {
            games: 0,
            move_counter: HashMap::new(),
            game: Game {
                index: 0,
                pos: Chess::default(),
                sans: Vec::new(),
                success: true,
                game_data: GameData::new(),
            },
        }
    }
}

impl Visitor for Validator {
    type Result = Game;

    fn begin_game(&mut self) {
        self.games += 1;
    }
/*
--    [Event "Rated Blitz game"]
    [Site "https://lichess.org/WY6NhS1w"]
    [White "Daniluch"]
    [Black "amgad55"]
    [Result "0-1"]
    [UTCDate "2017.12.31"]
    [UTCTime "23:03:16"]
--    [WhiteElo "1449"]
--    [BlackElo "1474"]
    [WhiteRatingDiff "-10"]
    [BlackRatingDiff "+10"]
    [ECO "A40"]
    [Opening "Horwitz Defense"]
    [TimeControl "300+0"]
    [Termination "Normal"]
*/

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        // Support games from a non-standard starting position.
        match key {
            b"WhiteElo" => self.game.game_data.white_player.set_elo(&value.as_bytes()),
            b"BlackElo" => self.game.game_data.black_player.set_elo(&value.as_bytes()),
            b"Event" => self.game.game_data.parse_time_control(&value.as_bytes()),
            b"Result" => self.game.game_data.parse_result(&value.as_bytes()),
            b"Termination" => self.game.game_data.parse_termination(&value.as_bytes()),
            _ => {}
        }
    }

    fn end_headers(&mut self) -> Skip {
        Skip(!self.game.success)
    }

    fn san(&mut self, san_plus: SanPlus) {
        if self.game.success {
            // let cleaned_san = helpers::clean_sanplus(&san_plus);
            *self.move_counter.entry(san_plus.clone()).or_insert(0) += 1;
            self.game.sans.push(san_plus.san);
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn end_game(&mut self) -> Self::Result {
        mem::replace(
            &mut self.game,
            Game {
                index: self.games,
                pos: Chess::default(),
                sans: Vec::with_capacity(80),
                success: true,
                game_data: GameData::new(),
            },
        )
    }
}

fn main() {
    let mut complete_success = true;

    for arg in env::args().skip(1) {
        let success = Arc::new(AtomicBool::new(true));
        let game_counter = Arc::new(AtomicI64::new(0));
        let now = Instant::now();


        let file = File::open(&arg).expect("fopen");

        let uncompressed: Box<dyn io::Read + Send> = if arg.ends_with(".zst") {
            Box::new(zstd::Decoder::new(file).expect("zst decoder"))
        } else {
            Box::new(file)
        };

        let mut validator = Validator::new();
        let (game_send, game_recv) = crossbeam::channel::bounded(128);
        let (result_send, result_recv) = crossbeam::channel::bounded(128);
        let send_game_counter = game_counter.clone();
        // let receive_game_counter = game_counter.clone();


        crossbeam::scope(|scope| {

            let filename = arg.clone();
            let moves_filename = filename.clone().replace(".pgn.zst", ".moves");
            let result_filename = filename.clone().replace(".pgn.zst", ".bin");
            let num_threads = 8;

            scope.spawn(move |_| {
                let mut num_games = 0;
                for game in BufferedReader::new(uncompressed).into_iter(&mut validator) {
                    let game_data = game.expect("io");
                    game_send.send(game_data).unwrap();
                    num_games += 1;
                }
                drop(game_send);
                println!("Parsed {} games", num_games);
                send_game_counter.store(num_games, Ordering::SeqCst);
                helpers::save_move_map(validator.move_counter, &moves_filename);
            });

            for _ in 0..num_threads {
                let game_recv: crossbeam::channel::Receiver<Game> = game_recv.clone();
                let result_send = result_send.clone();
                let success = success.clone();
                scope.spawn(move |_| {
                    for mut game in game_recv {
                        let index = game.index;
                        let is_valid = game.validate();
                        if !is_valid {
                            eprintln!("illegal move in game {}", index);
                            success.store(false, Ordering::SeqCst);
                        }
                        let result = game.get_game_data();
                        result_send.send(result).unwrap();
                    }
                    drop(result_send);
                });
            }

            drop(result_send);

            scope.spawn(move |_| {
                let v: Vec<GameData> = result_recv.iter().collect();
                println!("Saving {} elements", v.len());

                let p = v.as_ptr().cast();
                let l = v.len() * mem::size_of::<GameData>();
                let d = unsafe { slice::from_raw_parts(p, l) };
                std::fs::write(result_filename, d).unwrap();
            });

        }).unwrap();



        let elapsed = now.elapsed();
        let num_games = game_counter.clone().load(Ordering::SeqCst);
        let speed = num_games as f64 / elapsed.as_secs_f64();

        println!("Elapsed: {:.2?}\nSpeed: {:.2} games/second\nGames: {}", elapsed, speed, num_games);
        let success = success.load(Ordering::SeqCst);
        println!("{}: {}", &arg, if success { "success" } else { "errors" });
        // println!("{:?}", move_counter);
        complete_success &= success;
    }

    if !complete_success {
        std::process::exit(1);
    }
}