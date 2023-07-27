use std::{io::{self, Stdout}, env, path::{PathBuf, Path}, rc::Rc, cell::RefCell};
use crossterm::event::KeyCode;
use tui::{
    layout::{Rect, Constraint}, 
    Frame, 
    backend::CrosstermBackend, 
    widgets::{Block, Borders, Row, Table, TableState}, 
    style::{Style, Color, Modifier},
};
use crate::{ui::Window, audio::{AudioInterface, AudioFile}};


pub struct LibraryWindow {
    title: String,
    audio_interface: Rc<RefCell<AudioInterface>>,
    music_list: Vec<AudioFile>,
    state: TableState,
}


impl LibraryWindow {
    pub fn new(audio_interface: Rc<RefCell<AudioInterface>>) -> Self {
        // TODO: Remove the env::home_dir() call and replace it with a config file
        let music_list = recursive_file_walk(&env::home_dir().unwrap().join("Music"))
            .into_iter()
            .map(|path| path.to_str().unwrap().to_string())
            .collect::<Vec<_>>();
        let music_list = music_list
            .iter()
            .filter_map(|path| {
                if let Ok(audiofile) = AudioFile::new(path) {
                    Some(audiofile)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let mut state = TableState::default();
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

    fn get_wrapped_music_list(&self, i: usize) -> Vec<AudioFile> {
        let get_music_list = self.music_list.clone();
        // split the list in two at the index i given:
        let first_half = get_music_list.get(0..i).unwrap_or(&[]);
        let second_half = get_music_list.get(i..).unwrap_or(&[]);
        // combine the two halves into a new list
        let mut wrapped_music_list = Vec::new();
        wrapped_music_list.extend_from_slice(second_half);
        wrapped_music_list.extend_from_slice(first_half);
        wrapped_music_list
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
        let mut table_widget_vec = Vec::new();
        for file in self.music_list.iter() {
            table_widget_vec.push(
                Row::new(vec![
                    file.get_title().clone(),
                    file.get_artist().clone(),
                    file.get_album().clone(),
                    file.get_year().to_string(),
                    file.get_duration(),
                ])
            )
        }
        let chunks = tui::layout::Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints(
                [
                    tui::layout::Constraint::Percentage(95),
                    tui::layout::Constraint::Percentage(5),
                ].as_ref()
            )
            .split(area);
        let table_widget = Table::new(table_widget_vec)
            .block(Block::default().title("Music Found").borders(Borders::ALL))
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Green).fg(Color::White))
            .header(
                Row::new(vec!["Title", "Artist", "Album", "Year", "Duration"])
                    .style(Style::default().fg(Color::Yellow))
            )
            .widths(&[Constraint::Length(50), Constraint::Length(50), Constraint::Length(50), Constraint::Length(10), Constraint::Length(10)]);
        let progress_bar = tui::widgets::Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Green).bg(Color::Black))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
            .label(match self.audio_interface.borrow().get_currently_playing() {
                Some(audiofile) => {
                    let title = audiofile.get_title().clone();
                    let ratio = self.audio_interface.borrow().get_sink_length() as f64 / audiofile.get_raw_duration();
                    format!("{} - {:.2}%", title, ratio * 100.0)
                },
                None => "Nothing Playing".to_string(),
            })
            .ratio(match self.audio_interface.borrow().get_currently_playing() {
                Some(audiofile) => self.audio_interface.borrow().get_sink_length() as f64 / audiofile.get_raw_duration(), 
                None => 0.0,
            });
        f.render_stateful_widget(table_widget, chunks[0], &mut self.state);
        f.render_widget(progress_bar, chunks[1]);
        Ok(())
    }

    fn handle_input(&mut self, key: crossterm::event::KeyCode) -> Result<(), io::Error> {
        match key {
            KeyCode::Up => {self.previous()},
            KeyCode::Down => {self.next()},
            KeyCode::Enter => {
                if let Some(i) = self.state.selected() {
                    self.audio_interface.borrow_mut().hard_clear_queue();
                    let mut wrapped_music_list = self.get_wrapped_music_list(i);
                    self.audio_interface.borrow_mut().append_to_queue(&mut wrapped_music_list);
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
