use human_format::Formatter;
use itertools::Itertools;
use ratatui::{
    text::{Line, Text},
    widgets::{Block, Paragraph},
};

use super::file_progress::{FileProgress, FileStatus};

pub fn create_lines<'a>(files: &'a [&FileProgress], f: fn(&FileProgress) -> Line) -> Vec<Line<'a>> {
    files
        .iter()
        .filter(|fi| fi.status != FileStatus::Hidden)
        .map(|fi| f(fi))
        .collect_vec()
}

pub fn get_elapsed_time(files: &[&FileProgress]) -> f64 {
    let start_time = files
        .iter()
        .map(|fi| fi.start_time)
        .min()
        .unwrap_or_else(std::time::Instant::now);
    start_time.elapsed().as_secs_f64()
}

pub fn to_paragraph<'a>(lines: Vec<Line<'a>>, title: &'a str) -> Paragraph<'a> {
    Paragraph::new(Text::from(lines)).block(Block::bordered().title(title))
}

pub fn to_human(value: f64) -> String {
    Formatter::new().format(value)
}
