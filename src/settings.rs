use crate::audio::AudioInterface;
use crate::ui::Window;
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize)]
pub struct Settings {
    lib_folders: Vec<String>,
    device: usize,
}

impl Settings {
    pub fn load() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let settings_path = cwd.join("settings.json");
        let mut settings = Settings {
            lib_folders: Vec::new(),
            device: 0,
        };
        if settings_path.exists() {
            let settings_contents = std::fs::read_to_string(settings_path).unwrap();
            settings = serde_json::from_str(settings_contents.as_str()).unwrap();
        } else {
            let settings_contents = serde_json::to_string(&settings).unwrap();
            std::fs::write(settings_path, settings_contents).unwrap();
        };
        settings
    }

    pub fn get_device(&self) -> usize {
        self.device
    }

    pub fn save(&self) {
        let cwd = std::env::current_dir().unwrap();
        let settings_path = cwd.join("settings.json");
        let settings_contents = serde_json::to_string(&self).unwrap();
        std::fs::write(settings_path, settings_contents).unwrap();
    }
}

enum Popup {
    None,
    Message(String),
    Input(String),
}

struct DeviceWindow {
    title: String,
    audio_interface: Rc<RefCell<AudioInterface>>,
    settings: Rc<RefCell<Settings>>,
    popup: Popup,
    state: ListState,
}

impl DeviceWindow {
    fn new(audio_interface: Rc<RefCell<AudioInterface>>, settings: Rc<RefCell<Settings>>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        settings.borrow_mut().device = audio_interface.borrow().devices.get_current_device();
        Self {
            title: String::from("Device List"),
            settings,
            popup: Popup::None,
            audio_interface,
            state,
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
    ) -> std::result::Result<(), io::Error> {
        let devices = self.audio_interface.borrow().devices.get_device_names();
        let mut devices_vec = devices
            .iter()
            .map(|device| ListItem::new(device.as_str()))
            .collect::<Vec<_>>();
        let curr_device = self.audio_interface.borrow().devices.get_current_device();
        devices_vec[curr_device] =
            ListItem::new(devices[curr_device].as_str()).style(Style::default().fg(Color::Yellow));
        let devices_window = List::new(devices_vec)
            .block(
                Block::default()
                    .title(self.get_title())
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().bg(Color::Green).fg(Color::White))
            .highlight_symbol(">> ");
        // get the current device and highlight it a different color:
        f.render_stateful_widget(devices_window, area, &mut self.state);
        Ok(())
    }

    fn handle_input(&mut self, key: KeyCode) -> std::result::Result<(), io::Error> {
        match key {
            KeyCode::Up => self.next(),
            KeyCode::Down => self.previous(),
            KeyCode::Enter => {
                let selected = self.state.selected().unwrap();
                self.settings.borrow_mut().device = selected;
                self.popup = Popup::Message(String::from("Device changed - Restart to apply."));
            }
            _ => (),
        };
        Ok(())
    }
}

impl DeviceWindow {
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self
                    .audio_interface
                    .borrow()
                    .devices
                    .get_device_names()
                    .len()
                    - 1
                {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.audio_interface
                        .borrow()
                        .devices
                        .get_device_names()
                        .len()
                        - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

struct FoldersWindow {
    title: String,
    audio_interface: Rc<RefCell<AudioInterface>>,
    state: ListState,
    popup: Popup,
    settings: Rc<RefCell<Settings>>,
}

impl Window for FoldersWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(
        &mut self,
        area: Rect,
        f: &mut Frame<CrosstermBackend<Stdout>>,
    ) -> std::result::Result<(), io::Error> {
        let ref_settings = self.settings.borrow();
        let folder_list_widget = List::new(
            ref_settings
                .lib_folders
                .iter()
                .map(|folder| ListItem::new(folder.as_str()))
                .collect::<Vec<_>>(),
        )
        .block(Block::default().title("Folders").borders(Borders::ALL))
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default().bg(Color::Green).fg(Color::White))
        .highlight_symbol(">> ");
        f.render_stateful_widget(folder_list_widget, area, &mut self.state);
        Ok(())
    }

    fn handle_input(&mut self, key: KeyCode) -> std::result::Result<(), io::Error> {
        match key {
            KeyCode::Char('a') => {
                self.popup = Popup::Input(String::from("Enter a folder to add:"));
                self.settings
                    .borrow_mut()
                    .lib_folders
                    .push(String::from("test"));
            }
            KeyCode::Char('d') => {
                let selected = self.state.selected().unwrap();
                self.settings.borrow_mut().lib_folders.remove(selected);
            }
            KeyCode::Up => self.previous(),
            KeyCode::Down => self.next(),
            _ => {}
        };
        Ok(())
    }
}

impl FoldersWindow {
    pub fn new(
        audio_interface: Rc<RefCell<AudioInterface>>,
        settings: Rc<RefCell<Settings>>,
    ) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            audio_interface,
            settings: settings.clone(),
            popup: Popup::None,
            title: "Folders".to_string(),
            state,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.settings.borrow().lib_folders.len() - 1 {
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
                    self.settings.borrow().lib_folders.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub struct SettingsWindow {
    title: String,
    audio_interface: Rc<RefCell<AudioInterface>>,
    state: ListState,
    settings: Rc<RefCell<Settings>>,
    selected_window: usize,
    settings_windows: Vec<Box<dyn Window>>,
}

impl SettingsWindow {
    pub fn new(
        settings: Rc<RefCell<Settings>>,
        audio_interface: Rc<RefCell<AudioInterface>>,
    ) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            title: String::from("Settings"),
            audio_interface: audio_interface.clone(),
            state,
            selected_window: 0,
            settings: settings.clone(),
            settings_windows: vec![
                Box::new(DeviceWindow::new(audio_interface.clone(), settings.clone())),
                Box::new(FoldersWindow::new(
                    audio_interface.clone(),
                    settings.clone(),
                )),
            ],
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
    ) -> std::result::Result<(), io::Error> {
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

    fn handle_input(
        &mut self,
        key: crossterm::event::KeyCode,
    ) -> std::result::Result<(), std::io::Error> {
        match self.selected_window {
            0 => match key {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Right => {
                    self.selected_window = 1;
                }
                _ => {}
            },
            1 => match key {
                KeyCode::Left => self.selected_window = 0,
                _ => {
                    let num = self.get_state();
                    self.settings_windows[num].handle_input(key)?;
                }
            },
            _ => {
                self.selected_window = 0;
            }
        }
        Ok(())
    }
}
