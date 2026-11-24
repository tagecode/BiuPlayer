use std::fs::File;
use std::path::Path;

use biu_core::{AudioFormat, DecodeStats, ProbeInfo};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{
    CodecType, DecoderOptions, CODEC_TYPE_AAC, CODEC_TYPE_ALAC, CODEC_TYPE_FLAC, CODEC_TYPE_MP3,
    CODEC_TYPE_VORBIS,
};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported or unreadable media: {0}")]
    Unsupported(String),
    #[error("decode error: {0}")]
    Decode(String),
    #[error("output error: {0}")]
    Output(String),
}

impl From<SymphoniaError> for AudioError {
    fn from(value: SymphoniaError) -> Self {
        match value {
            SymphoniaError::IoError(err) => Self::Io(err),
            SymphoniaError::Unsupported(what) => Self::Unsupported(what.to_string()),
            other => Self::Decode(other.to_string()),
        }
    }
}

/// Fully decode a file into f32 samples to prove the codec path works.
pub fn probe_path(path: impl AsRef<Path>) -> Result<ProbeInfo, AudioError> {
    let path = path.as_ref();
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
    let codec = codec_name(params.codec);
    let ext = path.extension().and_then(|v| v.to_str()).unwrap_or("");
    let mut format_kind = AudioFormat::from_extension(ext);
    if codec == "alac" {
        format_kind = AudioFormat::Alac;
    } else if codec == "aac" {
        format_kind = AudioFormat::Aac;
    }

    let mut decoder = symphonia::default::get_codecs()
        .make(&params, &DecoderOptions::default())
        .map_err(|e| AudioError::Unsupported(e.to_string()))?;

    let mut stats = DecodeStats::default();
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(err) => return Err(err.into()),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(err) => return Err(err.into()),
        };

        let spec = *decoded.spec();
        if sample_rate == 0 {
            sample_rate = spec.rate;
        }
        if channels == 0 {
            channels = spec.channels.count();
        }
        let mut samples = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        samples.copy_interleaved_ref(decoded);
        for sample in samples.samples() {
            let abs = sample.abs();
            if abs > stats.peak {
                stats.peak = abs;
            }
        }
        if spec.channels.count() > 0 {
            stats.frames += (samples.samples().len() / spec.channels.count()) as u64;
        }
    }

    if channels == 0 || sample_rate == 0 {
        return Err(AudioError::Unsupported(
            "missing channel or sample-rate metadata".into(),
        ));
    }

    let duration_ms = params
        .n_frames
        .map(|frames| frames * 1000 / sample_rate as u64)
        .unwrap_or_else(|| stats.frames * 1000 / sample_rate as u64);

    Ok(ProbeInfo {
        path: path.display().to_string(),
        format: format_kind,
        codec,
        sample_rate,
        channels,
        duration_ms,
        decoded_frames: stats.frames,
        peak_dbfs: stats.peak_dbfs(),
    })
}

fn codec_name(codec: CodecType) -> String {
    match codec {
        CODEC_TYPE_MP3 => "mp3".into(),
        CODEC_TYPE_AAC => "aac".into(),
        CODEC_TYPE_ALAC => "alac".into(),
        CODEC_TYPE_VORBIS => "vorbis".into(),
        CODEC_TYPE_FLAC => "flac".into(),
        _ => "pcm/other".into(),
    }
}
