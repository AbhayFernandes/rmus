use std::collections::VecDeque;
use std::io::Error;
use std::io::{BufReader, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::Instant;

use audiotags::Tag;
use rodio::cpal;
use rodio::cpal::traits::HostTrait;
use rodio::DeviceTrait;

#[derive(Clone)]
pub struct AudioFile {
    path: PathBuf,
    title: String,
    artist: String,
    year: i32,
    album: String,
    duration: f64,
}

const EMPTY_ALBUM: audiotags::types::Album = audiotags::types::Album {
    title: "Unknown",
    artist: None,
    cover: None,
};

impl AudioFile {
    pub fn new(path: &String) -> Result<Self, std::io::Error> {
        if let Ok(tag) = Tag::new().read_from_path(path) {
            // get duration, scaffolding for when an implementation 
            // for finding the bitrate and estimating the duration
            let duration: f64 = match tag.duration() {
                Some(duration) => duration,
                None => 0.0, 
            };
            Ok(Self {
                path: PathBuf::from(path),
                title: tag.title().unwrap_or("Unknown").to_string(),
                year: tag.year().unwrap_or(0),
                artist: tag.artist().unwrap_or("Unknown").to_string(),
                album: tag.album().unwrap_or(EMPTY_ALBUM).title.to_string(),
                duration,
            })
        } else {
            Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Failed to read file",
            ))
        }
    }

    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_album(&self) -> &String {
        &self.album
    }

    pub fn get_artist(&self) -> &String {
        &self.artist
    }

    pub fn get_raw_duration(&self) -> f64 {
        self.duration
    }

    pub fn get_duration(&self) -> String {
        let minutes = self.duration as i32 / 60;
        let seconds = self.duration as i32 % 60;
        format!("{}:{:02}", minutes, seconds)
    }

    pub fn get_year(&self) -> i32 {
        self.year
    }
}

pub struct Devices {
    devices: Vec<rodio::Device>,
    device_names: Vec<String>,
    current_device: usize,
}

impl Devices {
    pub fn new(curr_device: usize) -> Self {
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
            .map(|device| {
                if let Ok(name) = device.name() {
                    name
                } else {
                    String::from("Unknown")
                }
            })
            .collect::<Vec<_>>();
        // get index of current device:
        Devices {
            devices,
            device_names,
            current_device: curr_device,
        }
    }

    pub fn get_device_names(&self) -> Vec<String> {
        self.device_names.clone()
    }

    pub fn get_device_by_index(&self, index: usize) -> &rodio::Device {
        &self.devices[index]
    }

    pub fn get_current_device(&self) -> usize {
        self.current_device
    }
}

struct Track {
    start_time: Instant,
    pause_time: Option<Instant>,
    pause_duration: f64,
}

impl Track {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            pause_time: None,
            pause_duration: 0.0,
        }
    }

    fn toggle_pause(&mut self) {
        match self.pause_time {
            Some(time) => {
                self.pause_duration += time.elapsed().as_secs_f64();
                self.pause_time = None;
            }
            None => {
                self.pause_time = Some(Instant::now());
            }
        }
    }

    fn time(&self) -> f64 {
        match self.pause_time {
            None => self.start_time.elapsed().as_secs_f64() - self.pause_duration,
            Some(time) => {
                self.start_time.elapsed().as_secs_f64()
                    - time.elapsed().as_secs_f64()
                    - self.pause_duration
            }
        }
    }

    fn reset(&mut self) {
        self.start_time = Instant::now();
        self.pause_time = None;
        self.pause_duration = 0.0;
    }
}

pub struct AudioInterface {
    pub devices: Devices,
    queue: VecDeque<AudioFile>,
    // prevent the stream from being dropped
    stream: rodio::OutputStream,
    currently_playing: Option<AudioFile>,
    pause: bool,
    track: Track,
    sink: rodio::Sink,
}

impl AudioInterface {
    pub fn new(stream: rodio::OutputStream, sink: rodio::Sink, devices: Devices) -> Self {
        Self {
            devices,
            stream,
            sink,
            pause: false,
            track: Track::new(),
            currently_playing: None,
            queue: VecDeque::new(),
        }
    }

    pub fn get_paused(&self) -> bool {
        self.pause
    }

    pub fn get_currently_playing(&self) -> &Option<AudioFile> {
        &self.currently_playing
    }

    pub fn toggle_pause(&mut self) {
        self.track.toggle_pause();
        self.pause = !self.pause;
        if self.pause {
            self.sink.pause();
        } else {
            self.sink.play();
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

    pub fn hard_clear_queue(&mut self) {
        self.queue.clear();
        self.sink.stop();
        self.currently_playing = None;
    }

    pub fn handle_queue(&mut self) {
        if self.sink.empty() && self.currently_playing.is_none() {
            self.currently_playing = self.get_next().cloned();
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

    pub fn get_sink_length(&self) -> usize {
        if self.sink.empty() && self.currently_playing.is_none() {
            0
        } else {
            self.track.time() as usize
        }
    }

    fn play_next(&mut self) {
        if let Some(next) = self.queue.pop_front() {
            self.currently_playing = Some(next);
            self.track.reset();
            if self.pause {
                self.pause = false;
                self.sink.play();
            }
            self.play(self.currently_playing.as_ref().unwrap().get_path())
                .unwrap();
        }
    }

    fn play(&self, file: &Path) -> Result<(), std::io::Error> {
        self.sink.stop();
        let file = BufReader::new(std::fs::File::open(file)?);
        match rodio::Decoder::new(file) {
            Ok(source) => {
                self.sink.append(source);
                Ok(())
            }
            Err(e) => {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    e,
                )) 
            }
        }
    }
}
