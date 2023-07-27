use std::io::{BufReader, Error, ErrorKind};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use audiotags::Tag;
use rodio::DeviceTrait;
use rodio::cpal::traits::HostTrait;
use rodio::cpal;

#[derive(Clone)]
pub struct AudioFile {
    path: PathBuf,
    title: String,
    artist: String,
}

impl AudioFile {
    pub fn new(path: &String) -> Self {
        if let Ok(tag) = Tag::new().read_from_path(path) {
            Self {
                path: PathBuf::from(path),
                title: tag.title().unwrap_or("Unknown").to_string(),
                artist: tag.artist().unwrap_or("Unknown").to_string(),
            }
        } else {
            Self {
                path: PathBuf::from(path),
                title: String::from("Unknown"),
                artist: String::from("Unknown"),
            }
        }
    }

    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }
}

pub struct Devices {
    devices: Vec<rodio::Device>,
    device_names: Vec<String>,
    current_device: usize,
}

impl Devices {
    fn new() -> Self {
        let device_list = match cpal::default_host().output_devices() {
            Ok(devices) => devices,
            Err(_) => panic!("No devices found"),
        };
        let mut devices = Vec::new();
        for device in device_list {
            if let Ok(_name) = device.name() {
                devices.push(device);
            }
        }
        let device_names = devices
        .iter()
        .map(|device| device.name().unwrap())
        .collect::<Vec<_>>();
        // get index of current device:
        let current_device = devices
            .iter()
            .position(|device| device.name().unwrap() == device.name().unwrap())
            .unwrap();
        Devices{
            devices,
            device_names,
            current_device,
        }
    }

    pub fn get_device_names(&self) -> Vec<String> {
        self.device_names.clone()
    }

    pub fn get_current_device(&self) -> usize {
        self.current_device
    }
}

pub struct AudioInterface {
    pub devices: Devices,
    queue: VecDeque<AudioFile>,
    currently_playing: Option<AudioFile>,
    sink: rodio::Sink,
}



impl AudioInterface {
    pub fn new(sink: rodio::Sink) -> Self {
        Self {
            devices: Devices::new(),
            sink,
            currently_playing: None,
            queue: VecDeque::new(),
        }
    }

    pub fn append_to_queue(&mut self, new_queue: &mut Vec<AudioFile>) {
        // Vec to VecDeque
        let mut new_queue = new_queue.drain(..).collect::<VecDeque<_>>();
        self.queue.append(&mut new_queue);
        if self.currently_playing.is_none() {
            self.play_next();
        }
    }

    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    pub fn hard_clear_queue(&mut self) {
        self.queue.clear();
        self.sink.stop();
        self.currently_playing = None;
    }

    pub fn handle_queue(&mut self) {
        if self.sink.empty() && self.currently_playing.is_none() {
            self.currently_playing = self.get_next().map(|s| s.clone());
            self.play_next();
        } else if self.sink.empty() && self.currently_playing.is_some() {
            self.currently_playing = None;
        }
    }

    pub fn get_next(&self) -> Option<&AudioFile> {
        if let Some(next) = self.queue.front() {
            Some(next)
        } else {
            None
        }
    }

    fn play_next(&mut self) {
        if let Some(next) = self.queue.pop_front() {
            self.currently_playing = Some(next);
            self.play(self.currently_playing.as_ref().unwrap().get_path()).unwrap();
        }
    }

    pub fn play(&self, file: &Path) -> Result<(), std::io::Error>{
        self.sink.stop();
        let extension = file
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        let file = BufReader::new(std::fs::File::open(file)?);
        // match on the extension of the file:
        match extension {
            "mp3" => {
                if let Ok(source) = rodio::Decoder::new_mp3(file) {
                    self.sink.append(source);
                } else {
                    return Err(Error::new(ErrorKind::Other, "Failed to play file"))
                };
            },
            "wav" => {
                if let Ok(source) = rodio::Decoder::new_wav(file) {
                    self.sink.append(source);
                } else {
                    return Err(Error::new(ErrorKind::Other, "Failed to play file"))
                };
            },
            "flac" => {
                if let Ok(source) = rodio::Decoder::new_flac(file) {
                    self.sink.append(source);
                } else {
                    return Err(Error::new(ErrorKind::Other, "Failed to play file"))
                };
            },
            "ogg" => {
                if let Ok(source) = rodio::Decoder::new(file) {
                    self.sink.append(source);
                } else {
                    return Err(Error::new(ErrorKind::Other, "Failed to play file"))
                };
            },
            _ => {
                return Err(Error::new(ErrorKind::Other, "Failed to play file"))
            }
        }
        Ok(())
    }
}