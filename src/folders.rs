use std::{
    cell::RefCell,
    io::{self, Stdout},
    path::Path,
    rc::Rc,
};

use crossterm::event::KeyCode;
use tui::{
    prelude::{CrosstermBackend, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::{settings::Settings, ui::Window};

enum ExplorerState {
    Explore(String),
    None,
}

pub struct FoldersWindow {
    title: String,
    state: ListState,
    explorer_window: FileExplorerWindow,
    estate: ExplorerState,
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
        match &self.estate {
            ExplorerState::None => {
                let ref_settings = self.settings.borrow();
                let mut lib_folders = ref_settings
                    .lib_folders
                    .iter()
                    .map(|folder| ListItem::new(folder.as_str()))
                    .collect::<Vec<_>>();
                lib_folders
                    .push(ListItem::new("Add a Folder").style(Style::default().fg(Color::Yellow)));
                let folder_list_widget = List::new(lib_folders)
                    .block(Block::default().title("Folders").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Green))
                    .highlight_style(Style::default().bg(Color::Green).fg(Color::White))
                    .highlight_symbol(">> ");
                f.render_stateful_widget(folder_list_widget, area, &mut self.state);
                Ok(())
            }
            ExplorerState::Explore(s) => {
                self.explorer_window.set_cwd(s);
                self.explorer_window.draw(area, f)
            }
        }
    }

    fn handle_input(&mut self, key: KeyCode) -> std::result::Result<(), io::Error> {
        match &self.estate {
            ExplorerState::None => {
                match key {
                    KeyCode::Up => self.previous(),
                    KeyCode::Down => self.next(),
                    KeyCode::Enter => self.file_explorer(if let Some(i) = self.state.selected() {
                        self.settings.borrow().lib_folders[i].clone()
                    } else {
                        format!("{}", home::home_dir().unwrap().display())
                    }),
                    _ => {}
                }
                Ok(())
            }
            ExplorerState::Explore(s) => Ok(()),
        }
    }
}

impl FoldersWindow {
    pub fn new(settings: Rc<RefCell<Settings>>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        let file_explorer_window: FileExplorerWindow = FileExplorerWindow::new();
        Self {
            settings: settings.clone(),
            title: "Folders".to_string(),
            state,
            estate: ExplorerState::None,
            explorer_window: file_explorer_window,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.settings.borrow().lib_folders.len() {
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

    fn file_explorer(&self, path: String) {}
}

pub struct FileExplorerWindow {
    title: String,
    path: String,
}

impl Window for FileExplorerWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(
        &mut self,
        area: Rect,
        f: &mut Frame<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error> {
        // let new_path = Path::new(&self.path);
        // let mut Files = new_path.read_dir().unwrap();
        // let mut file_vec = Vec::new();
        // Files.map(|x| {
        //     // let y = x.unwrap().path().file_name().unwrap().to_str().unwrap();
        //     ListItem::new(y.clone());
        // });
        // let mut FileWindow = List::new(file_vec);
        // f.render_widget(FileWindow, area);
        Ok(())
    }
}

impl FileExplorerWindow {
    fn new() -> Self {
        let path = format!("{}", home::home_dir().unwrap().display());
        Self {
            title: path.clone(),
            path: path.clone(),
        }
    }

    fn set_cwd(&mut self, s: &str) {
        self.path = s.to_string();
    }
}
