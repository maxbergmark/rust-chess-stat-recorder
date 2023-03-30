use crate::game::Game;
use crate::game_data::GameData;
use crate::helpers;
use crate::rabbitmq_handler::RabbitMqHandler;
use crate::validator::Validator;
use core_affinity::CoreId;
use crossbeam::channel::{Receiver, Sender};
use crossbeam::thread::Scope;
use pgn_reader::BufferedReader;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{io, mem, slice};

pub(crate) struct ParallelParser {
    filename: Arc<Mutex<String>>,
    channel_id: usize,
    num_threads: usize,
    complete: Arc<AtomicBool>,
    game_counter: Arc<AtomicI64>,
}

impl ParallelParser {
    pub(crate) fn new(channel_id: usize, num_threads: usize) -> ParallelParser {
        ParallelParser {
            filename: Arc::new(Mutex::new(String::new())),
            channel_id,
            num_threads,
            complete: Arc::new(AtomicBool::new(true)),
            game_counter: Arc::new(AtomicI64::new(0)),
        }
    }

    fn get_moves_filename(&self) -> String {
        self.filename.lock().unwrap().replace(".pgn.zst", "") + ".moves"
    }

    fn get_bin_filename(&self) -> String {
        self.filename.lock().unwrap().replace(".pgn.zst", "") + ".bin"
    }

    fn spawn_parser_thread<'a>(&'a self, scope: &Scope<'a>, game_send: Sender<Game>) {
        let game_counter = self.game_counter.clone();

        scope.spawn(move |_| {
            let res = core_affinity::set_for_current(CoreId {
                id: self.channel_id,
            });
            if !res {
                eprintln!("Could not set affinity");
            }
            let game_stream = BufferedReader::new(self.get_file());
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
            helpers::save_move_map(validator.move_counter, self.get_moves_filename());
        });
    }

    fn spawn_worker_threads(
        &self,
        scope: &Scope,
        game_recv: Receiver<Game>,
        result_send: Sender<GameData>,
    ) {
        for _ in 0..self.num_threads {
            let game_recv: Receiver<Game> = game_recv.clone();
            let result_send = result_send.clone();

            scope.spawn(move |_| {
                for mut game in game_recv {
                    let index = game.index;
                    let is_valid = game.validate();
                    if !is_valid {
                        eprintln!("illegal move in game {}", index);
                    } else {
                        let result = game.get_game_data();
                        result_send.send(result).unwrap();
                    }
                }
                drop(result_send);
            });
        }
        drop(result_send);
    }

    fn write_batch(file: &mut File, v: &Vec<GameData>) {
        let p = v.as_ptr().cast();
        let size = mem::size_of::<GameData>();
        let l = v.len() * size;
        let d = unsafe { slice::from_raw_parts(p, l) };
        // println!("Saving {} elements of size {}", v.len(), size);
        file.write(d).expect("Could not write to file");
    }

    fn spawn_collector_thread<'a>(&'a self, scope: &Scope<'a>, result_recv: Receiver<GameData>) {
        let filename = self.get_bin_filename();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(filename)
            .unwrap();

        scope.spawn(move |_| {
            let batch_size = 1000;
            let mut v = Vec::with_capacity(batch_size);
            for game_data in result_recv.iter() {
                v.push(game_data);
                if v.len() >= batch_size {
                    ParallelParser::write_batch(&mut file, &v);
                    v.clear();
                }
            }
            if v.len() > 0 {
                ParallelParser::write_batch(&mut file, &v);
            }
            self.complete.store(true, Ordering::SeqCst);
        });
    }

    fn get_file(&self) -> Box<dyn io::Read + Send> {
        let uncompressed: Box<dyn io::Read + Send> =
            if self.filename.lock().unwrap().ends_with(".zst") {
                let file = File::open(&self.filename.lock().unwrap().clone()).expect("file open");
                Box::new(zstd::Decoder::new(file).expect("zst decoder"))
            } else if self.filename.lock().unwrap().ends_with(".remote") {
                Box::new(zstd::Decoder::new(io::stdin()).expect("zst stdin decoder"))
            } else {
                let file = File::open(&self.filename.lock().unwrap().clone()).expect("file open");
                Box::new(file)
            };
        uncompressed
    }

    fn print_stats(&self, start_time: Instant) {
        let elapsed = start_time.elapsed();
        let num_games = self.game_counter.load(Ordering::SeqCst);
        let speed = num_games as f64 / elapsed.as_secs_f64();

        println!("{}", self.filename.lock().unwrap());
        println!(
            "Elapsed: {:.2?}\nSpeed: {:.2} games/second\nGames: {}",
            elapsed, speed, num_games
        );
    }

    fn consume_file<'a>(&'a self, scope: &Scope<'a>, filename: String) {
        println!(
            "Starting consuming on thread {}: {}",
            self.channel_id, &filename
        );
        *self.filename.lock().unwrap() = filename;
        self.complete.store(false, Ordering::SeqCst);
        let (game_send, game_recv) = crossbeam::channel::bounded(128);
        let (result_send, result_recv) = crossbeam::channel::bounded(128);

        self.spawn_parser_thread(scope, game_send);
        self.spawn_worker_threads(scope, game_recv, result_send);
        self.spawn_collector_thread(scope, result_recv);

    }

    pub(crate) fn create_channel(&self) {
        loop {
            let filename = match RabbitMqHandler::get_filename_from_queue() {
                Some(s) => s,
                None => {
                    println!("queue is empty, exiting");
                    break
                },
            };

            let start_time = Instant::now();
            crossbeam::scope(|scope| {
                self.consume_file(scope, filename);
            })
            .unwrap();
            self.print_stats(start_time);
        }
    }
}
