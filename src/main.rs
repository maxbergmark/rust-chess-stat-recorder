// Validates moves in PGNs.
// Usage: cargo run --release --example validate -- [PGN]...

use std::{env, fs::File, io, mem, slice, sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
}};
use std::sync::atomic::AtomicI64;
use std::time::Instant;

use pgn_reader::{BufferedReader, RawHeader, San, SanPlus, Skip, Visitor};
use shakmaty::{Chess, Position, Move};

#[derive(Debug)]
#[repr(C)]
struct GamePlayerData {
    elo: i16,
    missed_mates: i16,
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
        for possible_move in pos.legal_moves() {
            if possible_move.eq(m) {
                continue;
            }
            let mut board_copy = pos.clone();
            board_copy.play_unchecked(&possible_move);
            if board_copy.is_checkmate() {
                self.missed_mates += 1;
                if possible_move.is_en_passant() {
                    self.missed_en_passant_mates += 1;
                }
            }

            if possible_move.is_en_passant() && !m.is_en_passant() {
                self.declined_en_passants += 1;
            }
        }

    }
}

#[derive(Debug)]
#[repr(C)]
struct GameData {
    white_player: GamePlayerData,
    black_player: GamePlayerData,
}

impl GameData {
    fn new() -> GameData {
        GameData {
            white_player: GamePlayerData::new(),
            black_player: GamePlayerData::new(),
        }
    }
}

struct Game {
    index: usize,
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
    games: usize,
    game: Game,
}

impl Validator {
    fn new() -> Validator {
        Validator {
            games: 0,
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

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        // Support games from a non-standard starting position.
        match key {
            b"WhiteElo" => {
                let s = std::str::from_utf8(value.as_bytes()).unwrap();
                if s.eq("?") {
                    self.game.game_data.white_player.elo = 0;
                } else {
                    self.game.game_data.white_player.elo = s.parse::<i16>().unwrap();
                }
            },
            b"BlackElo" => {
                let s = std::str::from_utf8(value.as_bytes()).unwrap();
                if s.eq("?") {
                    self.game.game_data.black_player.elo = 0;
                } else {
                    self.game.game_data.black_player.elo = s.parse::<i16>().unwrap();
                }
            },
            _ => {}
        }
    }

    fn end_headers(&mut self) -> Skip {
        Skip(!self.game.success)
    }

    fn san(&mut self, san_plus: SanPlus) {
        if self.game.success {
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
        let receive_game_counter = game_counter.clone();


        crossbeam::scope(|scope| {
            scope.spawn(move |_| {
                let mut num_games = 0;
                for game in BufferedReader::new(uncompressed).into_iter(&mut validator) {
                    let game_data = game.expect("io");
                    // println!("Sending: {:?}", game_data.game_data);
                    game_send.send(game_data).unwrap();
                    num_games += 1;
                }
                send_game_counter.store(num_games, Ordering::SeqCst);
            });

            for _ in 0..7 {
                let game_recv = game_recv.clone();
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
                });
            }

            scope.spawn(move |_| {
                let mut v = Vec::new();
                let mut num_games = 0;
                for game_data in result_recv {
                    // println!("Receiving {}: {:?}", num_games, game_data);
                    v.push(game_data);
                    num_games += 1;
                    if num_games == receive_game_counter.load(Ordering::SeqCst) {
                        break;
                    }
                }

                let p = v.as_ptr().cast();
                let l = v.len() * mem::size_of::<GameData>();
                let d = unsafe { slice::from_raw_parts(p, l) };
                std::fs::write("../resources/data.bin", d).unwrap();
            });

        }).unwrap();



        let elapsed = now.elapsed();
        let num_games = game_counter.clone().load(Ordering::SeqCst);
        let speed = num_games as f64 / elapsed.as_secs_f64();
        println!("Elapsed: {:.2?}\nSpeed: {:.2} games/second\nGames: {}", elapsed, speed, num_games);
        let success = success.load(Ordering::SeqCst);
        println!("{}: {}", arg, if success { "success" } else { "errors" });
        complete_success &= success;
    }

    if !complete_success {
        std::process::exit(1);
    }
}