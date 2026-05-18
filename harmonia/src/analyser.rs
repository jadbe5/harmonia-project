use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::f32::consts::PI;
use std::sync::Arc;

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
        assert_eq!(
            audio_buffer.len(),
            self.window_table.len(),
            "Audio buffer size ({}) doesn't match with window_table size ({}).",
            audio_buffer.len(),
            self.window_table.len()
        );

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
        for (complex, &sample) in complex_buffer.iter_mut().zip(windowed_audio.iter()) {
            complex.re = sample;
            complex.im = 0.0;
        }

        self.fft.process_with_scratch(complex_buffer, &mut self.scratch_buffer);

        let half_size = self.fft_size / 2;
        for (mag, complex) in magnitudes.iter_mut().zip(complex_buffer[..half_size].iter()) {
            *mag = complex.norm();
        }
    }

    // Méthode recommandée pour l'accordeur : autocorrélation normalisée.
    // Elle marche mieux qu'un simple pic FFT sur une guitare réelle, car le plus gros pic
    // du spectre peut être une harmonique au lieu de la fondamentale.
    pub fn find_frequency_from_samples(&self, samples: &[f32]) -> Option<f32> {
        if samples.len() < 2 {
            return None;
        }

        let level = average_amplitude(samples);
        if level < 0.002 {
            return None;
        }

        let min_freq = 70.0;
        let max_freq = 380.0;

        let min_lag = (self.sample_rate / max_freq).floor() as usize;
        let max_lag = (self.sample_rate / min_freq).ceil() as usize;

        if min_lag < 2 || max_lag + 1 >= samples.len() {
            return None;
        }

        let mut scores = vec![0.0f32; max_lag + 1];
        let mut best_score = 0.0;

        for lag in min_lag..=max_lag {
            let mut sum = 0.0;
            let mut energy_a = 0.0;
            let mut energy_b = 0.0;

            let limit = samples.len() - lag;
            for i in 0..limit {
                let a = samples[i];
                let b = samples[i + lag];
                sum += a * b;
                energy_a += a * a;
                energy_b += b * b;
            }

            if energy_a > 1e-9 && energy_b > 1e-9 {
                let score = sum / (energy_a.sqrt() * energy_b.sqrt());
                scores[lag] = score;
                if score > best_score {
                    best_score = score;
                }
            }
        }

        // Si la corrélation est trop faible, on considère qu'il n'y a pas de note fiable.
        if best_score < 0.35 {
            return None;
        }

        // On prend le premier pic local suffisamment fort.
        // Cela évite souvent de choisir l'octave au-dessus.
        let threshold = best_score * 0.85;
        let mut best_lag = 0;
        for lag in (min_lag + 1)..max_lag {
            if scores[lag] >= threshold
                && scores[lag] >= scores[lag - 1]
                && scores[lag] >= scores[lag + 1]
            {
                best_lag = lag;
                break;
            }
        }

        if best_lag == 0 {
            return None;
        }

        // Interpolation parabolique autour du pic pour améliorer la précision.
        let left = scores[best_lag - 1];
        let center = scores[best_lag];
        let right = scores[best_lag + 1];
        let denominator = left - 2.0 * center + right;

        let correction = if denominator.abs() > 1e-6 {
            0.5 * (left - right) / denominator
        } else {
            0.0
        };

        let exact_lag = best_lag as f32 + correction.clamp(-0.5, 0.5);
        if exact_lag <= 0.0 {
            return None;
        }

        Some(self.sample_rate / exact_lag)
    }

    // Méthode FFT gardée si vous voulez afficher/debugger le spectre.
    pub fn find_precise_frequency(&self, magnitudes: &[f32]) -> Option<f32> {
        let bin_low = (70.0 * self.fft_size as f32 / self.sample_rate).ceil() as usize;
        let bin_high = (380.0 * self.fft_size as f32 / self.sample_rate).ceil() as usize;
        let bin_high = bin_high.min(magnitudes.len().saturating_sub(2));

        if bin_low >= bin_high {
            return None;
        }

        let mut max_idx = bin_low;
        let mut max_mag = 0.0;

        for i in bin_low..=bin_high {
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

        let p = if denominator.abs() > 1e-6 {
            0.5 * (alpha - gamma) / denominator
        } else {
            0.0
        };

        let exact_bin = max_idx as f32 + p;
        Some(exact_bin * self.sample_rate / self.fft_size as f32)
    }

    pub fn hz_to_note(&self, frequency: f32) -> Note {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

        let midi_note = 12.0 * (frequency / 440.0).log2() + 69.0;
        let closest_midi = midi_note.round() as i32;
        let cents = (midi_note - closest_midi as f32) * 100.0;
        let note_index = closest_midi.rem_euclid(12) as usize;
        let octave = (closest_midi / 12) - 1;

        Note {
            name: note_names[note_index].to_string(),
            _octave: octave,
            _cents: cents,
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
