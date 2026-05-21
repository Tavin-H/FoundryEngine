// Audio playback
use cpal::StreamConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use symphonia::core::audio::GenericAudioBufferRef;

// Files and decoding
use ringbuf::HeapRb;
use ringbuf::traits::*;
use std::fs::File;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use symphonia::core::audio::Audio;

//Other essentials
use std::time::Duration;

pub struct AudioManager {
    host: cpal::Host,
    output_device: cpal::Device,
}

impl AudioManager {
    pub fn new() -> Self {
        // ---SYMPHONIA SETUP---
        let path = "Sounds/fah.mp3";
        let Ok(file) = File::open(path) else {
            panic!("Failed to find file");
        };

        //Move to the AudioManager?
        let (mut producer_r, mut receiver_r) = HeapRb::<f32>::new(48000 * 2).split();
        let (mut producer_l, mut receiver_l) = HeapRb::<f32>::new(48000 * 2).split();

        let mut audio_decoded_left: Vec<i32> = Vec::new();
        let mut audio_decoded_right: Vec<i32> = Vec::new();

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
            panic!("No audio track");
        };

        println!(
            "{}",
            track
                .clone()
                .codec_params
                .unwrap()
                .audio()
                .unwrap()
                .sample_rate
                .unwrap()
        );

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
                Ok(decoded) => {
                    // Consume the decoded audio samples (see below).
                    //AudioManager::handle_decoded_sample(decoded, producer_r);
                    match decoded {
                        GenericAudioBufferRef::F32(buffer) => {
                            let Some(slice_l) = buffer.plane(0) else {
                                panic!("");
                            };
                            let Some(slice_r) = buffer.plane(1) else {
                                panic!("");
                            };
                            for left in slice_l {
                                producer_l.try_push(*left);
                            }
                            for right in slice_r {
                                producer_r.try_push(*right);
                            }
                        }
                        _ => panic!("Unsuproted audio type"),
                    }
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

        // ---CPAL SETUP---
        let host = cpal::default_host();
        let Some(output_device) = host.default_output_device() else {
            panic!("Could not find an output device");
        };
        println!(
            "Using audio device: {}",
            output_device.description().unwrap()
        );
        let supported_configs = output_device.supported_output_configs();
        let Ok(default_config) = output_device.default_output_config() else {
            panic!("")
        };
        let mut config: StreamConfig = default_config.into();
        config.sample_rate = 44100 / 2;

        let stream = output_device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // react to stream events and read or write stream data here.
                for sample in data {
                    let mini_packet = match receiver_r.try_pop() {
                        Some(single) => single,
                        None => 0.0,
                    };
                    *sample = mini_packet;
                }
            },
            move |err| {
                // react to errors here.
            },
            None, // None=blocking, Some(Duration)=timeout
        );

        let Ok(output_stream) = stream else {
            panic!("");
        };
        output_stream.play().unwrap();

        std::thread::sleep(Duration::from_secs(3));
        output_stream.pause().unwrap();
        drop(output_stream);
        AudioManager {
            host: host,
            output_device: output_device,
        }
    }

    /*
    pub fn decode(&mut self, path: &str) -> Result<(Vec<f32>, Vec<f32>), &str> {
        // Main resource used for setup:
        // https://github.com/pdeljanov/Symphonia/blob/main/symphonia/examples/getting-started.rs
        // https://users.rust-lang.org/t/decode-a-audio-file-that-cpal-can-consume-properly/110792/2
    }

    pub fn handle_decoded_sample(
        sample_buffer: GenericAudioBufferRef,
        producer_r: &mut i32,
        producer_l: &mut i32,
    ) -> (f64, f64) {
    }
    */
    pub fn play(&mut self, path: &str) {}
    //Settings functions
    pub fn get_available_output_devices() {}
}
