use std::time::{Duration, Instant};

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
    pub initialization_time: Instant,
    pub status: FileStatus,
    pub message: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FileStatus {
    Waiting,
    Downloading { start_time: Instant, file_size: u64 },
    Processing { start_time: Instant },
    Error,
    Done { processing_time: Duration },
    Hidden,
}

impl FileProgress {
    pub fn to_line(&self, s: &str) -> Line {
        let span = Span::styled(s.to_string(), self.style());
        Line::from(vec![span])
    }

    pub fn speed(&self) -> f64 {
        let duration = match self.status {
            FileStatus::Processing { start_time } | FileStatus::Downloading { start_time, .. } => {
                start_time.elapsed()
            }
            FileStatus::Done { processing_time } => processing_time,
            _ => Duration::from_secs(0),
        };
        let elapsed = duration.as_secs_f64();

        if elapsed == 0.0 {
            return 0.0;
        }
        self.progress.games as f64 / elapsed
    }

    pub fn move_speed(&self) -> f64 {
        let duration = match self.status {
            FileStatus::Processing { start_time } => start_time.elapsed(),
            FileStatus::Done { processing_time } => processing_time,
            _ => Duration::from_secs(0),
        };
        let elapsed = duration.as_secs_f64();

        if elapsed == 0.0 {
            return 0.0;
        }
        self.progress.move_variations as f64 / elapsed
    }

    pub fn get_status(&self) -> Line {
        let s = match self.status {
            FileStatus::Downloading { .. } => "Downloading",
            FileStatus::Processing { .. } => "Processing",
            FileStatus::Done { .. } => "Done",
            FileStatus::Waiting => "Waiting",
            FileStatus::Hidden => "Hidden",
            FileStatus::Error => "Error",
        };
        let span = Span::styled(s, self.style());
        Line::from(vec![span])
    }

    pub fn get_message(&self) -> Line {
        let span = Span::styled(self.message.clone().unwrap_or_default(), self.style());
        Line::from(vec![span])
    }

    fn style(&self) -> Style {
        Style::default().fg(self.get_color())
    }

    const fn get_color(&self) -> Color {
        match self.status {
            FileStatus::Waiting => Color::Yellow,
            FileStatus::Downloading { .. } => Color::LightBlue,
            FileStatus::Processing { .. } => Color::White,
            FileStatus::Done { .. } => Color::Green,
            FileStatus::Hidden => Color::DarkGray,
            FileStatus::Error => Color::Red,
        }
    }
}
