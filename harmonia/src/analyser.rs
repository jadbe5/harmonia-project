use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::f32::consts::PI;
use std::sync::Arc;

use crate::notes::GUITAR;

pub struct FrequencyAnalyzer {
    pub fft: Arc<dyn Fft<f32>>,
    pub window_table: Vec<f32>,
    pub scratch_buffer: Vec<Complex<f32>>,
    pub sample_rate: f32,
    pub fft_size: usize,
}

pub struct Note {
    pub name: String,
    pub _octave: i32,
    pub _cents: f32,
}

impl FrequencyAnalyzer {
    pub fn new(sample_rate: f32, fft_size: usize) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(fft_size);

        let window_table: Vec<f32> = (0..fft_size)
            .map(|i| {
                let n = i as f32;
                let n_total = (fft_size - 1) as f32;
                0.5 * (1.0 - (2.0 * PI * n / n_total).cos())
            })
            .collect();

        let scratch_size = fft.get_inplace_scratch_len();
        let scratch_buffer = vec![Complex { re: 0.0, im: 0.0 }; scratch_size];

        Self {
            fft,
            window_table,
            scratch_buffer,
            sample_rate,
            fft_size,
        }
    }

    pub fn apply_window(&self, audio_buffer: &mut [f32]) {
        if audio_buffer.len() != self.window_table.len() {
            return;
        }

        for (sample, window_value) in audio_buffer.iter_mut().zip(&self.window_table) {
            *sample *= window_value;
        }
    }

    pub fn compute_fft_magnitude(
        &mut self,
        windowed_audio: &[f32],
        complex_buffer: &mut [Complex<f32>],
        magnitudes: &mut [f32],
    ) {
        for complex in complex_buffer.iter_mut() {
            complex.re = 0.0;
            complex.im = 0.0;
        }

        for (complex, &sample) in complex_buffer.iter_mut().zip(windowed_audio.iter()) {
            complex.re = sample;
            complex.im = 0.0;
        }

        self.fft
            .process_with_scratch(complex_buffer, &mut self.scratch_buffer);

        let half_size = self.fft_size / 2;
        for (mag, complex) in magnitudes.iter_mut().zip(complex_buffer[..half_size].iter()) {
            *mag = complex.norm();
        }
    }

    pub fn find_frequency(&self, samples: &[f32]) -> Option<f32> {
        if samples.len() < 32 {
            return None;
        }

        let level = average_amplitude(samples);
        if level < 0.001 {
            return None;
        }

        let mut centered = samples.to_vec();
        remove_dc_offset(&mut centered);

        let min_freq = 70.0;
        let max_freq = 380.0;

        let mut freq = self.yin_frequency(&centered, min_freq, max_freq)?;
        freq = correct_guitar_octave(freq);

        Some(freq)
    }

    pub fn find_precise_frequency(&self, magnitudes: &[f32]) -> Option<f32> {
        if magnitudes.len() < 3 {
            return None;
        }

        let low_bin = hz_to_bin(70.0, self.sample_rate, self.fft_size).max(1);
        let high_bin = hz_to_bin(380.0, self.sample_rate, self.fft_size)
            .min(magnitudes.len() - 2);

        if low_bin >= high_bin {
            return None;
        }

        let mut max_idx = low_bin;
        let mut max_mag = 0.0;

        for i in low_bin..=high_bin {
            if magnitudes[i] > max_mag {
                max_mag = magnitudes[i];
                max_idx = i;
            }
        }

        if max_mag < 0.001 {
            return None;
        }

        let alpha = magnitudes[max_idx - 1];
        let beta = magnitudes[max_idx];
        let gamma = magnitudes[max_idx + 1];
        let denominator = alpha - 2.0 * beta + gamma;

        let offset = if denominator.abs() > 1e-6 {
            0.5 * (alpha - gamma) / denominator
        } else {
            0.0
        };

        let exact_bin = max_idx as f32 + offset;
        let freq = exact_bin * self.sample_rate / self.fft_size as f32;

        Some(correct_guitar_octave(freq))
    }

    fn yin_frequency(&self, samples: &[f32], min_freq: f32, max_freq: f32) -> Option<f32> {
        let min_tau = (self.sample_rate / max_freq).floor() as usize;
        let max_tau = (self.sample_rate / min_freq).ceil() as usize;

        if min_tau < 2 || max_tau >= samples.len() {
            return None;
        }

        let mut difference = vec![0.0f32; max_tau + 1];

        for tau in 1..=max_tau {
            let mut sum = 0.0;
            let limit = samples.len() - tau;
            for i in 0..limit {
                let delta = samples[i] - samples[i + tau];
                sum += delta * delta;
            }
            difference[tau] = sum;
        }

        let mut cmnd = vec![1.0f32; max_tau + 1];
        let mut running_sum = 0.0;

        for tau in 1..=max_tau {
            running_sum += difference[tau];
            if running_sum > 0.0 {
                cmnd[tau] = difference[tau] * tau as f32 / running_sum;
            }
        }

        let yin_threshold = 0.15;
        let mut tau = 0usize;

        for t in min_tau..=max_tau {
            if cmnd[t] < yin_threshold {
                tau = t;
                while tau + 1 <= max_tau && cmnd[tau + 1] < cmnd[tau] {
                    tau += 1;
                }
                break;
            }
        }

        if tau == 0 {
            let mut best_tau = min_tau;
            let mut best_value = cmnd[min_tau];

            for t in min_tau..=max_tau {
                if cmnd[t] < best_value {
                    best_value = cmnd[t];
                    best_tau = t;
                }
            }

            if best_value > 0.35 {
                return None;
            }

            tau = best_tau;
        }

        let precise_tau = parabolic_tau(&cmnd, tau);
        if precise_tau <= 0.0 {
            return None;
        }

        let freq = self.sample_rate / precise_tau;
        if freq < min_freq || freq > max_freq {
            return None;
        }

        Some(freq)
    }

    pub fn hz_to_note(&self, frequency: f32) -> Note {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

        let midi_note = 12.0 * (frequency / 440.0).log2() + 69.0;
        let closest_midi = midi_note.round() as i32;
        let _cents = (midi_note - closest_midi as f32) * 100.0;
        let note_index = closest_midi.rem_euclid(12) as usize;
        let _octave = (closest_midi / 12) - 1;

        Note {
            name: note_names[note_index].to_string(),
            _octave,
            _cents,
        }
    }
}

fn average_amplitude(samples: &[f32]) -> f32 {
    let mut sum = 0.0;
    for &sample in samples {
        sum += sample.abs();
    }
    sum / samples.len() as f32
}

fn remove_dc_offset(samples: &mut [f32]) {
    let mut mean = 0.0;
    for &sample in samples.iter() {
        mean += sample;
    }
    mean /= samples.len() as f32;

    for sample in samples.iter_mut() {
        *sample -= mean;
    }
}

fn parabolic_tau(values: &[f32], tau: usize) -> f32 {
    if tau == 0 || tau + 1 >= values.len() {
        return tau as f32;
    }

    let left = values[tau - 1];
    let center = values[tau];
    let right = values[tau + 1];
    let denominator = left - 2.0 * center + right;

    if denominator.abs() < 1e-6 {
        tau as f32
    } else {
        tau as f32 + 0.5 * (left - right) / denominator
    }
}

fn hz_to_bin(freq: f32, sample_rate: f32, fft_size: usize) -> usize {
    (freq * fft_size as f32 / sample_rate).round() as usize
}

fn correct_guitar_octave(freq: f32) -> f32 {
    let candidates = [freq / 2.0, freq, freq * 2.0];

    let mut best_freq = freq;
    let mut best_error = f32::MAX;

    for candidate in candidates {
        if !(70.0..=380.0).contains(&candidate) {
            continue;
        }

        for note in GUITAR.iter() {
            let error = (1200.0 * (candidate / note.frequency).log2()).abs();
            if error < best_error {
                best_error = error;
                best_freq = candidate;
            }
        }
    }

    best_freq
}
