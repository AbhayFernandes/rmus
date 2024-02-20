use std::{cell::RefCell, io, rc::Rc};

use folders::FoldersWindow;
use library::LibraryWindow;
use settings::SettingsWindow;

mod audio;
mod folders;
mod library;
mod settings;
mod ui;

fn main() -> Result<(), io::Error> {
    // terminal initialization
    let settings = Rc::new(RefCell::new(settings::Settings::load()));
    let device = settings.borrow().get_device();
    let devices = audio::Devices::new(device);
    println!("{}", devices.get_device_names().len());
    let (stream, stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    let (stream, stream_handle) = (stream, stream_handle);
    let audio_interface = Rc::new(RefCell::new(audio::AudioInterface::new(
        stream,
        rodio::Sink::try_new(&stream_handle).unwrap(),
        devices,
    )));
    let mut ui: ui::UI = ui::UI::new(
        settings.clone(),
        audio_interface.clone(),
        tidal_session.clone(),
    )?;
    ui.push_window(Box::new(LibraryWindow::new(
        settings.clone(),
        audio_interface.clone(),
    )));
    ui.push_window(Box::new(FoldersWindow::new(settings.clone())));
    ui.push_window(Box::new(SettingsWindow::new(
        settings.clone(),
        audio_interface,
    )));
    ui.run()
}
