// Audio playback
use cpal::StreamConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// Files and decoding
//use crate::BufReader;
use std::fs::File;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

pub struct AudioManager {
    host: cpal::Host,
    output_device: cpal::Device,
}

impl AudioManager {
    pub fn new() -> Self {
        // ---SYMPHONIA SETUP---

        // ---CPAL SETUP---
        let host = cpal::default_host();
        let Some(output_device) = host.default_output_device() else {
            panic!("Could not find an output device");
        };
        let supported_configs = output_device.supported_output_configs();
        let Ok(default_config) = output_device.default_output_config() else {
            panic!("")
        };
        let config: StreamConfig = default_config.into();

        let stream = output_device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // react to stream events and read or write stream data here.
            },
            move |err| {
                // react to errors here.
            },
            None, // None=blocking, Some(Duration)=timeout
        );
        AudioManager {
            host: host,
            output_device: output_device,
        }
    }
    pub fn decode(&mut self, path: &str) -> Result<i32, &str> {
        // Main resource used for setup:
        // https://github.com/pdeljanov/Symphonia/blob/main/symphonia/examples/getting-started.rs
        let Ok(file) = File::open(path) else {
            return Err("Failed to find file");
        };

        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        hint.with_extension("mp3");

        //Use defaults (idk what these are)
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let mut format = symphonia::default::get_probe()
            .probe(&hint, mss, fmt_opts, meta_opts)
            .expect("unsupported format");

        let Some(track) = format.default_track(TrackType::Audio) else {
            return Err("No audio track");
        };

        let dec_opts: AudioDecoderOptions = Default::default();

        let mut decoder = symphonia::default::get_codecs()
            .make_audio_decoder(
                track
                    .codec_params
                    .as_ref()
                    .expect("codec parameters missing")
                    .audio()
                    .unwrap(),
                &dec_opts,
            )
            .expect("unsupported codec");

        let track_id = track.id;

        loop {
            // Get the next packet from the media format.
            let packet = match format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => {
                    // Reached the end of the stream.
                    break;
                }
                Err(Error::ResetRequired) => {
                    //This is a complicated experimental thing so I'll just ignore it with a panic
                    panic!();
                }
                Err(err) => {
                    panic!("{}", err);
                }
            };

            while !format.metadata().is_latest() {
                format.metadata().pop();
            }

            if packet.track_id != track_id {
                continue;
            }

            // Decode the packet into audio samples.
            match decoder.decode(&packet) {
                Ok(_decoded) => {
                    // Consume the decoded audio samples (see below).
                }
                Err(Error::IoError(_)) => {
                    // The packet failed to decode due to an IO error, skip the packet.
                    continue;
                }
                Err(Error::DecodeError(_)) => {
                    // The packet failed to decode due to invalid data, skip the packet.
                    continue;
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }

        Ok(42)
    }
    pub fn play(&mut self, path: &str) {}
    //Settings functions
    pub fn get_available_output_devices() {}
}
