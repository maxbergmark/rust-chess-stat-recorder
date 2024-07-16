use atomic_time::AtomicInstant;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use crate::{
    config::Config,
    game_parser::{GameData, GamePlayerData, RareMoveWithLink},
    Error, Result,
};

pub struct Plotter {
    rec: rerun::RecordingStream,
    elo_hist: Vec<AtomicI64>,
    missed_wins_hist: Vec<AtomicI64>,
    en_passant_hist: Vec<AtomicI64>,
    declined_en_passant_hist: Vec<AtomicI64>,
    half_moves_hist: Vec<AtomicI64>,
    last_update: AtomicInstant,
    update_interval: Duration,
}

impl Plotter {
    fn from_config(config: &Config) -> Result<Self> {
        let port = config.port.unwrap_or(9876);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from(config.rerun_ip)), port);
        let rec = rerun::RecordingStreamBuilder::new("chess_analysis_evelyn")
            .connect_opts(addr, Some(Duration::from_secs(1)))?;
        // .spawn()?;

        Ok(Self {
            rec,
            elo_hist: Self::get_vec(4000),
            missed_wins_hist: Self::get_vec(4000),
            en_passant_hist: Self::get_vec(4000),
            declined_en_passant_hist: Self::get_vec(4000),
            half_moves_hist: Self::get_vec(602),
            last_update: AtomicInstant::now(),
            update_interval: config.update_interval,
        })
    }

    fn add_player_samples(player_data: &GamePlayerData, plotter: &Self) {
        let elo = player_data.elo;
        Self::add_sample(&plotter.elo_hist, elo);
        Self::add_percentage_sample(&plotter.missed_wins_hist, elo, player_data.missed_wins);
        Self::add_percentage_sample(&plotter.en_passant_hist, elo, player_data.en_passants);
        Self::add_percentage_sample(
            &plotter.declined_en_passant_hist,
            elo,
            player_data.declined_en_passants,
        );
    }

    pub fn add_samples(game_data: &GameData, plotter: &Self) {
        Self::add_player_samples(&game_data.white_player, plotter);
        Self::add_player_samples(&game_data.black_player, plotter);
        Self::add_sample(&plotter.half_moves_hist, game_data.half_moves as i16);
    }

    pub fn log_rare_move(plotter: &Self, rare_move: &RareMoveWithLink) -> Result<()> {
        let message = rare_move.to_string();
        plotter.info(&message, None)?;
        Ok(())
    }

    fn get_vec(n: usize) -> Vec<AtomicI64> {
        (0..n).map(|_| AtomicI64::new(0)).collect()
    }

    pub fn new_arc(config: &Config) -> Result<std::sync::Arc<Self>> {
        Self::from_config(config).map(Arc::new)
    }

    pub fn add_sample(histogram: &[AtomicI64], val: impl Into<i64>) {
        let bucket = val.into() as usize;
        histogram[bucket].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn add_percentage_sample(
        histogram: &[AtomicI64],
        elo: impl Into<i64>,
        val: impl Into<i64>,
    ) {
        let bucket = elo.into() as usize;
        histogram[bucket].fetch_add(val.into(), std::sync::atomic::Ordering::Relaxed);
    }

    pub fn update(&self) -> Result<()> {
        let prev = self.last_update.load(Ordering::Relaxed);
        if prev.elapsed() < self.update_interval {
            return Ok(());
        }
        self.last_update.store(Instant::now(), Ordering::Relaxed);

        let elo_buckets = Self::to_buckets(&self.elo_hist);
        self.plot(&elo_buckets, "elo")?;
        self.plot(&Self::to_buckets(&self.half_moves_hist), "half_moves")?;

        self.plot_percentage(
            &self.missed_wins_hist,
            &elo_buckets,
            "missed_win_percentage",
        )?;
        self.plot_percentage(&self.en_passant_hist, &elo_buckets, "en_passant_percentage")?;
        self.plot_percentage(
            &self.declined_en_passant_hist,
            &elo_buckets,
            "declined_en_passant_percentage",
        )?;
        Ok(())
    }

    fn to_buckets(atomics: &[AtomicI64]) -> Vec<f64> {
        atomics
            .iter()
            .map(|x| x.load(std::sync::atomic::Ordering::Relaxed) as f64)
            .collect::<Vec<_>>()
    }

    fn percentage(a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter().zip(b.iter()).map(|(x, y)| *x / *y).collect()
    }

    fn plot_percentage(
        &self,
        histogram: &[AtomicI64],
        elo_buckets: &[f64],
        name: &str,
    ) -> Result<()> {
        let buckets = Self::to_buckets(histogram);
        let buckets = Self::percentage(&buckets, elo_buckets);
        self.plot(&buckets, name)
    }

    fn plot(&self, buckets: &Vec<f64>, name: &str) -> Result<()> {
        let chart = rerun::BarChart::new(buckets.as_slice());
        self.rec.log(name, &chart)?;
        Ok(())
    }

    pub fn info(&self, message: &str, level: Option<&str>) -> Result<()> {
        let level = level.unwrap_or(rerun::TextLogLevel::INFO);
        let log = rerun::TextLog::new(message).with_level(level);
        Ok(self.rec.log("logs", &log)?)
    }

    pub fn error(&self, message: &str) -> Result<()> {
        self.info(message, Some(rerun::TextLogLevel::ERROR))
    }

    pub fn log_error(&self, err: &Error) -> Result<()> {
        self.error(&err.to_string())
    }
}
