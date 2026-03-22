use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

pub struct AudioChunk
{
    pub samples: Vec<f32>,
    pub _sample_rate: u32,
    pub _level: f32,
    pub _threshold: f32,
}

pub struct AudioConfig
{
    pub target_sample_rate: u32,
    pub frame_size: usize,
    pub hop_size: usize,
    pub queue_capacity: usize,
    pub calibration_frames: usize,
    pub noise_margin: f32,
    pub max_gain: f32,
}

impl Default for AudioConfig
{
    fn default() -> Self {
        Self {
            target_sample_rate: 44_100,
            frame_size: 4096,
            hop_size: 1024,
            queue_capacity: 16,
            calibration_frames: 6,
            noise_margin: 2.0,
            max_gain: 4.0,
        }
    }
}

pub struct AudioInput
{
    _stream: cpal::Stream,
    receiver: Receiver<AudioChunk>,
}

struct SharedState
{
    rolling: VecDeque<f32>,
    noise_sum: f32,
    noise_count: usize,
    calibrated: bool,
    threshold: f32,
}

struct SelectedConfig
{
    stream_config: cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
}

impl AudioInput
{
    pub fn start(config: AudioConfig) -> Result<Self, Box<dyn std::error::Error>>
    {
        if config.frame_size == 0 || config.hop_size == 0 || config.hop_size > config.frame_size
        {
            return Err("invalid audio config".into());
        }

        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("no input device found")?;
        let selected = select_input_config(&device, config.target_sample_rate)?;
        let (sender, receiver) = bounded(config.queue_capacity);

        let state = Arc::new(Mutex::new(SharedState{
            rolling: VecDeque::with_capacity(config.frame_size * 2),
            noise_sum: 0.0,
            noise_count: 0,
            calibrated: false,
            threshold: 0.01,
        }));

        let stream = match selected.sample_format
        {
            cpal::SampleFormat::F32 =>
            {
                build_stream::<f32>(&device, &selected.stream_config, sender, state, config)?
            }
            cpal::SampleFormat::I16 =>
            {
                build_stream::<i16>(&device, &selected.stream_config, sender, state, config)?
            }
            cpal::SampleFormat::U16 =>
            {
                build_stream::<u16>(&device, &selected.stream_config, sender, state, config)?
            }
            _ => return Err("unsupported sample format".into()),
        };

        stream.play()?;

        Ok(Self {
            _stream: stream,
            receiver,
        })
    }

    pub fn recv(&self) -> Result<AudioChunk, crossbeam_channel::RecvError>
    {
        self.receiver.recv()
    }
    /*
    pub fn try_recv(&self) -> Result<AudioChunk, crossbeam_channel::TryRecvError>
    {
        self.receiver.try_recv()
    }
    */
}

fn select_input_config(
    device: &cpal::Device,
    target_sample_rate: u32,
) -> Result<SelectedConfig, Box<dyn std::error::Error>>
{
    let configs = device.supported_input_configs()?;

    let mut best_score: Option<(u32, u32, u32)> = None;
    let mut best_config: Option<SelectedConfig> = None;

    for range in configs
    {
        let channels = range.channels();
        let sample_format = range.sample_format();
        let min_rate = range.min_sample_rate().0;
        let max_rate = range.max_sample_rate().0;

        let chosen_rate = if target_sample_rate < min_rate
        {
            min_rate
        }
        else if target_sample_rate > max_rate
        {
            max_rate
        }
        else
        {
            target_sample_rate
        };

        let mono_penalty = if channels == 1 { 0 } else { 100 + channels as u32 };

        let format_penalty = match sample_format
        {
            cpal::SampleFormat::F32 => 0,
            cpal::SampleFormat::I16 => 1,
            cpal::SampleFormat::U16 => 2,
            _ => 10,
        };

        let rate_penalty = chosen_rate.abs_diff(target_sample_rate);

        let score = (mono_penalty, format_penalty, rate_penalty);

        if best_score.is_none() || score < best_score.unwrap()
        {
            best_score = Some(score);
            best_config = Some(SelectedConfig{
                stream_config: cpal::StreamConfig {
                    channels,
                   sample_rate: cpal::SampleRate(chosen_rate),
                    buffer_size: cpal::BufferSize::Default,
                },
                sample_format,
            });
        }
    }

    best_config.ok_or_else(|| "no supported input config found".into())
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sender: Sender<AudioChunk>,
    state: Arc<Mutex<SharedState>>,
    audio_config: AudioConfig,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    let sample_rate = config.sample_rate.0;

    device.build_input_stream(
        config,
        move |data: &[T], _| {
            process_input(data, channels, sample_rate, &sender, &state, &audio_config);
        },
        move |err| {
            eprintln!("audio stream error: {}", err);
        },
        None,
    )
}

fn process_input<T>(
    data: &[T],
    channels: usize,
    _sample_rate: u32,
    sender: &Sender<AudioChunk>,
    state: &Arc<Mutex<SharedState>>,
    config: &AudioConfig,
) where
    T: cpal::Sample,
    f32: cpal::FromSample<T>,
{
    if channels == 0
    {
        return;
    }

    let mut state = match state.lock()
    {
        Ok(guard) => guard,
        Err(_) => return,
    };

    for frame in data.chunks(channels)
    {
        let mut mono = 0.0;
        for &sample in frame
        {
            let value: f32 = sample.to_sample::<f32>();
            mono += value;
        }

        mono /= frame.len() as f32;
        state.rolling.push_back(mono);
    }

    while state.rolling.len() >= config.frame_size
    {
        let mut samples = state
            .rolling
            .iter()
            .take(config.frame_size)
            .copied()
            .collect::<Vec<f32>>();

        remove_dc_offset(&mut samples);

        let _level = average_amplitude(&samples);

        if !state.calibrated
        {
            state.noise_sum += _level;
            state.noise_count += 1;

            if state.noise_count >= config.calibration_frames
            {
                let noise_floor = state.noise_sum / state.noise_count as f32;
                state.threshold = (noise_floor * config.noise_margin).max(0.005);
                state.calibrated = true;
            }
        }

        if state.calibrated && _level >= state.threshold
        {
            normalize_gain(&mut samples, config.max_gain);
        }

        let chunk = AudioChunk
        {
            samples,
            _sample_rate,
            _level,
            _threshold: state.threshold,
        };

        match sender.try_send(chunk)
        {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => return,
        }

        advance_buffer(&mut state.rolling, config.hop_size);
    }
}

fn advance_buffer(buffer: &mut VecDeque<f32>, hop_size: usize)
{
    for _ in 0..hop_size
    {
        buffer.pop_front();
    }
}

fn average_amplitude(samples: &[f32]) -> f32
{
    let mut sum = 0.0;

    for &sample in samples
    {
        sum += sample.abs();
    }
    sum / samples.len() as f32
}

fn remove_dc_offset(samples: &mut [f32])
{
    let mut mean = 0.0;
    for &sample in samples.iter()
    {
        mean += sample;
    }

    mean /= samples.len() as f32;

    for sample in samples.iter_mut()
    {
        *sample -= mean;
    }
}

fn normalize_gain(samples: &mut [f32], max_gain: f32)
{
    let mut peak = 0.0;
    for &sample in samples.iter()
    {
        let value = sample.abs();
        if value > peak
        {
            peak = value;
        }
    }

    if peak < 1e-6
    {
        return;
    }

    let gain = (0.8 / peak).min(max_gain);
    for sample in samples.iter_mut()
    {
        *sample = (*sample * gain).clamp(-1.0, 1.0);
    }
}
