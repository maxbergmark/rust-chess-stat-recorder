use std::time::Instant;

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::util::Progress;

use super::FileInfo;

#[derive(Debug)]
pub struct FileProgress {
    pub file_info: FileInfo,
    pub progress: Progress,
    pub start_time: Instant,
    pub status: FileStatus,
    pub message: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FileStatus {
    Waiting,
    Downloading,
    Processing,
    Error,
    Done,
    Hidden,
}

impl FileProgress {
    pub fn to_line(&self, s: &str) -> Line {
        let span = Span::styled(s.to_string(), Style::default().fg(self.get_color()));
        Line::from(vec![span])
    }

    pub fn speed(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        self.progress.games as f64 / elapsed
    }

    pub fn move_speed(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        self.progress.move_variations as f64 / elapsed
    }

    pub fn get_status(&self) -> Line {
        let span = Span::styled(
            format!("{:?}", self.status),
            Style::default().fg(self.get_color()),
        );
        Line::from(vec![span])
    }

    pub fn get_message(&self) -> Line {
        let span = Span::styled(
            self.message.clone().unwrap_or_default(),
            Style::default().fg(self.get_color()),
        );
        Line::from(vec![span])
    }

    const fn get_color(&self) -> Color {
        match self.status {
            FileStatus::Waiting => Color::Yellow,
            FileStatus::Downloading => Color::LightBlue,
            FileStatus::Processing => Color::White,
            FileStatus::Done => Color::Green,
            FileStatus::Hidden => Color::DarkGray,
            FileStatus::Error => Color::Red,
        }
    }
}
