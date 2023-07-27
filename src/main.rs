use std::{io, rc::Rc, cell::RefCell};

use settings::SettingsWindow;
use library::LibraryWindow;

mod ui;
mod audio; 
mod library;
mod settings;

fn main() -> Result<(), io::Error> {
    // terminal initialization
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let audio_interface = Rc::new(RefCell::new(audio::AudioInterface::new(sink)));
    let mut ui: ui::UI = ui::UI::new(audio_interface.clone())?;
    ui.push_window(
        Box::new(LibraryWindow::new(audio_interface.clone())),
    );
    ui.push_window(
        Box::new(SettingsWindow::new(audio_interface)),
    );
    ui.run()
}

