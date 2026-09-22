use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::error::{AppError, AppResult};

const TARGET_SAMPLE_RATE: u32 = 16_000;
/// Length of each audio chunk sent to Whisper, in seconds.
const CHUNK_SECONDS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    Microphone,
    SystemAudio,
}

impl CaptureKind {
    pub fn source_label(&self) -> &'static str {
        match self {
            CaptureKind::Microphone => "microphone",
            CaptureKind::SystemAudio => "system-audio",
        }
    }
}

/// A running audio-capture session. Samples are collected on a dedicated OS
/// thread (cpal streams are not `Send`) into a shared buffer; a separate async
/// task drains that buffer in fixed-size chunks for transcription.
pub struct AudioSession {
    running: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<AtomicU32>,
    kind: CaptureKind,
}

impl AudioSession {
    pub fn kind(&self) -> CaptureKind {
        self.kind
    }

    /// Signals the capture thread to stop. The stream is dropped when the
    /// thread observes the flag.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// Starts capturing audio for the given source. Returns a session handle plus
/// the shared state needed by the caller to drive transcription.
pub fn start_capture(kind: CaptureKind) -> AppResult<AudioSession> {
    let running = Arc::new(AtomicBool::new(true));
    let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let sample_rate = Arc::new(AtomicU32::new(TARGET_SAMPLE_RATE));

    let thread_running = running.clone();
    let thread_buffer = buffer.clone();
    let thread_rate = sample_rate.clone();

    // The stream lives entirely inside this thread.
    let (init_tx, init_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    thread::Builder::new()
        .name(format!("audio-{}", kind.source_label()))
        .spawn(move || {
            match build_stream(kind, thread_buffer, thread_rate) {
                Ok(stream) => {
                    if let Err(e) = stream.play() {
                        let _ = init_tx.send(Err(e.to_string()));
                        return;
                    }
                    let _ = init_tx.send(Ok(()));
                    // Keep the stream alive until asked to stop.
                    while thread_running.load(Ordering::SeqCst) {
                        thread::sleep(Duration::from_millis(100));
                    }
                    drop(stream);
                }
                Err(e) => {
                    let _ = init_tx.send(Err(e.to_string()));
                }
            }
        })
        .map_err(|e| AppError::Audio(e.to_string()))?;

    // Wait for the stream to initialise so errors surface synchronously.
    match init_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(AppError::Audio(e)),
        Err(_) => return Err(AppError::Audio("audio stream did not start".into())),
    }

    Ok(AudioSession {
        running,
        buffer,
        sample_rate,
        kind,
    })
}

impl AudioSession {
    /// Drains up to one chunk of buffered audio, returning mono 16 kHz WAV bytes
    /// ready for the Whisper API. Returns `None` if not enough audio is buffered.
    pub fn take_chunk_wav(&self) -> Option<Vec<u8>> {
        let device_rate = self.sample_rate.load(Ordering::SeqCst).max(1);
        let needed = device_rate as usize * CHUNK_SECONDS;

        let samples = {
            let mut buf = self.buffer.lock();
            if buf.len() < needed {
                return None;
            }
            let drained: Vec<f32> = buf.drain(..needed).collect();
            drained
        };

        let resampled = resample_to_target(&samples, device_rate);
        encode_wav(&resampled).ok()
    }

    /// Flushes any remaining buffered audio (used when stopping).
    pub fn take_remaining_wav(&self) -> Option<Vec<u8>> {
        let device_rate = self.sample_rate.load(Ordering::SeqCst).max(1);
        let samples = {
            let mut buf = self.buffer.lock();
            if buf.len() < device_rate as usize / 2 {
                buf.clear();
                return None;
            }
            std::mem::take(&mut *buf)
        };
        let resampled = resample_to_target(&samples, device_rate);
        encode_wav(&resampled).ok()
    }
}

fn build_stream(
    kind: CaptureKind,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<AtomicU32>,
) -> AppResult<cpal::Stream> {
    let host = cpal::default_host();
    let device = select_device(&host, kind)?;

    let config = device
        .default_input_config()
        .map_err(|e| AppError::Audio(e.to_string()))?;

    let channels = config.channels() as usize;
    sample_rate.store(config.sample_rate().0, Ordering::SeqCst);

    let err_fn = |err| log::error!("audio stream error: {err}");
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();

    // Downmix interleaved frames to mono and append to the shared buffer.
    let push = move |mono: f32, buffer: &Arc<Mutex<Vec<f32>>>| {
        buffer.lock().push(mono);
    };

    let stream = match sample_format {
        SampleFormat::F32 => {
            let buffer = buffer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _| {
                    for frame in data.chunks(channels) {
                        let sum: f32 = frame.iter().copied().sum();
                        push(sum / channels as f32, &buffer);
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::I16 => {
            let buffer = buffer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    for frame in data.chunks(channels) {
                        let sum: f32 = frame
                            .iter()
                            .map(|s| *s as f32 / i16::MAX as f32)
                            .sum();
                        push(sum / channels as f32, &buffer);
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::U16 => {
            let buffer = buffer.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[u16], _| {
                    for frame in data.chunks(channels) {
                        let sum: f32 = frame
                            .iter()
                            .map(|s| (*s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                            .sum();
                        push(sum / channels as f32, &buffer);
                    }
                },
                err_fn,
                None,
            )
        }
        other => {
            return Err(AppError::Audio(format!(
                "unsupported sample format: {other:?}"
            )))
        }
    }
    .map_err(|e| AppError::Audio(e.to_string()))?;

    Ok(stream)
}

/// Chooses the input device for the requested capture source. For system audio
/// we look for a loopback/monitor device exposed by the OS; if none is found we
/// fall back to the default input device.
fn select_device(host: &cpal::Host, kind: CaptureKind) -> AppResult<Device> {
    match kind {
        CaptureKind::Microphone => host
            .default_input_device()
            .ok_or_else(|| AppError::Audio("no microphone found".into())),
        CaptureKind::SystemAudio => {
            if let Ok(devices) = host.input_devices() {
                for device in devices {
                    if let Ok(name) = device.name() {
                        let lower = name.to_lowercase();
                        if lower.contains("loopback")
                            || lower.contains("monitor")
                            || lower.contains("stereo mix")
                            || lower.contains("blackhole")
                            || lower.contains("what u hear")
                        {
                            return Ok(device);
                        }
                    }
                }
            }
            // Fall back so the feature degrades gracefully.
            host.default_input_device().ok_or_else(|| {
                AppError::Audio(
                    "no system-audio loopback device found. Install a loopback \
                     driver (e.g. BlackHole on macOS, VB-Cable on Windows)."
                        .into(),
                )
            })
        }
    }
}

/// Naive linear resampling to the Whisper target rate.
fn resample_to_target(samples: &[f32], from_rate: u32) -> Vec<f32> {
    if from_rate == TARGET_SAMPLE_RATE || samples.is_empty() {
        return samples.to_vec();
    }
    let ratio = TARGET_SAMPLE_RATE as f32 / from_rate as f32;
    let out_len = (samples.len() as f32 * ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f32 / ratio;
        let idx = src.floor() as usize;
        let frac = src - idx as f32;
        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac);
    }
    out
}

fn encode_wav(samples: &[f32]) -> AppResult<Vec<u8>> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| AppError::Audio(e.to_string()))?;
        for &sample in samples {
            let clamped = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(clamped)
                .map_err(|e| AppError::Audio(e.to_string()))?;
        }
        writer
            .finalize()
            .map_err(|e| AppError::Audio(e.to_string()))?;
    }
    Ok(cursor.into_inner())
}
