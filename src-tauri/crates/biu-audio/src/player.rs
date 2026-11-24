use std::collections::VecDeque;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, Stream, StreamConfig};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

use crate::AudioError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Idle,
    Loading,
    Playing,
    Paused,
    Stopped,
    Error,
}

impl PlaybackState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Loading => "loading",
            Self::Playing => "playing",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
            Self::Error => "error",
        }
    }

    fn code(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Loading => 1,
            Self::Playing => 2,
            Self::Paused => 3,
            Self::Stopped => 4,
            Self::Error => 5,
        }
    }

    fn from_code(code: u8) -> Self {
        match code {
            1 => Self::Loading,
            2 => Self::Playing,
            3 => Self::Paused,
            4 => Self::Stopped,
            5 => Self::Error,
            _ => Self::Idle,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerSnapshot {
    pub state: PlaybackState,
    pub path: Option<String>,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub volume: f32,
    pub error: Option<String>,
}

struct Shared {
    buffer: Mutex<VecDeque<f32>>,
    state: AtomicU8,
    position_frames: AtomicU64,
    duration_ms: AtomicU64,
    src_rate: AtomicU32,
    src_channels: AtomicU16,
    out_rate: AtomicU32,
    volume: AtomicU32,
    eof: AtomicBool,
    started: AtomicBool,
    path: Mutex<Option<String>>,
    error: Mutex<Option<String>>,
}

impl Shared {
    fn new() -> Self {
        Self {
            buffer: Mutex::new(VecDeque::new()),
            state: AtomicU8::new(PlaybackState::Idle.code()),
            position_frames: AtomicU64::new(0),
            duration_ms: AtomicU64::new(0),
            src_rate: AtomicU32::new(0),
            src_channels: AtomicU16::new(0),
            out_rate: AtomicU32::new(44_100),
            volume: AtomicU32::new(1.0f32.to_bits()),
            eof: AtomicBool::new(false),
            started: AtomicBool::new(false),
            path: Mutex::new(None),
            error: Mutex::new(None),
        }
    }

    fn set_state(&self, state: PlaybackState) {
        self.state.store(state.code(), Ordering::Relaxed);
    }

    fn state(&self) -> PlaybackState {
        PlaybackState::from_code(self.state.load(Ordering::Relaxed))
    }

    fn set_error(&self, message: impl Into<String>) {
        *self.error.lock().unwrap() = Some(message.into());
        self.set_state(PlaybackState::Error);
    }

    fn snapshot(&self) -> PlayerSnapshot {
        let out_rate = self.out_rate.load(Ordering::Relaxed).max(1);
        let frames = self.position_frames.load(Ordering::Relaxed);
        PlayerSnapshot {
            state: self.state(),
            path: self.path.lock().unwrap().clone(),
            position_ms: frames.saturating_mul(1000) / out_rate as u64,
            duration_ms: self.duration_ms.load(Ordering::Relaxed),
            sample_rate: self.src_rate.load(Ordering::Relaxed),
            channels: self.src_channels.load(Ordering::Relaxed),
            volume: f32::from_bits(self.volume.load(Ordering::Relaxed)),
            error: self.error.lock().unwrap().clone(),
        }
    }
}

enum Command {
    Load(PathBuf),
    Play,
    Pause,
    Stop,
    Seek(u64),
    SetVolume(f32),
    Shutdown,
}

/// A player handle. The cpal stream and decoder live on a dedicated thread,
/// so the handle is `Send + Sync` and never blocks a Tauri command.
pub struct Player {
    tx: Sender<Command>,
    shared: Arc<Shared>,
}

impl Player {
    pub fn new() -> Result<Self, AudioError> {
        let shared = Arc::new(Shared::new());
        let (tx, rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();

        let worker_shared = shared.clone();
        thread::Builder::new()
            .name("biu-audio-player".into())
            .spawn(move || worker(worker_shared, rx, ready_tx))
            .map_err(|e| AudioError::Output(e.to_string()))?;

        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => Ok(Self { tx, shared }),
            Ok(Err(message)) => Err(AudioError::Output(message)),
            Err(_) => Err(AudioError::Output("audio device init timeout".into())),
        }
    }

    pub fn load(&self, path: impl AsRef<Path>) {
        let _ = self.tx.send(Command::Load(path.as_ref().to_path_buf()));
    }

    pub fn play(&self) {
        let _ = self.tx.send(Command::Play);
    }

    pub fn pause(&self) {
        let _ = self.tx.send(Command::Pause);
    }

    pub fn stop(&self) {
        let _ = self.tx.send(Command::Stop);
    }

    pub fn seek_ms(&self, position_ms: u64) {
        let _ = self.tx.send(Command::Seek(position_ms));
    }

    pub fn set_volume(&self, volume: f32) {
        let _ = self.tx.send(Command::SetVolume(volume.clamp(0.0, 1.0)));
    }

    pub fn snapshot(&self) -> PlayerSnapshot {
        self.shared.snapshot()
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Shutdown);
    }
}

fn worker(shared: Arc<Shared>, rx: mpsc::Receiver<Command>, ready: Sender<Result<(), String>>) {
    let output = match Output::new(shared.clone()) {
        Ok(output) => {
            let _ = ready.send(Ok(()));
            output
        }
        Err(err) => {
            let _ = ready.send(Err(err.to_string()));
            return;
        }
    };
    let out_rate = output.rate;
    shared.out_rate.store(out_rate, Ordering::Relaxed);

    let mut reader: Option<TrackReader> = None;
    let mut resampler: Option<Resampler> = None;

    loop {
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                Command::Shutdown => {
                    let _ = output.stream.pause();
                    return;
                }
                Command::Load(path) => match TrackReader::open(&path) {
                    Ok(track) => {
                        shared.buffer.lock().unwrap().clear();
                        shared.position_frames.store(0, Ordering::Relaxed);
                        shared.eof.store(false, Ordering::Relaxed);
                        shared.started.store(false, Ordering::Relaxed);
                        shared
                            .duration_ms
                            .store(track.duration_ms, Ordering::Relaxed);
                        shared.src_rate.store(track.sample_rate, Ordering::Relaxed);
                        shared.src_channels.store(track.channels, Ordering::Relaxed);
                        *shared.path.lock().unwrap() = Some(path.display().to_string());
                        *shared.error.lock().unwrap() = None;
                        shared.set_state(PlaybackState::Loading);
                        resampler =
                            Some(Resampler::new(track.sample_rate, out_rate, track.channels));
                        reader = Some(track);
                    }
                    Err(err) => {
                        reader = None;
                        shared.set_error(err.to_string());
                    }
                },
                Command::Play => {
                    if reader.is_some() {
                        if shared.state() == PlaybackState::Stopped {
                            shared.position_frames.store(0, Ordering::Relaxed);
                        }
                        shared.set_state(PlaybackState::Playing);
                    }
                }
                Command::Pause => {
                    if shared.state() == PlaybackState::Playing {
                        shared.set_state(PlaybackState::Paused);
                    }
                }
                Command::Stop => {
                    reader = None;
                    shared.buffer.lock().unwrap().clear();
                    shared.position_frames.store(0, Ordering::Relaxed);
                    shared.started.store(false, Ordering::Relaxed);
                    shared.eof.store(false, Ordering::Relaxed);
                    shared.set_state(PlaybackState::Stopped);
                }
                Command::Seek(position_ms) => {
                    if let Some(track) = reader.as_mut() {
                        if let Err(err) = track.seek_ms(position_ms) {
                            shared.set_error(err.to_string());
                        } else {
                            shared.buffer.lock().unwrap().clear();
                            let frames = position_ms.saturating_mul(out_rate as u64) / 1000;
                            shared.position_frames.store(frames, Ordering::Relaxed);
                            shared.eof.store(false, Ordering::Relaxed);
                            let state = shared.state();
                            if state == PlaybackState::Playing {
                                shared.set_state(PlaybackState::Playing);
                            }
                        }
                    }
                }
                Command::SetVolume(value) => {
                    shared.volume.store(value.to_bits(), Ordering::Relaxed);
                }
            }
        }

        if shared.state() == PlaybackState::Playing {
            let queued = shared.buffer.lock().unwrap().len() as u64;
            let max_frames = (out_rate as u64).saturating_mul(2);
            if queued < max_frames * 2 {
                if let (Some(track), Some(resampler)) = (reader.as_mut(), resampler.as_mut()) {
                    match track.next_chunk() {
                        Ok(Some(samples)) => {
                            let mut out = Vec::with_capacity(samples.len());
                            resampler.process(&samples, &mut out);
                            shared.buffer.lock().unwrap().extend(out);
                            if shared.started.load(Ordering::Relaxed) {
                                shared.set_state(PlaybackState::Playing);
                            }
                        }
                        Ok(None) => {
                            shared.eof.store(true, Ordering::Relaxed);
                        }
                        Err(err) => {
                            shared.set_error(err.to_string());
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(8));
    }
}

struct Output {
    stream: Stream,
    rate: u32,
}

impl Output {
    fn new(shared: Arc<Shared>) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| AudioError::Output("no default output device".into()))?;
        let supported = device
            .default_output_config()
            .map_err(|e| AudioError::Output(e.to_string()))?;
        let sample_format = supported.sample_format();
        let config: StreamConfig = supported.config();
        let rate = config.sample_rate.0;
        let channels = config.channels as usize;

        let err_fn = |err| eprintln!("player output error: {err}");
        let stream = match sample_format {
            SampleFormat::F32 => build::<f32>(&device, &config, channels, shared, err_fn)?,
            SampleFormat::I16 => build::<i16>(&device, &config, channels, shared, err_fn)?,
            SampleFormat::U16 => build::<u16>(&device, &config, channels, shared, err_fn)?,
            other => {
                return Err(AudioError::Output(format!(
                    "unsupported sample format: {other:?}"
                )))
            }
        };
        stream
            .play()
            .map_err(|e| AudioError::Output(e.to_string()))?;
        Ok(Self { stream, rate })
    }
}

fn build<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    channels: usize,
    shared: Arc<Shared>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, AudioError>
where
    T: SizedSample + FromSample<f32>,
{
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| fill(data, channels, &shared),
            err_fn,
            None,
        )
        .map_err(|e| AudioError::Output(e.to_string()))
}

fn fill<T>(data: &mut [T], channels: usize, shared: &Shared)
where
    T: SizedSample + FromSample<f32>,
{
    let volume = f32::from_bits(shared.volume.load(Ordering::Relaxed));
    let playing = shared.state() == PlaybackState::Playing;
    let mut buffer = match shared.buffer.try_lock() {
        Ok(buffer) => buffer,
        Err(_) => {
            for sample in data.iter_mut() {
                *sample = T::from_sample(0.0);
            }
            return;
        }
    };

    let frames = data.len() / channels.max(1);
    for frame in data.chunks_mut(channels.max(1)) {
        let left = if playing {
            buffer.pop_front().unwrap_or(0.0)
        } else {
            0.0
        };
        let right = if playing {
            buffer.pop_front().unwrap_or(left)
        } else {
            0.0
        };
        for (index, sample) in frame.iter_mut().enumerate() {
            let value = match index {
                0 => left,
                1 => right,
                _ => 0.0,
            };
            *sample = T::from_sample(value * volume);
        }
    }

    let empty = buffer.is_empty();
    drop(buffer);

    if playing {
        shared
            .position_frames
            .fetch_add(frames as u64, Ordering::Relaxed);
        shared.started.store(true, Ordering::Relaxed);
        if empty && shared.eof.load(Ordering::Relaxed) {
            shared.set_state(PlaybackState::Stopped);
        }
    }
}

struct Resampler {
    src_rate: u32,
    dst_rate: u32,
    channels: usize,
    acc: u64,
}

impl Resampler {
    fn new(src_rate: u32, dst_rate: u32, channels: u16) -> Self {
        Self {
            src_rate: src_rate.max(1),
            dst_rate: dst_rate.max(1),
            channels: channels.max(1) as usize,
            acc: 0,
        }
    }

    /// Nearest-neighbour resampling. Good enough for the MVP; replaced by
    /// rubato in A1-05/DSP.
    fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        let frames = input.len() / self.channels;
        for index in 0..frames {
            self.acc += self.dst_rate as u64;
            while self.acc >= self.src_rate as u64 {
                self.acc -= self.src_rate as u64;
                let base = index * self.channels;
                let left = input[base];
                let right = if self.channels > 1 {
                    input[base + 1]
                } else {
                    left
                };
                out.push(left);
                out.push(right);
            }
        }
    }
}

struct TrackReader {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    sample_rate: u32,
    channels: u16,
    duration_ms: u64,
}

impl TrackReader {
    fn open(path: &Path) -> Result<Self, AudioError> {
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|v| v.to_str()) {
            hint.with_extension(ext);
        }
        let probed = symphonia::default::get_probe().format(
            &hint,
            mss,
            &FormatOptions {
                enable_gapless: true,
                ..Default::default()
            },
            &MetadataOptions::default(),
        )?;
        let mut format = probed.format;
        let track = format
            .default_track()
            .ok_or_else(|| AudioError::Unsupported("no default audio track".into()))?;
        let track_id = track.id;
        let params = track.codec_params.clone();
        let mut sample_rate = params.sample_rate.unwrap_or(0);
        let mut channels = params.channels.map(|c| c.count()).unwrap_or(0);
        let duration_ms = params
            .n_frames
            .map(|frames| frames * 1000 / sample_rate.max(1) as u64)
            .unwrap_or(0);

        let mut decoder = symphonia::default::get_codecs()
            .make(&params, &DecoderOptions::default())
            .map_err(|e| AudioError::Unsupported(e.to_string()))?;

        if sample_rate == 0 || channels == 0 {
            let (rate, ch) = first_spec(&mut format, decoder.as_mut(), track_id)?;
            sample_rate = rate;
            channels = ch;
        }

        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            channels: channels as u16,
            duration_ms,
        })
    }

    fn next_chunk(&mut self) -> Result<Option<Vec<f32>>, AudioError> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(err))
                    if err.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    return Ok(None)
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(err) => return Err(err.into()),
            };
            if packet.track_id() != self.track_id {
                continue;
            }
            let trim_start = packet.trim_start as usize;
            let trim_end = packet.trim_end as usize;
            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(err) => return Err(err.into()),
            };
            let spec = *decoded.spec();
            let frames = decoded.frames();
            if frames == 0 {
                continue;
            }
            let mut samples = SampleBuffer::<f32>::new(frames as u64, spec);
            samples.copy_interleaved_ref(decoded);
            let channels = spec.channels.count();
            let raw = samples.samples();
            let start = (trim_start * channels).min(raw.len());
            let end = raw.len().saturating_sub(trim_end * channels).max(start);
            if start >= end {
                continue;
            }
            return Ok(Some(raw[start..end].to_vec()));
        }
    }

    fn seek_ms(&mut self, position_ms: u64) -> Result<(), AudioError> {
        let time = Time::new(position_ms / 1000, (position_ms % 1000) as f64 / 1000.0);
        self.format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| AudioError::Decode(e.to_string()))?;
        self.decoder.reset();
        Ok(())
    }
}

fn first_spec(
    format: &mut Box<dyn FormatReader>,
    decoder: &mut dyn Decoder,
    track_id: u32,
) -> Result<(u32, usize), AudioError> {
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                return Err(AudioError::Corrupt("truncated audio stream".into()))
            }
            Err(err) => return Err(err.into()),
        };
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) if decoded.frames() > 0 => {
                return Ok((decoded.spec().rate, decoded.spec().channels.count()))
            }
            Ok(_) => continue,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(err) => return Err(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_codes_roundtrip() {
        for state in [
            PlaybackState::Idle,
            PlaybackState::Loading,
            PlaybackState::Playing,
            PlaybackState::Paused,
            PlaybackState::Stopped,
            PlaybackState::Error,
        ] {
            assert_eq!(PlaybackState::from_code(state.code()), state);
        }
    }

    #[test]
    fn resampler_produces_expected_frame_count() {
        let mut resampler = Resampler::new(44_100, 48_000, 2);
        let input = vec![0.25f32; 44_100 * 2];
        let mut out = Vec::new();
        resampler.process(&input, &mut out);
        assert!((out.len() as i64 - 48_000 * 2).abs() <= 4, "{}", out.len());
        assert!(out.iter().all(|s| (*s - 0.25).abs() < f32::EPSILON));
    }

    #[test]
    fn mono_input_is_duplicated_to_stereo() {
        let mut resampler = Resampler::new(48_000, 48_000, 1);
        let mut out = Vec::new();
        resampler.process(&[0.5, -0.5], &mut out);
        assert_eq!(out, vec![0.5, 0.5, -0.5, -0.5]);
    }
}
