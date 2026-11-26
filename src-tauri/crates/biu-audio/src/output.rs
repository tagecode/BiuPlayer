use std::f32::consts::TAU;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use serde::{Deserialize, Serialize};

use crate::AudioError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDeviceInfo {
    pub name: String,
    pub is_default: bool,
    pub channels: u16,
    pub sample_rates: Vec<u32>,
    pub sample_formats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackInfo {
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: String,
    pub seconds: f32,
}

pub fn list_output_devices() -> Result<Vec<OutputDeviceInfo>, AudioError> {
    let host = cpal::default_host();
    let default_name = host
        .default_output_device()
        .and_then(|device| device.name().ok());

    let mut result = Vec::new();
    for device in host
        .output_devices()
        .map_err(|e| AudioError::Output(e.to_string()))?
    {
        let name = device.name().unwrap_or_else(|_| "<unknown>".into());
        let mut sample_rates = Vec::new();
        let mut sample_formats = Vec::new();
        let mut channels = 0;
        if let Ok(configs) = device.supported_output_configs() {
            for config in configs {
                channels = channels.max(config.channels());
                for rate in [config.min_sample_rate().0, config.max_sample_rate().0] {
                    if !sample_rates.contains(&rate) {
                        sample_rates.push(rate);
                    }
                }
                let format = format!("{:?}", config.sample_format());
                if !sample_formats.contains(&format) {
                    sample_formats.push(format);
                }
            }
        }
        sample_rates.sort_unstable();
        result.push(OutputDeviceInfo {
            is_default: default_name.as_deref() == Some(name.as_str()),
            name,
            channels,
            sample_rates,
            sample_formats,
        });
    }
    Ok(result)
}

/// Play a short stereo sine wave and block until it finishes.
pub fn play_sine(
    device_name: Option<&str>,
    requested_rate: u32,
    seconds: f32,
    frequency: f32,
) -> Result<PlaybackInfo, AudioError> {
    let host = cpal::default_host();
    let device = match device_name {
        Some(name) => host
            .output_devices()
            .map_err(|e| AudioError::Output(e.to_string()))?
            .find(|device| device.name().map(|n| n == name).unwrap_or(false))
            .ok_or_else(|| AudioError::Output(format!("output device not found: {name}")))?,
        None => host
            .default_output_device()
            .ok_or_else(|| AudioError::Output("no default output device".into()))?,
    };
    let device_name = device.name().unwrap_or_else(|_| "<unknown>".into());

    let config = select_config(&device, requested_rate)?;
    let stream_config: StreamConfig = config.config();
    let sample_format = config.sample_format();
    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate.0;
    let step = TAU * frequency / sample_rate as f32;

    let err_fn = |err| eprintln!("output stream error: {err}");
    let mut phase = 0.0f32;
    let stream = match sample_format {
        SampleFormat::F32 => device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _| {
                    fill_f32(data, channels, &mut phase, step, 0.15);
                },
                err_fn,
                None,
            )
            .map_err(|e| AudioError::Output(e.to_string()))?,
        SampleFormat::I16 => device
            .build_output_stream(
                &stream_config,
                move |data: &mut [i16], _| {
                    fill_i16(data, channels, &mut phase, step, 0.15);
                },
                err_fn,
                None,
            )
            .map_err(|e| AudioError::Output(e.to_string()))?,
        SampleFormat::U16 => device
            .build_output_stream(
                &stream_config,
                move |data: &mut [u16], _| {
                    fill_u16(data, channels, &mut phase, step, 0.15);
                },
                err_fn,
                None,
            )
            .map_err(|e| AudioError::Output(e.to_string()))?,
        other => {
            return Err(AudioError::Output(format!(
                "unsupported sample format: {other:?}"
            )))
        }
    };

    stream
        .play()
        .map_err(|e| AudioError::Output(e.to_string()))?;
    std::thread::sleep(Duration::from_secs_f32(seconds + 0.1));
    stream
        .pause()
        .map_err(|e| AudioError::Output(e.to_string()))?;

    Ok(PlaybackInfo {
        device: device_name,
        sample_rate,
        channels: channels as u16,
        sample_format: format!("{sample_format:?}"),
        seconds,
    })
}

/// Sample formats this crate can write, most-preferred first.
fn format_rank(format: SampleFormat) -> Option<u8> {
    match format {
        SampleFormat::F32 => Some(0),
        SampleFormat::I16 => Some(1),
        SampleFormat::U16 => Some(2),
        _ => None,
    }
}

fn select_config(
    device: &cpal::Device,
    requested_rate: u32,
) -> Result<cpal::SupportedStreamConfig, AudioError> {
    let configs: Vec<_> = device
        .supported_output_configs()
        .map_err(|e| AudioError::Output(e.to_string()))?
        .collect();

    // Only pick a stereo config at the requested rate whose sample format the
    // writers above support, preferring f32 over integer formats. Some hosts
    // list u8 first, which would otherwise be selected and then rejected.
    let requested = configs
        .iter()
        .filter(|c| {
            c.channels() == 2
                && c.min_sample_rate().0 <= requested_rate
                && c.max_sample_rate().0 >= requested_rate
        })
        .filter_map(|c| format_rank(c.sample_format()).map(|rank| (rank, c)))
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, c)| c.with_sample_rate(SampleRate(requested_rate)));

    if let Some(config) = requested {
        return Ok(config);
    }

    let fallback = device
        .default_output_config()
        .map_err(|e| AudioError::Output(e.to_string()))?;
    if format_rank(fallback.sample_format()).is_none() {
        return Err(AudioError::Output(format!(
            "no supported output format for {}",
            device.name().unwrap_or_else(|_| "<unknown>".into())
        )));
    }
    Ok(fallback)
}

fn fill_f32(data: &mut [f32], channels: usize, phase: &mut f32, step: f32, amp: f32) {
    for frame in data.chunks_mut(channels) {
        let sample = amp * phase.sin();
        for out in frame {
            *out = sample;
        }
        advance(phase, step);
    }
}

fn fill_i16(data: &mut [i16], channels: usize, phase: &mut f32, step: f32, amp: f32) {
    for frame in data.chunks_mut(channels) {
        let sample = (amp * phase.sin() * i16::MAX as f32) as i16;
        for out in frame {
            *out = sample;
        }
        advance(phase, step);
    }
}

fn fill_u16(data: &mut [u16], channels: usize, phase: &mut f32, step: f32, amp: f32) {
    for frame in data.chunks_mut(channels) {
        let sample = ((amp * phase.sin() * 0.5 + 0.5) * u16::MAX as f32) as u16;
        for out in frame {
            *out = sample;
        }
        advance(phase, step);
    }
}

fn advance(phase: &mut f32, step: f32) {
    *phase += step;
    if *phase >= TAU {
        *phase -= TAU;
    }
}
