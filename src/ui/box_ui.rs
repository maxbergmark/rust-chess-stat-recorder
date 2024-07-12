use std::{
    collections::HashMap,
    io::{stdout, Stdout},
    rc::Rc,
    thread,
    time::Instant,
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
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::{
    ui::util::{get_elapsed_time, to_human},
    util::{FileInfo, Progress},
    Error, Result,
};

use super::{
    file_progress::{FileProgress, FileStatus},
    util::{create_lines, to_paragraph},
    UserInterface,
};

#[derive(Debug)]
pub struct BoxUI {
    file_info: HashMap<String, FileProgress>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    last_update: Instant,
}

impl BoxUI {
    pub fn new() -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Self::setup()?;
        Ok(Self {
            file_info: HashMap::new(),
            terminal,
            last_update: Instant::now(),
        })
    }

    fn setup() -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(Clear(ClearType::All))?;
        stdout().execute(EnterAlternateScreen)?;
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
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

    fn draw(file_info_map: &HashMap<String, FileProgress>, frame: &mut Frame) {
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
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(8),
                Constraint::Percentage(27),
            ],
        )
        .split(main_layout[1]);

        Self::draw_sections(file_info_map, frame, &inner_layout);
    }

    fn draw_sections(
        file_info_map: &HashMap<String, FileProgress>,
        frame: &mut Frame,
        inner_layout: &Rc<[ratatui::layout::Rect]>,
    ) {
        let files = file_info_map
            .values()
            .sorted_by_key(|&fi| &fi.file_info.filename)
            .collect_vec();

        render_paragraph(frame, inner_layout[0], &files, create_filenames);
        render_paragraph(frame, inner_layout[1], &files, create_progress);
        render_paragraph(frame, inner_layout[2], &files, create_speed);
        render_paragraph(frame, inner_layout[3], &files, create_move_variations);
        render_paragraph(frame, inner_layout[4], &files, create_move_speed);
        render_paragraph(frame, inner_layout[5], &files, create_status);
        render_paragraph(frame, inner_layout[6], &files, create_messages);
    }
}

fn render_paragraph<'a>(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    files: &'a [&FileProgress],
    f: impl Fn(&'a [&FileProgress]) -> Paragraph<'a>,
) {
    let paragraph = f(files);
    frame.render_widget(paragraph, area);
}

fn create_filenames<'a>(files: &'a [&FileProgress]) -> Paragraph<'a> {
    create_paragraph(
        files,
        "Files",
        |fp| fp.file_info.filename.clone(),
        |_| Line::raw("Total"),
    )
}

fn create_progress<'a>(files: &'a [&FileProgress]) -> Paragraph<'a> {
    fn f(fp: &FileProgress) -> String {
        match fp.status {
            FileStatus::Waiting | FileStatus::Downloading => {
                let p = 1e-6 * fp.progress.bytes as f64;
                format!("{p:8.0} MB")
            }
            _ => {
                let p = 100.0 * fp.progress.games as f64 / fp.file_info.num_games as f64;
                format!("{p:8.2}%")
            }
        }
    }

    fn reducer<'a>(file_info: &'a [&FileProgress]) -> Line<'a> {
        let p: u64 = file_info.iter().map(|fp| fp.progress.games).sum();
        let s = format!("{:>10} games", to_human(p as f64));
        Line::raw(s)
    }

    create_paragraph(files, "Progress", f, reducer)
}

fn create_speed<'a>(files: &'a [&FileProgress]) -> Paragraph<'a> {
    fn f(fp: &FileProgress) -> String {
        match fp.status {
            FileStatus::Waiting | FileStatus::Downloading => {
                format!("{:8.0} MB/s", 1e-6 * fp.download_speed())
            }
            _ => format!("{:8.0} games/s", fp.speed()),
        }
    }

    fn reducer<'a>(files: &'a [&FileProgress]) -> Line<'a> {
        let p: f64 = files.iter().map(|fp| fp.progress.games as f64).sum();
        let t = get_elapsed_time(files);
        let s = format!("{:>8} games/s", to_human(p / t));
        Line::raw(s)
    }

    create_paragraph(files, "Speed", f, reducer)
}

fn create_move_variations<'a>(files: &'a [&FileProgress]) -> Paragraph<'a> {
    fn reducer<'a>(file_info: &'a [&FileProgress]) -> Line<'a> {
        let p: u64 = file_info.iter().map(|fp| fp.progress.move_variations).sum();
        let s = format!("{:>10} moves", to_human(p as f64));
        Line::raw(s)
    }

    create_paragraph(
        files,
        "Move Variations",
        |fp| format!("{:>10} moves", to_human(fp.progress.move_variations as f64)),
        reducer,
    )
}

fn create_move_speed<'a>(files: &'a [&FileProgress]) -> Paragraph<'a> {
    fn reducer<'a>(file_info: &'a [&FileProgress]) -> Line<'a> {
        let p: f64 = file_info
            .iter()
            .map(|fp| fp.progress.move_variations as f64)
            .sum();
        let t = get_elapsed_time(file_info);
        let s = format!("{:>8} moves/s", to_human(p / t));
        Line::raw(s)
    }

    create_paragraph(
        files,
        "Move speed",
        |fp| format!("{:>8} moves/s", to_human(fp.move_speed())),
        reducer,
    )
}

fn create_status<'a>(files: &'a [&FileProgress]) -> Paragraph<'a> {
    to_paragraph(create_lines(files, FileProgress::get_status), "Status")
}

fn create_messages<'a>(files: &'a [&FileProgress]) -> Paragraph<'a> {
    to_paragraph(create_lines(files, FileProgress::get_message), "Messages")
}

fn create_paragraph<'a>(
    files: &'a [&FileProgress],
    title: &'a str,
    f: impl Fn(&FileProgress) -> String,
    reducer: impl Fn(&'a [&FileProgress]) -> Line<'a>,
) -> Paragraph<'a> {
    let lines = files
        .iter()
        .filter(|fi| fi.status != FileStatus::Hidden)
        .map(|&fp| fp.to_line(&f(fp)))
        .chain(std::iter::once(reducer(files)))
        .collect_vec();
    to_paragraph(lines, title)
}

impl UserInterface for BoxUI {
    fn add_file(&mut self, file_info: &FileInfo) {
        let filename = file_info.filename.clone();
        self.file_info.insert(
            filename,
            FileProgress {
                file_info: file_info.clone(),
                progress: Progress::default(),
                start_time: Instant::now(),
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

    fn hide_file(&mut self, filename: &str) -> Result<()> {
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.status = FileStatus::Hidden;
        }
        self.update()
    }

    fn update_progress(&mut self, filename: &str, progress: Progress) -> Result<()> {
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.progress = progress;
        }
        self.update()
    }

    fn complete_file(&mut self, filename: &str, progress: Progress) -> Result<()> {
        self.update_progress(filename, progress)?;
        if let Some(file_info) = self.file_info.get_mut(filename) {
            file_info.status = FileStatus::Done;
        }
        self.update()
    }

    fn wait_for_exit(&mut self) -> Result<()> {
        self.update()?;
        while !handle_events()? {
            thread::sleep(std::time::Duration::from_millis(500));
        }
        self.exit()
    }

    fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        self.terminal.clear()?;
        std::process::exit(0);
    }
}

fn handle_events() -> std::io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
