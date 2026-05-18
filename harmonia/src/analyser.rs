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
    /*
    pub fn find_precise_frequency(&self, magnitudes: &[f32]) -> Option<f32> {
        let bin_low  = (70.0  * self.fft_size as f32 / self.sample_rate).ceil() as usize;
        let bin_high = (380.0 * self.fft_size as f32 / self.sample_rate).ceil() as usize;
        let bin_high = bin_high.min(magnitudes.len() - 2);

        let mut max_idx = bin_low;
        let mut max_mag = 0.0f32;

        for (i, &mag) in magnitudes[bin_low..bin_high].iter().enumerate() {
            if mag > max_mag {
                max_mag = mag;
                max_idx = i + bin_low;
            }
        }

        if max_mag < 0.001 {
            return None;
        }

        if max_idx == 0 || max_idx >= magnitudes.len() - 1 {
            let bin_freq = max_idx as f32 * self.sample_rate / self.fft_size as f32;
            return Some(bin_freq);
        }

        let alpha = magnitudes[max_idx - 1];
        let beta  = magnitudes[max_idx];
        let gamma = magnitudes[max_idx + 1];

        let denominator = alpha - 2.0 * beta + gamma;

        let p = if denominator.abs() > 1e-6 {
            0.5 * (alpha - gamma) / denominator
        } else {
            0.0
        };

        let exact_bin = max_idx as f32 + p;
        let frequency = exact_bin * self.sample_rate / self.fft_size as f32;

        Some(frequency)
    }
        */

        /*
        pub fn find_precise_frequency(&self, magnitudes: &[f32]) -> Option<f32> {
        // Bornes de recherche pour la guitare (70 Hz à 380 Hz)
        let bin_low  = (70.0  * self.fft_size as f32 / self.sample_rate).ceil() as usize;
        let bin_high = (380.0 * self.fft_size as f32 / self.sample_rate).ceil() as usize;

        // Nombre d'harmoniques à analyser (3 ou 4 est idéal pour la guitare)
        let num_harmonics = 4;

        // Sécurité : on s'assure que i * num_harmonics ne dépasse pas la taille du spectre
        // (Pour FFT 4096, magnitudes fait 2048 de long, donc aucun risque avec 380 Hz)
        let bin_high = bin_high.min((magnitudes.len() - 2) / num_harmonics);

        if bin_low >= bin_high {
            return None;
        }

        // 1. Calcul du spectre HPS
        let mut hps_spectrum = vec![0.0f32; bin_high + 1];
        let mut max_hps_value = 0.0f32;
        let mut max_idx = bin_low;

        for i in bin_low..=bin_high {
            let mut product = magnitudes[i];

            // Multiplier l'amplitude de la note par celle de ses harmoniques
            for r in 2..=num_harmonics {
                product *= magnitudes[i * r];
            }

            hps_spectrum[i] = product;

            // On repère le pic maximum dans le spectre HPS
            if product > max_hps_value {
                max_hps_value = product;
                max_idx = i;
            }
        }

        // Seuil de sécurité : si l'amplitude sur le spectre original est trop faible,
        // c'est du bruit de fond ou du silence.
        if magnitudes[max_idx] < 0.005 {
            return None;
        }

        // 2. Interpolation parabolique
        // TRÈS IMPORTANT : On applique l'interpolation sur le spectre ORIGINAL (magnitudes)
        // autour de l'index trouvé par le HPS. Le spectre HPS déforme la courbe des pics,
        // l'interpolation y serait donc faussée.
        if max_idx == 0 || max_idx >= magnitudes.len() - 1 {
            let bin_freq = max_idx as f32 * self.sample_rate / self.fft_size as f32;
            return Some(bin_freq);
        }

        let alpha = magnitudes[max_idx - 1];
        let beta  = magnitudes[max_idx];
        let gamma = magnitudes[max_idx + 1];

        let denominator = alpha - 2.0 * beta + gamma;

        let p = if denominator.abs() > 1e-6 {
            0.5 * (alpha - gamma) / denominator
        } else {
            0.0
        };

        // Calcul de la fréquence finale ultra-précise
        let exact_bin = max_idx as f32 + p;
        let frequency = exact_bin * self.sample_rate / self.fft_size as f32;

        Some(frequency)
    }

*/

pub fn find_precise_frequency(&self, magnitudes: &[f32]) -> Option<f32> {
        // Bornes de recherche de la guitare : ~70 Hz à ~380 Hz
        let bin_low  = (70.0  * self.fft_size as f32 / self.sample_rate).ceil() as usize;
        let bin_high = (380.0 * self.fft_size as f32 / self.sample_rate).ceil() as usize;
        
        // On analyse jusqu'à 4 harmoniques
        let num_harmonics = 4;
        let hss_bin_high = bin_high.min((magnitudes.len() - 1) / num_harmonics);

        if bin_low >= hss_bin_high {
            return None;
        }

        // 1. SEUIL DE SÉCURITÉ GLOBAL (Vérifie si le micro capte du son)
        let mut max_raw_mag = 0.0f32;
        for &mag in &magnitudes[bin_low..bin_high.min(magnitudes.len())] {
            if mag > max_raw_mag {
                max_raw_mag = mag;
            }
        }
        
        // Si aucun son significatif n'est détecté, on retourne None (silence)
        if max_raw_mag < 0.001 { 
            return None;
        }

        // 2. ALGORITHME HSS "FLOU" (Fuzzy Harmonic Sum Spectrum)
        let mut hss_values = vec![0.0f32; hss_bin_high + 1];
        let mut max_hss_value = 0.0f32;

        // Fonction locale (closure) pour tolérer un léger décalage de bin
        let get_mag_around = |bin: usize| -> f32 {
            let mut max_m = magnitudes[bin];
            if bin > 0 && magnitudes[bin - 1] > max_m { 
                max_m = magnitudes[bin - 1]; 
            }
            if bin + 1 < magnitudes.len() && magnitudes[bin + 1] > max_m { 
                max_m = magnitudes[bin + 1]; 
            }
            max_m
        };

        for i in bin_low..=hss_bin_high {
            // Addition des harmoniques avec tolérance de voisinage
            let sum = get_mag_around(i) 
                    + get_mag_around(i * 2) 
                    + get_mag_around(i * 3) 
                    + get_mag_around(i * 4);
            
            hss_values[i] = sum;
            if sum > max_hss_value {
                max_hss_value = sum;
            }
        }

        // 3. SÉLECTION DU PREMIER PIC SIGNIFICATIF (Anti-octave)
        // On cherche la fondamentale la plus basse qui a au moins 50% de l'énergie maximale
        let mut fundamental_idx = bin_low;
        let threshold = max_hss_value * 0.50; 
        let mut found = false;

        for i in bin_low..=hss_bin_high {
            if hss_values[i] >= threshold {
                // On s'assure que c'est un "sommet" (pic local) et pas juste une pente
                let is_local_max = (i == bin_low || hss_values[i] >= hss_values[i - 1]) 
                                && (i == hss_bin_high || hss_values[i] >= hss_values[i + 1]);
                
                if is_local_max {
                    fundamental_idx = i;
                    found = true;
                    break; // On s'arrête au premier pic trouvé (la fondamentale) !
                }
            }
        }

        if !found {
            return None;
        }

        // 4. INTERPOLATION PARABOLIQUE
        // On retourne sur le spectre original "magnitudes" pour affiner la précision
        if fundamental_idx == 0 || fundamental_idx >= magnitudes.len() - 1 {
            let bin_freq = fundamental_idx as f32 * self.sample_rate / self.fft_size as f32;
            return Some(bin_freq);
        }

        let alpha = magnitudes[fundamental_idx - 1];
        let beta  = magnitudes[fundamental_idx];
        let gamma = magnitudes[fundamental_idx + 1];

        let denominator = alpha - 2.0 * beta + gamma;

        let p = if denominator.abs() > 1e-6 {
            0.5 * (alpha - gamma) / denominator
        } else {
            0.0
        };

        // Calcul final de la fréquence exacte
        let exact_bin = fundamental_idx as f32 + p;
        let frequency = exact_bin * self.sample_rate / self.fft_size as f32;

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

