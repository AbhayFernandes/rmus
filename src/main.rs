use std::{cell::RefCell, io, rc::Rc};

use library::LibraryWindow;
use settings::SettingsWindow;
mod audio;
mod library;
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
    let (_stream, stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    // let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let audio_interface = Rc::new(RefCell::new(audio::AudioInterface::new(
        rodio::Sink::try_new(&stream_handle).unwrap(),
        devices,
    )));
    let text_input_handler = Rc::new(RefCell::new(ui::TextInputHandler::new()));
    let mut ui: ui::UI = ui::UI::new(settings.clone(), audio_interface.clone(), text_input_handler.clone())?;
    ui.push_window(Box::new(LibraryWindow::new(settings.clone(), audio_interface.clone())));
    ui.push_window(Box::new(SettingsWindow::new(settings.clone(), audio_interface, text_input_handler.clone())));
    ui.run()
}
