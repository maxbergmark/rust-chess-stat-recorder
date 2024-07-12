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
        .map(|fi| fi.initialization_time)
        .min()
        .unwrap_or_else(std::time::Instant::now);
    start_time.elapsed().as_secs_f64()
}

pub fn to_paragraph<'a>(lines: Vec<Line<'a>>, title: &'a str) -> Paragraph<'a> {
    Paragraph::new(Text::from(lines)).block(Block::bordered().title(title))
}

fn format_below_1000(value: f64, suffix: Option<&str>) -> Option<String> {
    let suffix = suffix.unwrap_or("");
    if value < 10.0 {
        Some(format!("{value:.3}{suffix}"))
    } else if value < 100.0 {
        Some(format!("{value:.2}{suffix}"))
    } else if value < 1000.0 {
        Some(format!("{value:.1}{suffix}"))
    } else {
        None
    }
}

pub fn to_human(value: f64) -> String {
    format_below_1000(value, None)
        .or_else(|| format_below_1000(value / 1e3, Some("k")))
        .or_else(|| format_below_1000(value / 1e6, Some("M")))
        .or_else(|| format_below_1000(value / 1e9, Some("G")))
        .or_else(|| format_below_1000(value / 1e12, Some("T")))
        .unwrap_or_else(|| format!("{:.1}T", value / 1e12))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_human() {
        assert_eq!(to_human(0.0), "0.000");
        assert_eq!(to_human(1.0), "1.000");
        assert_eq!(to_human(10.0), "10.00");
        assert_eq!(to_human(999.0), "999.0");
        assert_eq!(to_human(1_000.0), "1.000k");
        assert_eq!(to_human(10_000.0), "10.00k");
        assert_eq!(to_human(100_000.0), "100.0k");
        assert_eq!(to_human(1_000_000.0), "1.000M");
        assert_eq!(to_human(1_000_000_000.0), "1.000G");
        assert_eq!(to_human(1_000_000_000_000.0), "1.000T");
        assert_eq!(to_human(1_000_000_000_000_000.0), "1000.0T");
    }
}
