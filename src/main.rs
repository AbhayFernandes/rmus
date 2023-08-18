use std::{cell::RefCell, io, rc::Rc};

use library::LibraryWindow;
use player::PlayerWindow;
use settings::SettingsWindow;
use tidal::TidalWindow;

mod audio;
mod library;
mod player;
mod settings;
mod tidal;
mod ui;

fn main() -> Result<(), io::Error> {
    // terminal initialization
    let settings = Rc::new(RefCell::new(settings::Settings::load()));
    let device = settings.borrow().get_device();
    let devices = audio::Devices::new(device);
    let device = devices.get_device_by_index(device);
    println!("{}", devices.get_device_names().len());
    let (stream, stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    let audio_interface = Rc::new(RefCell::new(audio::AudioInterface::new(
        stream,
        rodio::Sink::try_new(&stream_handle).unwrap(),
        devices,
    )));
    let tidal_session = Rc::new(RefCell::new(tidal::TidalSession::new()));
    let mut ui: ui::UI = ui::UI::new(
        settings.clone(),
        audio_interface.clone(),
        tidal_session.clone(),
    )?;
    ui.push_window(Box::new(LibraryWindow::new(
        settings.clone(),
        audio_interface.clone(),
    )));
    ui.push_window(Box::new(PlayerWindow::new(
        audio_interface.clone(),
        settings.clone(),
    )));
    ui.push_window(Box::new(TidalWindow::new(tidal_session.clone())));
    ui.push_window(Box::new(SettingsWindow::new(
        settings.clone(),
        audio_interface,
    )));
    ui.run()
}
