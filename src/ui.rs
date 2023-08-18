use crossterm::{
    event::{poll, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
use std::{
    cell::RefCell,
    io::{self, Stdout},
    rc::Rc,
    time::Duration,
};
use tui::Terminal;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::settings::Settings;
use crate::{
    audio::{AudioFile, AudioInterface},
    tidal::TidalSession,
};

const TICK_RATE: Duration = Duration::from_millis(500);

pub trait Window {
    fn get_title(&self) -> String {
        String::from("Window")
    }

    fn draw(
        &mut self,
        area: Rect,
        f: &mut Frame<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error>;

    fn handle_input(&mut self, _key: KeyCode) -> Result<(), io::Error> {
        Ok(())
    }
}

pub struct UpNextWindow {
    title: String,
    audio_interface: Rc<RefCell<AudioInterface>>,
    next_up: Option<AudioFile>,
}

impl UpNextWindow {
    fn new(audio_interface: Rc<RefCell<AudioInterface>>) -> Self {
        Self {
            audio_interface,
            title: String::from("Up Next"),
            next_up: None,
        }
    }

    fn update_up_next(&mut self) {
        if let Some(next) = self.audio_interface.borrow().get_next() {
            self.next_up = Some(next.clone());
        } else {
            self.next_up = None;
        }
    }
}

impl Window for UpNextWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(
        &mut self,
        area: Rect,
        f: &mut Frame<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error> {
        self.update_up_next();
        let up_next = Paragraph::new(match &self.next_up {
            Some(audio_file) => format!(
                "{} by {}",
                audio_file.get_title().clone(),
                audio_file.get_artist().clone()
            ),
            None => String::from("Nothing"),
        })
        .block(Block::default().title("Next Up:").borders(Borders::ALL))
        .style(Style::default().fg(Color::Green));
        f.render_widget(up_next, area);
        Ok(())
    }

    fn handle_input(&mut self, _key: KeyCode) -> Result<(), io::Error> {
        Ok(())
    }
}

pub struct UI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    windows: Vec<Box<dyn Window>>,
    current_tab: usize,
    pub audio_interface: Rc<RefCell<AudioInterface>>,
    pub tidal_session: Rc<RefCell<TidalSession>>,
    pub settings: Rc<RefCell<Settings>>,
}

impl UI {
    fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.windows.len();
    }

    fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = self.windows.len() - 1;
        }
    }

    pub fn new(
        settings: Rc<RefCell<Settings>>,
        audio_interface: Rc<RefCell<AudioInterface>>,
        tidal_session: Rc<RefCell<TidalSession>>,
    ) -> Result<Self, io::Error> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self {
            terminal,
            windows: Vec::new(),
            current_tab: 0,
            tidal_session,
            audio_interface,
            settings,
        })
    }

    pub fn push_window(&mut self, window: Box<dyn Window>) {
        self.windows.push(window);
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        let mut up_next = UpNextWindow::new(self.audio_interface.clone());
        self.terminal.clear()?;
        loop {
            self.draw(&mut up_next)?;
            self.audio_interface.borrow_mut().handle_queue();
            if poll(TICK_RATE)? {
                if let Event::Key(key) = crossterm::event::read()? {
                    match key.code {
                        KeyCode::Char('q') => {
                            self.settings.borrow().save();
                            self.tidal_session.borrow().save();
                            break;
                        }
                        KeyCode::Char('h') => {
                            self.previous_tab();
                        }
                        KeyCode::Char('l') => {
                            self.next_tab();
                        }
                        KeyCode::Char('c') => {
                            self.audio_interface.borrow_mut().toggle_pause();
                        }
                        _ => {
                            self.windows[self.current_tab].handle_input(key.code)?;
                        }
                    }
                } else {
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, up_next: &mut UpNextWindow) -> Result<(), io::Error> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(self.terminal.size()?);
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
            .split(chunks[0]);
        let window_tabs = Tabs::new(
            self.windows
                .iter()
                .map(|w| Line::from(w.get_title()))
                .collect::<Vec<_>>(),
        )
        .block(Block::default().title("Rmus - Tabs").borders(Borders::ALL))
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(self.current_tab);
        let remaining_space = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(chunks[1]);
        self.terminal.draw(|f| {
            f.render_widget(window_tabs, top_chunks[0]);
            if let Err(e) = up_next.draw(top_chunks[1], f) {
                println!("Error drawing up next: {}", e);
            };
            if let Err(e) = self.windows[self.current_tab].draw(remaining_space[0], f) {
                eprintln!(
                    "Error drawing window ({}): {}",
                    self.windows[self.current_tab].get_title(),
                    e
                )
            };
        })?;
        Ok(())
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        println!("Dropping UI");
        disable_raw_mode().unwrap();
        execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )
        .unwrap();
        self.terminal.show_cursor().unwrap();
    }
}

pub fn centered_rect(x: u16, y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - y) / 2),
                Constraint::Percentage(y),
                Constraint::Percentage((100 - y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - x) / 2),
                Constraint::Percentage(x),
                Constraint::Percentage((100 - x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
