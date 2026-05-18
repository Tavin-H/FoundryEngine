use rodio::{Decoder, MixerDeviceSink, source::Source};
use std::fs::File;
use std::io::BufReader;

// Get an OS-Sink handle to the default physical sound device.
// Note that the playback stops when the handle is dropped.//
pub struct AudioManager {
    sink_handle: rodio::MixerDeviceSink,
    active_sounds: Vec<rodio::Player>,
}

impl AudioManager {
    pub fn new() -> Self {
        AudioManager {
            sink_handle: rodio::DeviceSinkBuilder::open_default_sink()
                .expect("open default audio stream"),
            active_sounds: Vec::new(),
        }
    }
    pub fn play(&mut self, path: &str) {
        let file = BufReader::new(File::open(path).unwrap());
        let player = rodio::play(&self.sink_handle.mixer(), file).unwrap();
        self.active_sounds.push(player);
    }
}
