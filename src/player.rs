use std::cell::RefCell;
use std::io::{self, Stdout};
use std::rc::Rc;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear},
    Frame,
};

use crate::audio::AudioInterface;
use crate::settings::Settings;
use crate::ui::Window;

const ALBUM_CENTER_WIDTH: u16 = 33;
const ALBUM_CENTER_HEIGHT: u16 = 50;

pub struct PlayerWindow {
    title: String,
    audio_interface: Rc<RefCell<AudioInterface>>,
    settings: Rc<RefCell<Settings>>,
}

impl PlayerWindow {
    pub fn new(
        audio_interface: Rc<RefCell<AudioInterface>>,
        settings: Rc<RefCell<Settings>>,
    ) -> Self {
        Self {
            audio_interface,
            settings,
            title: String::from("Player"),
        }
    }
}

impl Window for PlayerWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(
        &mut self,
        area: Rect,
        f: &mut Frame<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error> {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - ALBUM_CENTER_HEIGHT) / 2),
                    Constraint::Percentage(ALBUM_CENTER_HEIGHT),
                    Constraint::Percentage((100 - ALBUM_CENTER_HEIGHT) / 2),
                ]
                .as_ref(),
            )
            .split(area);

        let horizontal_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - ALBUM_CENTER_WIDTH) / 2),
                    Constraint::Percentage(ALBUM_CENTER_WIDTH),
                    Constraint::Percentage((100 - ALBUM_CENTER_WIDTH) / 2),
                ]
                .as_ref(),
            )
            .split(vertical_layout[1]);

        // populate each of the three sections with a block
        for layout in vertical_layout.iter() {
            let block = Block::default().borders(Borders::ALL);
            f.render_widget(block, *layout);
        }
        f.render_widget(Clear, horizontal_split[1]);
        let block_yellow = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(block_yellow, horizontal_split[1]);
        Ok(())
    }
}
