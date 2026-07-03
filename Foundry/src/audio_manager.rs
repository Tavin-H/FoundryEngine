// Audio playback
use cpal::StreamConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::storage::Heap;
use ringbuf::wrap::caching::Caching;
use symphonia::core::audio::GenericAudioBufferRef;

use ::std::sync::Arc;

// Files and decoding
use ringbuf::traits::*;
use ringbuf::{HeapRb, SharedRb};
use std::fs::File;
use std::thread;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, Track, TrackType};
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
        // ---CPAL SETUP---
        let host = cpal::default_host();
        let Some(output_device) = host.default_output_device() else {
            panic!("Could not find an output device");
        };
        AudioManager {
            host: host,
            output_device: output_device,
        }
    }

    pub fn decode(
        mut format: Box<dyn FormatReader + Send>,
        mut producer_r: impl Producer<Item = f32>,
        mut producer_l: impl Producer<Item = f32>,
    ) {
        // Main resource used for setup:
        // https://github.com/pdeljanov/Symphonia/blob/main/symphonia/examples/getting-started.rs
        // https://users.rust-lang.org/t/decode-a-audio-file-that-cpal-can-consume-properly/110792/2
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

        thread::spawn(|| {});
        loop {
            // Get the next packet from the media format.
            let packet = match format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => {
                    // Reached the end of the stream.
                    panic!("")
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
    }

    pub fn play(&mut self, path: &str) {
        // ---RINGBUFFER SETUP---
        let (mut producer_r, mut receiver_r) = HeapRb::<f32>::new(48000 * 2).split();
        let (mut producer_l, mut receiver_l) = HeapRb::<f32>::new(48000 * 2).split();

        // ---SYMPHONIA SETUP---
        let Ok(file) = File::open(path) else {
            panic!("Failed to find file");
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

        //Use seperate thread for decoding
        thread::spawn(|| {
            AudioManager::decode(format, producer_l, producer_r);
        });
        println!(
            "Using audio device: {}",
            self.output_device.description().unwrap()
        );
        let supported_configs = self.output_device.supported_output_configs();
        let Ok(default_config) = self.output_device.default_output_config() else {
            panic!("")
        };
        let mut config: StreamConfig = default_config.into();
        config.sample_rate = 44100;

        let stream = self.output_device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Split for 2 channels
                for sample in data.chunks_mut(2) {
                    let mini_packet_r = match receiver_r.try_pop() {
                        Some(single) => single,
                        None => 0.0,
                    };
                    let mini_packet_l = match receiver_l.try_pop() {
                        Some(single) => single,
                        None => 0.0,
                    };
                    sample[0] = mini_packet_l;
                    sample[1] = mini_packet_r;
                }
            },
            move |err| {},
            None, // None=blocking, Some(Duration)=timeout
        );

        //Use seperate thread for playing
        thread::spawn(|| {
            let Ok(output_stream) = stream else {
                panic!("");
            };
            output_stream.play().unwrap();

            std::thread::sleep(Duration::from_secs(3));
            output_stream.pause().unwrap();
            drop(output_stream);
        });
    }
    //Settings functions
    pub fn get_available_output_devices() {}
}
