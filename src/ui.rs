use std::{
    borrow::Cow,
    collections::HashMap,
    io::{self, stdout, Stdout},
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use itertools::Itertools;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{
            disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::error::Error;

pub trait UserInterface {
    // fn new() -> Result<Self, io::Error>;
    fn add_file(&mut self, filename: &str);
    fn set_downloading(&mut self, filename: &str);
    fn set_processing(&mut self, filename: &str);
    fn hide_file(&mut self, filename: &str) -> Result<(), io::Error>;
    fn set_error(&mut self, filename: &str, err: &Error);
    fn update_progress(&mut self, filename: &str, progress: u64) -> Result<(), io::Error>;
    fn complete_file(&mut self, filename: &str, progress: u64) -> Result<(), io::Error>;
    fn wait_for_exit(&mut self) -> Result<(), io::Error>;
    fn exit(&mut self) -> Result<(), io::Error>;
}

pub enum UI {
    BoxUI(BoxUI),
    Empty,
}

#[derive(Debug)]
pub struct BoxUI {
    file_info: HashMap<String, FileInfo>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    last_update: Instant,
}

#[derive(Debug)]
struct FileInfo {
    filename: String,
    pub progress: u64,
    pub start_time: Instant,
    speed: f64,
    status: FileStatus,
    message: Option<String>,
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

#[allow(unused)]
impl UI {
    pub fn new() -> Result<Self, io::Error> {
        BoxUI::new().map(UI::BoxUI)
    }

    #[allow(clippy::unnecessary_wraps)]
    pub const fn new_empty() -> Result<Self, io::Error> {
        Ok(Self::Empty)
    }

    pub fn new_arc() -> Result<Arc<Mutex<Self>>, Error> {
        Self::new()
            .map(Mutex::new)
            .map(Arc::new)
            .map_err(|_| Error::UIError)
    }

    fn perform_ui_action(
        ui_mutex: &Arc<Mutex<Self>>,
        action: impl FnOnce(&mut Self) -> Result<(), std::io::Error>,
    ) -> Result<(), Error> {
        ui_mutex
            .lock()
            .map_err(|_| Error::UIError)
            .and_then(|mut ui| action(&mut ui).map_err(|_| Error::UIError))
    }

    pub fn update_progress(
        ui_mutex: &Arc<Mutex<Self>>,
        filename: &str,
        progress: u64,
    ) -> Result<(), Error> {
        Self::perform_ui_action(ui_mutex, |ui| ui.update_progress(filename, progress))
    }

    pub fn complete_file(
        ui_mutex: &Arc<Mutex<Self>>,
        filename: &str,
        progress: u64,
    ) -> Result<(), Error> {
        Self::perform_ui_action(ui_mutex, |ui| ui.complete_file(filename, progress))?;
        thread::sleep(Duration::from_secs(5));
        Self::perform_ui_action(ui_mutex, |ui| ui.hide_file(filename))
    }

    pub fn set_downloading(ui_mutex: &Arc<Mutex<Self>>, filename: &str) -> Result<(), Error> {
        Self::perform_ui_action(ui_mutex, |ui| {
            ui.set_downloading(filename);
            Ok(())
        })
    }

    pub fn set_processing(ui_mutex: &Arc<Mutex<Self>>, filename: &str) -> Result<(), Error> {
        Self::perform_ui_action(ui_mutex, |ui| {
            ui.set_processing(filename);
            Ok(())
        })
    }

    pub fn add_file(ui_mutex: &Arc<Mutex<Self>>, filename: &str) -> Result<(), Error> {
        Self::perform_ui_action(ui_mutex, |ui| {
            ui.add_file(filename);
            Ok(())
        })
    }

    pub fn set_error(ui_mutex: &Arc<Mutex<Self>>, filename: &str, err: &Error) {
        Self::perform_ui_action(ui_mutex, |ui| {
            ui.set_error(filename, err);
            Ok(())
        });
    }
}

impl UserInterface for UI {
    fn add_file(&mut self, filename: &str) {
        if let Self::BoxUI(ui) = self {
            ui.add_file(filename);
        }
    }

    fn set_downloading(&mut self, filename: &str) {
        if let Self::BoxUI(ui) = self {
            ui.set_downloading(filename);
        }
    }

    fn set_processing(&mut self, filename: &str) {
        if let Self::BoxUI(ui) = self {
            ui.set_processing(filename);
        }
    }

    fn set_error(&mut self, filename: &str, err: &Error) {
        if let Self::BoxUI(ui) = self {
            ui.set_error(filename, err);
        }
    }

    fn hide_file(&mut self, filename: &str) -> Result<(), io::Error> {
        if let Self::BoxUI(ui) = self {
            ui.hide_file(filename)
        } else {
            Ok(())
        }
    }

    fn update_progress(&mut self, filename: &str, progress: u64) -> Result<(), io::Error> {
        if let Self::BoxUI(ui) = self {
            ui.update_progress(filename, progress)
        } else {
            Ok(())
        }
    }

    fn complete_file(&mut self, filename: &str, progress: u64) -> Result<(), io::Error> {
        if let Self::BoxUI(ui) = self {
            ui.complete_file(filename, progress)
        } else {
            Ok(())
        }
    }

    fn wait_for_exit(&mut self) -> Result<(), io::Error> {
        match self {
            Self::BoxUI(ui) => ui.wait_for_exit(),
            Self::Empty => Ok(()),
        }
    }

    #[allow(dead_code)]
    fn exit(&mut self) -> Result<(), io::Error> {
        match self {
            Self::BoxUI(ui) => ui.exit(),
            Self::Empty => Ok(()),
        }
    }
}

impl BoxUI {
    pub fn new() -> Result<Self, io::Error> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Self::setup()?;
        Ok(Self {
            file_info: HashMap::new(),
            terminal,
            last_update: Instant::now(),
        })
    }

    fn setup() -> Result<(), io::Error> {
        enable_raw_mode()?;
        // clear the screen
        stdout().execute(Clear(ClearType::All))?;
        stdout().execute(EnterAlternateScreen)?;
        Ok(())
    }

    fn update(&mut self) -> Result<(), io::Error> {
        if self.last_update.elapsed().as_millis() < 500 {
            return Ok(());
        }
        self.last_update = Instant::now();
        self.terminal
            .draw(|frame| Self::draw(&self.file_info, frame))?;
        if handle_events()? {
            self.exit()?;
        }
        Ok(())
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
                Constraint::Percentage(25),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(8),
                Constraint::Percentage(47),
            ],
        )
        .split(main_layout[1]);

        Self::draw_sections(file_info_map, frame, &inner_layout);
    }

    fn draw_sections(
        file_info_map: &HashMap<String, FileInfo>,
        frame: &mut Frame,
        inner_layout: &Rc<[ratatui::layout::Rect]>,
    ) {
        let file_info = file_info_map
            .values()
            .sorted_by_key(|fi| fi.filename.clone())
            .collect_vec();

        let mut filenames = create_lines(&file_info, FileInfo::get_filename);
        let mut progress = create_lines(&file_info, FileInfo::get_progress);
        let mut speed = create_lines(&file_info, FileInfo::get_speed);
        let status = create_lines(&file_info, FileInfo::get_status);
        let messages = create_lines(&file_info, FileInfo::get_message);
        add_total(&file_info, &mut filenames, &mut progress, &mut speed);

        let progress = to_paragraph(progress, "Progress");
        let speed = to_paragraph(speed, "Speed");
        let filenames = to_paragraph(filenames, "Files");
        let status = to_paragraph(status, "Status");
        let messages = to_paragraph(messages, "Messages");

        frame.render_widget(filenames, inner_layout[0]);
        frame.render_widget(progress, inner_layout[1]);
        frame.render_widget(speed, inner_layout[2]);
        frame.render_widget(status, inner_layout[3]);
        frame.render_widget(messages, inner_layout[4]);
    }
}

impl UserInterface for BoxUI {
    fn add_file(&mut self, filename: &str) {
        let filename = filename.to_owned();
        let progress = 0;
        self.file_info.insert(
            filename.clone(),
            FileInfo {
                filename,
                progress,
                start_time: Instant::now(),
                speed: 0.0,
                status: FileStatus::Waiting,
                message: None,
            },
        );
    }

    fn set_downloading(&mut self, filename: &str) {
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.status = FileStatus::Downloading;
        }
    }

    fn set_processing(&mut self, filename: &str) {
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.status = FileStatus::Processing;
        }
    }

    fn set_error(&mut self, filename: &str, err: &Error) {
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.status = FileStatus::Error;
            file_info.message = Some(err.to_string());
        }
    }

    fn hide_file(&mut self, filename: &str) -> Result<(), io::Error> {
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.status = FileStatus::Hidden;
        }
        self.update()
    }

    fn update_progress(&mut self, filename: &str, progress: u64) -> Result<(), io::Error> {
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.progress = progress;
            file_info.speed = progress as f64 / file_info.start_time.elapsed().as_secs_f64();
        }
        self.update()
    }

    fn complete_file(&mut self, filename: &str, progress: u64) -> Result<(), io::Error> {
        self.update_progress(filename, progress)?;
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.status = FileStatus::Done;
        }
        self.update()
    }

    fn wait_for_exit(&mut self) -> Result<(), io::Error> {
        self.update()?;
        while !handle_events()? {
            thread::sleep(std::time::Duration::from_millis(500));
        }
        self.exit()
    }

    fn exit(&mut self) -> Result<(), io::Error> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        self.terminal.clear()?;
        // exit the program
        std::process::exit(0);
        // Ok(())
    }
}

fn create_lines<'a>(file_info: &'a [&FileInfo], f: fn(&FileInfo) -> Line) -> Vec<Line<'a>> {
    file_info
        .iter()
        .filter(|fi| fi.status != FileStatus::Hidden)
        .map(|fi| f(fi))
        .collect_vec()
}

fn add_total(
    file_info: &[&FileInfo],
    filenames: &mut Vec<Line>,
    progress: &mut Vec<Line>,
    speed: &mut Vec<Line>,
) {
    let start_time = file_info
        .iter()
        .map(|fi| fi.start_time)
        .min()
        .unwrap_or_else(Instant::now);
    let elapsed = start_time.elapsed().as_secs_f64();
    let total_progress = file_info
        .iter()
        .filter(|fi| fi.status != FileStatus::Downloading)
        .map(|fi| fi.progress)
        .sum::<u64>();
    filenames.push(Line::raw("Total"));
    progress.push(Line::raw(format!("{total_progress:10} games")));
    speed.push(Line::raw(format!(
        "{:8.0} games/s",
        total_progress as f64 / elapsed
    )));
}

impl FileInfo {
    pub fn get_filename(&self) -> Line {
        let span = Span::styled(
            Cow::from(&self.filename),
            Style::default().fg(self.get_color()),
        );
        Line::from(vec![span])
    }

    pub fn get_progress(&self) -> Line {
        let text = if self.status == FileStatus::Downloading {
            format!("{:10.0} MB", self.progress as f64 / 1e6)
        } else {
            format!("{:10} games", self.progress)
        };

        let span = Span::styled(text, Style::default().fg(self.get_color()));
        Line::from(vec![span])
    }

    pub fn get_speed(&self) -> Line {
        let text = if self.status == FileStatus::Downloading {
            format!("{:8.0} MB/s", self.speed / 1e6)
        } else {
            format!("{:8.0} games/s", self.speed)
        };
        let span = Span::styled(text, Style::default().fg(self.get_color()));
        Line::from(vec![span])
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
