use std::{io::{self, Stdout, BufReader}, env, path::{PathBuf, Path}, fs::File, rc::Rc, cell::RefCell};
use crossterm::event::KeyCode;
use rodio::{Decoder, Sink};
use tui::{
    layout::Rect, 
    Frame, 
    backend::CrosstermBackend, 
    widgets::{Block, Borders, List, ListItem, ListState}, 
    style::{Style, Color},
};
use crate::{ui::{Window, UI}, audio::AudioInterface};

pub struct LibraryWindow {
    title: String,
    audio_interface: Rc<RefCell<AudioInterface>>,
    music_list: Vec<String>,
    state: ListState,
}


impl LibraryWindow {
    pub fn new(audio_interface: Rc<RefCell<AudioInterface>>) -> Self {
        // TODO: Remove the env::home_dir() call and replace it with a config file
        let music_list = recursive_file_walk(&env::home_dir().unwrap().join("Music"))
            .into_iter()
            .map(|path| path.to_str().unwrap().to_string())
            .collect::<Vec<_>>();
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            title: String::from("Library"),
            music_list,
            state,
            audio_interface
        }
    }


    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.music_list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn get_state(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }


    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.music_list.len() - 1
                } else {
                    i - 1
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl Window for LibraryWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(&mut self, area: Rect, f: &mut Frame<CrosstermBackend<Stdout>>) -> Result<(), io::Error> {
        let list_widget = List::new(self.music_list
            .iter()
            .map(|file| ListItem::new(file.as_str()))
            .collect::<Vec<_>>())
            .block(Block::default().title("Music Found").borders(Borders::ALL))
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().bg(Color::Green).fg(Color::White))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list_widget, area, &mut self.state);
        Ok(())
    }

    fn handle_input(&mut self, key: crossterm::event::KeyCode) -> Result<(), io::Error> {
        match key {
            KeyCode::Up => {self.previous()},
            KeyCode::Down => {self.next()},
            KeyCode::Enter => {
                if let Some(i) = self.state.selected() {
                    self.audio_interface.borrow().play(&self.music_list[i]);
                }
            }
            _ => {},
        }
        Ok(())
    }
}

fn recursive_file_walk(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in path.read_dir().expect("read_dir call failed") {
        let entry = entry.expect("Error reading entry");
        let path = entry.path();
        if path.is_dir() {
            files.append(&mut recursive_file_walk(&path));
        } else {
            // Check if file is an mp3, flac, wav, or ogg and add it to the list
            if let Some(ext) = path.extension() {
                if ext == "mp3" || ext == "flac" || ext == "wav" || ext == "ogg" {
                    files.push(path);
                }
            } 
        }
    }
    files
}
