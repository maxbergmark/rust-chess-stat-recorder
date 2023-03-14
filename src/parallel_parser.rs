use std::{io, mem, slice};
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::time::Instant;
use crossbeam::channel::{Receiver, Sender};
use crossbeam::thread::Scope;
use pgn_reader::BufferedReader;
use crate::game::Game;
use crate::game_data::GameData;
use crate::helpers;
use crate::validator::Validator;

pub (crate) struct ParallelParser {
    filename: String,
    num_threads: i32,
    success: Arc<AtomicBool>,
    game_counter: Arc<AtomicI64>,
}

impl ParallelParser {

    pub(crate) fn new(filename: &String, num_threads: i32) -> ParallelParser {

        ParallelParser {
            filename: filename.clone(),
            num_threads,
            success: Arc::new(AtomicBool::new(true)),
            game_counter: Arc::new(AtomicI64::new(0)),
        }
    }

    fn get_moves_filename(&self) -> String {
        self.filename.clone().replace(".pgn.zst", "") + ".moves"
    }

    fn get_bin_filename(&self) -> String {
        self.filename.clone().replace(".pgn.zst", "") + ".bin"
    }

    fn spawn_parser_thread(&self, scope: &Scope, game_send: Sender<Game>) {

        let filename = self.get_moves_filename();
        let game_counter = self.game_counter.clone();
        let game_stream = BufferedReader::new(self.get_file());

        scope.spawn(move |_| {
            let mut validator = Validator::new();
            let mut num_games = 0;
            for game in game_stream.into_iter(&mut validator) {
                let game_data = game.expect("io");
                game_send.send(game_data).unwrap();
                num_games += 1;
            }
            drop(game_send);
            println!("Parsed {} games", num_games);
            game_counter.store(num_games, Ordering::SeqCst);
            helpers::save_move_map(validator.move_counter, &filename);
        });
    }

    fn spawn_worker_threads(&self, scope: &Scope, game_recv: Receiver<Game>, result_send: Sender<GameData>) {

        for _ in 0..self.num_threads {
            let game_recv: Receiver<Game> = game_recv.clone();
            let result_send = result_send.clone();
            let success = self.success.clone();
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
    }

    fn spawn_collector_thread(&self, scope: &Scope, result_recv: Receiver<GameData>) {
        let filename = self.get_bin_filename();
        scope.spawn(move |_| {
            let v: Vec<GameData> = result_recv.iter().collect();
            let size = mem::size_of::<GameData>();
            println!("Saving {} elements of size {}", v.len(), size);

            let p = v.as_ptr().cast();
            let l = v.len() * size;
            let d = unsafe { slice::from_raw_parts(p, l) };
            std::fs::write(filename, d).unwrap();
        });
    }

    fn get_file(&self) -> Box<dyn io::Read + Send> {
        let uncompressed: Box<dyn io::Read + Send> = if self.filename.ends_with(".zst") {
            let file = File::open(&self.filename).expect("file open");
            Box::new(zstd::Decoder::new(file).expect("zst decoder"))
        } else if self.filename.ends_with(".remote") {
            Box::new(zstd::Decoder::new(io::stdin()).expect("zst stdin decoder"))
        } else {
            let file = File::open(&self.filename).expect("file open");
            Box::new(file)
        };
        uncompressed
    }

    fn print_stats(&self, start_time: Instant) {
        let elapsed = start_time.elapsed();
        let num_games = self.game_counter.load(Ordering::SeqCst);
        let speed = num_games as f64 / elapsed.as_secs_f64();
        let success = self.success.load(Ordering::SeqCst);
        println!("{}: {}", self.filename, if success { "success" } else { "errors" });
        println!("Elapsed: {:.2?}\nSpeed: {:.2} games/second\nGames: {}", elapsed, speed, num_games);
    }

    pub(crate) fn process_file(&self) -> bool {
        let start_time = Instant::now();
        let (game_send, game_recv) = crossbeam::channel::bounded(128);
        let (result_send, result_recv) = crossbeam::channel::bounded(128);

        crossbeam::scope(|scope| {
            self.spawn_parser_thread(scope, game_send);
            self.spawn_worker_threads(scope, game_recv, result_send);
            self.spawn_collector_thread(scope, result_recv);
        }).unwrap();

        self.print_stats(start_time);
        self.success.load(Ordering::SeqCst)
    }
}
