use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::sync::Arc;
use std::f32::consts::PI;
//use rustfft::num_complex::Complex;


pub struct FrequencyAnalyzer {
    // 1. Le plan de calcul optimisé par rustfft
    pub fft: Arc<dyn Fft<f32>>, 

    pub window_table: Vec<f32>, // Hann windows

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
        let mut planner = FftPlanner::<f32>::new(); // found best algorithme
        let fft = planner.plan_fft_forward(fft_size); // create sin and cos tables

        let window_table: Vec<f32> = (0..fft_size)  // initialisation of Hann Window
            .map(|i| {
                let n = i as f32;
                let n_total = (fft_size - 1) as f32;
                0.5 * (1.0 - (2.0 * PI * n / n_total).cos())
            })
        .collect();

        let scratch_size = fft.get_inplace_scratch_len();  //buffer for intermediate calcul
        let scratch_buffer = vec![Complex { re: 0.0, im: 0.0 }; scratch_size];

        Self {
            fft,
            window_table,
            scratch_buffer,
            sample_rate,
            fft_size,
        }
    }

    // use Hann window to avoid spectral leakage
    pub fn apply_window(&self, audio_buffer: &mut [f32]) {
        assert_eq!(audio_buffer.len(), self.window_table.len(), //check if audio_buffer size match with window_table.len
            "Audio buffer size ({}) doesn't match with window_table size({}).",
            audio_buffer.len(), self.window_table.len());

        for (sample, window_value) in audio_buffer.iter_mut().zip(&self.window_table) {  
            *sample *= window_value;
        }
    }

    pub fn compute_fft_magnitude(&mut self, windowed_audio: &[f32], complex_buffer: &mut [Complex<f32>], magnitudes: &mut [f32]) {
        // Convert f32 to COmplex number
        for (complex, &sample) in complex_buffer.iter_mut().zip(windowed_audio.iter()) {
            complex.re = sample;
            complex.im = 0.0;
        }

        self.fft.process_with_scratch(complex_buffer, &mut self.scratch_buffer);  // execution of FFT 

        let half_size = self.fft_size / 2;  

        for (mag, complex) in magnitudes.iter_mut().zip(complex_buffer[..half_size].iter()) {  //calculates the amplitude of each frequency

            *mag = complex.norm();
        }
    }

    // found the fundamental frequency using parabolic interpolation
    pub fn find_precise_frequency(&self, magnitudes: &[f32]) -> Option<f32> {
        let mut max_idx = 0;
        let mut max_mag = 0.0;

        for (i, &mag) in magnitudes.iter().enumerate() { // found the case that contains the highter
                                                         // frequency
            if mag > max_mag {
                max_mag = mag;
                max_idx = i;
            }
        }

        if max_mag < 1.0 {  // check if there is a sound
            return None;
        }

        if max_idx == 0 || max_idx >= magnitudes.len() - 1 {  // check if it's the edge of the array
            let bin_freq = max_idx as f32 * self.sample_rate / self.fft_size as f32;
            return Some(bin_freq);  // don't do the interpolation
        }

        let alpha = magnitudes[max_idx - 1]; // left element
        let beta = magnitudes[max_idx];      // best element
        let gamma = magnitudes[max_idx + 1]; // right element 

        let denominator = alpha - 2.0 * beta + gamma;

        let p = if denominator.abs() > 1e-6 {    // calculate the gap to have a 0.1Hz precision

            0.5 * (alpha - gamma) / denominator
        } else {
            0.0 // avoid division by zero
        };

        let exact_bin = max_idx as f32 + p;

        let frequency = exact_bin * self.sample_rate / self.fft_size as f32;  //convertion to Hz

        Some(frequency)
    }

    pub fn hz_to_note(&self, frequency: f32) -> Note {
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

        let midi_note = 12.0 * (frequency / 440.0).log2() + 69.0; // nb of semintones between La
                                                                  // 440Hz and our frequency

        let closest_midi = midi_note.round() as i32;

        let _cents = (midi_note - closest_midi as f32) * 100.0; //calculate error in cents

        let note_index = closest_midi.rem_euclid(12) as usize;  //found the name of the note
        let _octave = (closest_midi / 12) - 1;

        Note {
            name: note_names[note_index].to_string(),
            _octave,
            _cents,
        }
    }
}
