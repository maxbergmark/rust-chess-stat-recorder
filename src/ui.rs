use std::{
    collections::HashMap,
    io::{self, stdout, Stdout},
    time::Instant,
};

use itertools::Itertools;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

#[derive(Debug)]
pub(crate) struct UI {
    file_info: HashMap<String, FileInfo>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[derive(Debug)]
struct FileInfo {
    filename: String,
    // size: u64,
    pub progress: u64,
    pub start_time: Instant,
    speed: f64,
    status: FileStatus,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FileStatus {
    #[allow(dead_code)]
    Downloading,
    Processing,
    Done,
}

impl UI {
    pub fn new() -> Result<Self, io::Error> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Self::setup()?;
        Ok(UI {
            file_info: HashMap::new(),
            terminal,
        })
    }

    pub fn add_file(&mut self, filename: &str) {
        // get file size from filename
        let filename = filename.to_owned();
        // let size = File::open(&filename).unwrap().metadata().unwrap().len();
        let progress = 0;
        self.file_info.insert(
            filename.clone(),
            FileInfo {
                filename,
                // size,
                progress,
                start_time: Instant::now(),
                speed: 0.0,
                status: FileStatus::Processing,
            },
        );
    }

    pub fn update_progress(&mut self, filename: &String, progress: u64) {
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.progress = progress;
            file_info.speed = progress as f64 / file_info.start_time.elapsed().as_secs_f64();
        }
    }

    pub fn complete_file(&mut self, filename: &String, progress: u64) {
        self.update_progress(filename, progress);
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.status = FileStatus::Done;
        }
    }

    fn setup() -> Result<(), io::Error> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), io::Error> {
        self.terminal
            .draw(|frame| Self::draw(&self.file_info, frame))?;
        if handle_events()? {
            self.exit()?;
        }
        Ok(())
    }

    pub fn exit(&self) -> Result<(), io::Error> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        // exit the program
        std::process::exit(0);
        // Ok(())
    }

    fn draw(file_info_map: &HashMap<String, FileInfo>, frame: &mut Frame) {
        let main_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
        )
        .split(frame.size());
        frame.render_widget(
            Block::new()
                .borders(Borders::TOP)
                .title("Chess game parser"),
            main_layout[0],
        );
        frame.render_widget(
            Block::new().borders(Borders::TOP).title("Status Bar"),
            main_layout[2],
        );

        let inner_layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Percentage(50),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ],
        )
        .split(main_layout[1]);

        let file_info = file_info_map
            .values()
            .sorted_by_key(|fi| fi.filename.clone())
            .collect_vec();

        let mut filenames = file_info.iter().map(|fi| fi.get_filename()).collect_vec();
        let mut progress = file_info.iter().map(|fi| fi.get_progress()).collect_vec();
        let mut speed = file_info.iter().map(|fi| fi.get_speed()).collect_vec();
        let status = file_info.iter().map(|fi| fi.get_status()).collect_vec();

        let start_time = file_info.iter().map(|fi| fi.start_time).min().unwrap();
        let elapsed = start_time.elapsed().as_secs_f64();
        let total_progress = file_info.iter().map(|fi| fi.progress).sum::<u64>();
        filenames.push(Line::raw("Total"));
        progress.push(Line::raw(format!("{:10} games", total_progress)));
        speed.push(Line::raw(format!(
            "{:9.2} games/s",
            total_progress as f64 / elapsed
        )));

        let progress = to_paragraph(progress, "Progress");
        let speed = to_paragraph(speed, "Speed");
        let filenames = to_paragraph(filenames, "Files");
        let status = to_paragraph(status, "Status");

        frame.render_widget(filenames, inner_layout[0]);
        frame.render_widget(progress, inner_layout[1]);
        frame.render_widget(speed, inner_layout[2]);
        frame.render_widget(status, inner_layout[3]);
    }
}

impl FileInfo {
    pub fn get_filename(&self) -> Line {
        let span = Span::styled(
            self.filename.to_string(),
            Style::default().fg(self.get_color()),
        );
        Line::from(vec![span])
    }

    pub fn get_progress(&self) -> Line {
        let span = Span::styled(
            format!("{:10}", self.progress),
            Style::default().fg(self.get_color()),
        );
        Line::from(vec![span])
    }

    pub fn get_speed(&self) -> Line {
        let span = Span::styled(
            format!("{:9.2}", self.speed),
            Style::default().fg(self.get_color()),
        );
        Line::from(vec![span])
    }

    pub fn get_status(&self) -> Line {
        let span = Span::styled(
            format!("{:?}", self.status),
            Style::default().fg(self.get_color()),
        );
        Line::from(vec![span])
    }

    fn get_color(&self) -> Color {
        match self.status {
            FileStatus::Downloading => Color::Yellow,
            FileStatus::Processing => Color::White,
            FileStatus::Done => Color::Green,
        }
    }
}

fn to_paragraph<'a>(lines: Vec<Line<'a>>, title: &'a str) -> Paragraph<'a> {
    Paragraph::new(Text::from(lines)).block(Block::bordered().title(title))
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
