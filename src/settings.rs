use crossterm::event::KeyCode;
use rodio::cpal::{self, traits::HostTrait};
use std::cell::RefCell;
use std::io::{self, Stdout};
use std::rc::Rc;
use tui::layout::{Constraint, Direction, Layout};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::audio::AudioInterface;
use crate::ui::Window;
pub struct Settings {
    lib_folders: Vec<String>,
    audio_device: rodio::Device,
}

impl Settings {
    pub fn new() -> Self {
        let lib_folders = Vec::new();
        let audio_device = cpal::default_host().default_output_device().unwrap();
        Self {
            lib_folders,
            audio_device,
        }
    }
}

struct DeviceWindow {
    title: String,
    audio_interface: Rc<RefCell<AudioInterface>>,
    state: ListState,
}

impl DeviceWindow {
    fn new(audio_interface: Rc<RefCell<AudioInterface>>) -> Self {
        Self {
            title: String::from("Device List"),
            audio_interface,
            state: ListState::default(),
        }
    }
}

impl Window for DeviceWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(
        &mut self,
        area: Rect,
        f: &mut Frame<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error> {
        let devices_list = self.audio_interface.borrow().devices.get_device_names();
        let devices_window = List::new(
            devices_list
                .iter()
                .map(|device| ListItem::new(device.as_str()))
                .collect::<Vec<_>>(),
        )
        .block(
            Block::default()
                .title(self.get_title())
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default().bg(Color::Green).fg(Color::White))
        .highlight_symbol(">> ");
        let mut device_state = ListState::default();
        device_state.select(Some(
            self.audio_interface.borrow().devices.get_current_device(),
        ));
        f.render_stateful_widget(devices_window, area, &mut device_state);
        Ok(())
    }

    fn handle_input(&mut self, _key: KeyCode) -> Result<(), io::Error> {
        Ok(())
    }
}

struct FoldersWindow {
    title: String,
    audio_interface: Rc<AudioInterface>,
    state: ListState,
    folders: Vec<String>,
}

impl Window for FoldersWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(
        &mut self,
        _area: Rect,
        _f: &mut Frame<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error> {
        Ok(())
    }

    fn handle_input(&mut self, _key: KeyCode) -> Result<(), io::Error> {
        Ok(())
    }
}

pub struct SettingsWindow {
    title: String,
    audio_interface: Rc<RefCell<AudioInterface>>,
    state: ListState,
    selected_window: usize,
    settings_windows: Vec<Box<dyn Window>>,
}

impl SettingsWindow {
    pub fn new(audio_interface: Rc<RefCell<AudioInterface>>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            title: String::from("Settings"),
            audio_interface: audio_interface.clone(),
            state,
            selected_window: 0,
            settings_windows: vec![Box::new(DeviceWindow::new(audio_interface.clone()))],
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.settings_windows.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.settings_windows.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn get_state(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }
}

impl Window for SettingsWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(
        &mut self,
        area: Rect,
        f: &mut Frame<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(area);
        let list_widget = List::new(
            self.settings_windows
                .iter()
                .map(|file| ListItem::new(file.get_title()))
                .collect::<Vec<_>>(),
        )
        .block(Block::default().title("Settings").borders(Borders::ALL))
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default().bg(Color::Green).fg(Color::White))
        .highlight_symbol(">> ");
        f.render_stateful_widget(list_widget, chunks[0], &mut self.state);
        let selected = self.get_state();
        self.settings_windows[selected].draw(chunks[1], f)?;
        Ok(())
    }

    fn handle_input(&mut self, key: crossterm::event::KeyCode) -> Result<(), std::io::Error> {
        match self.selected_window {
            0 => match key {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Right => {
                    self.selected_window = 1;
                }
                _ => {}
            },
            1 => {
                let num = self.get_state();
                self.settings_windows[num].handle_input(key)?;
            }
            _ => {
                self.selected_window = 0;
            }
        }
        Ok(())
    }
}
