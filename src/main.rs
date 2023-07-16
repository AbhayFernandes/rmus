use std::{io, rc::Rc};

mod ui; 
mod library;

fn main() -> Result<(), io::Error> {
    // terminal initialization
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let mut ui: ui::UI = ui::UI::new(Rc::new(sink))?;
    ui.run()?;
    Ok(())
}

