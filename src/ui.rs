use std::{io::{self, Stdout}, rc::Rc};
use tui::{
    backend::CrosstermBackend, 
    layout::{Constraint, Direction, Layout, Rect}, 
    widgets::{Block, Tabs, Borders, Paragraph},
    style::{Style, Color}, text::Spans, Frame};
use tui::Terminal;
use crossterm::{
    event::{EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode},
};

use crate::{audio::AudioInterface, library::LibraryWindow, settings::SettingsWindow};

pub trait Window {
    fn get_title(&self) -> String;
    fn draw(&mut self, area: Rect, f: &mut Frame<CrosstermBackend<Stdout>>) -> Result<(), io::Error>;
    fn handle_input(&mut self, key: KeyCode) -> Result<(), io::Error>;
}

pub struct UpNextWindow {
    title: String,
    next_up: String,
}

impl UpNextWindow {
    fn new() -> Self {
        Self {
            title: String::from("Up Next"),
            next_up: String::from("Nothing"),
        }
    }
}


impl Window for UpNextWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(&mut self, area: Rect, f: &mut Frame<CrosstermBackend<Stdout>>) -> Result<(), io::Error> {
        let up_next = Paragraph::new(self.next_up.as_str())
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
    pub audio_interface: Rc<AudioInterface>,
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

    pub fn new(audio_interface: Rc<AudioInterface>) -> Result<Self, io::Error> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(Self { 
            terminal,
            windows: Vec::new(), 
            current_tab: 0,
            audio_interface,
        })
    }
    

    pub fn push_window(&mut self, window: Box<dyn Window>) {
        self.windows.push(window);
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        self.draw()?;
        loop {
            // TODO: Make this work: self.audio_interface.as_mut().handle_queue();
            if let Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => {
                        self.previous_tab();
                    }
                    KeyCode::Char('l') => {
                        self.next_tab();
                    }
                    _ => {
                        self.windows[self.current_tab].handle_input(key.code)?;
                    }
                }
                self.draw()?;
            }
        }
        Ok(())
    }

    fn draw(&mut self) -> Result<(), io::Error> {
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
                .map(|w| Spans::from(w.get_title()))
                .collect::<Vec<_>>()
        )
            .block(Block::default().title("Rmus - Tabs").borders(Borders::ALL))
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().fg(Color::White))
            .select(self.current_tab);
        let mut up_next = UpNextWindow::new();
        let remaining_space = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(chunks[1]);
        self.terminal.draw(|f| {
            f.render_widget(window_tabs, top_chunks[0]);
            up_next.draw(top_chunks[1], f);
            self.windows[self.current_tab].draw(remaining_space[0], f); 
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
        ).unwrap();
        self.terminal.show_cursor().unwrap();
    }
}