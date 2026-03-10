use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::sync::Arc;
use std::f32::consts::PI;
//use rustfft::num_complex::Complex;


pub struct FrequencyAnalyzer {
    // 1. Le plan de calcul optimisé par rustfft
    pub fft: Arc<dyn Fft<f32>>, 

    // 2. La table de la fenêtre de Hann pré-calculée
    pub window_table: Vec<f32>, 

    // 3. Un buffer réutilisable pour éviter les allocations mémoire en temps réel
    pub scratch_buffer: Vec<Complex<f32>>, 

    // 4. Indispensable pour convertir un indice de tableau en Hz
    pub sample_rate: f32, 

    // 5. La taille de l'échantillon (ex: 2048 ou 4096)
    pub fft_size: usize, 
}

pub struct Note {
    pub name: String, // ex: "E" (Mi)
    pub octave: i32,      // ex: 2 (pour le Mi grave E2)
    pub cents: f32,       // L'écart entre -50.0 et +50.0
}


impl FrequencyAnalyzer {
    // Initialise les tables de sinus/cosinus pour ne pas les refaire en boucle
    pub fn new(sample_rate: f32, fft_size: usize) -> Self {
        // 1. Initialiser le planificateur et créer le plan FFT
        // On lui indique la taille de l'échantillon (fft_size) et qu'on veut une FFT "forward" 
        // (passage du domaine temporel au domaine fréquentiel).
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(fft_size);

        // 2. Pré-calculer la fenêtre de Hann
        // La formule mathématique est appliquée pour chaque point de 0 à fft_size - 1.
        let window_table: Vec<f32> = (0..fft_size)
            .map(|i| {
                let n = i as f32;
                let n_total = (fft_size - 1) as f32;
                0.5 * (1.0 - (2.0 * PI * n / n_total).cos())
            })
        .collect();

        // 3. Préparer le scratch_buffer
        // La magie de rustfft : la bibliothèque sait exactement de combien de mémoire 
        // temporaire elle aura besoin pour calculer la FFT sans rien allouer à la volée.
        let scratch_size = fft.get_inplace_scratch_len();
        let scratch_buffer = vec![Complex { re: 0.0, im: 0.0 }; scratch_size];

        // 4. Retourner l'instance prête pour le temps réel
        Self {
            fft,
            window_table,
            scratch_buffer,
            sample_rate,
            fft_size,
        }
    }

    pub fn apply_window(&self, audio_buffer: &mut [f32]) {
        assert_eq!(audio_buffer.len(), self.window_table.len(), //check if audio_buffer size match with window_table.len
            "La taille du buffer audio ({}) ne correspond pas à la taille de la fenêtre ({})",
            audio_buffer.len(), self.window_table.len());

        for (sample, window_value) in audio_buffer.iter_mut().zip(&self.window_table) { // iter  throughout the buffer 
                                                                                        // and the table at the same time  
            *sample *= window_value;
        }
    }

    pub fn compute_fft_magnitude(&mut self, windowed_audio: &[f32], complex_buffer: &mut [Complex<f32>], magnitudes: &mut [f32]) {
        // 1. Conversion des f32 en nombres complexes
        // La FFT a besoin d'une partie réelle (le signal) et imaginaire (0.0 ici)
        for (complex, &sample) in complex_buffer.iter_mut().zip(windowed_audio.iter()) {
            complex.re = sample;
            complex.im = 0.0;
        }

        // 2. Exécution de la FFT "in-place"
        // Le complex_buffer est écrasé par le résultat de la FFT.
        // Le scratch_buffer sert d'espace de travail temporaire pour l'algorithme.
        self.fft.process_with_scratch(complex_buffer, &mut self.scratch_buffer);

        // 3. Calcul de la magnitude (l'amplitude de chaque fréquence)
        // On ne boucle que sur la première moitié. La seconde moitié d'une FFT réelle 
        // est juste un miroir symétrique (Théorème de Nyquist-Shannon), donc inutile.
        let half_size = self.fft_size / 2;

        for (mag, complex) in magnitudes.iter_mut().zip(complex_buffer[..half_size].iter()) {
            // La magnitude est la norme du nombre complexe: sqrt(re^2 + im^2)
            // La méthode .norm() de la crate num_complex fait exactement ça.
            *mag = complex.norm();
        }
    }

    /// Trouve la fréquence fondamentale exacte en utilisant l'interpolation parabolique.
    pub fn find_precise_frequency(&self, magnitudes: &[f32]) -> Option<f32> {
        let mut max_idx = 0;
        let mut max_mag = 0.0;

        // 1. Trouver l'indice du bin avec la plus forte magnitude
        for (i, &mag) in magnitudes.iter().enumerate() {
            if mag > max_mag {
                max_mag = mag;
                max_idx = i;
            }
        }

        // Si le volume est trop faible (silence), on ne retourne rien
        // Note: 1.0 est une valeur arbitraire à ajuster selon le gain de ton micro
        if max_mag < 1.0 {
            return None;
        }

        // 2. Vérifier qu'on n'est pas aux bords du tableau (impossible d'interpoler)
        if max_idx == 0 || max_idx >= magnitudes.len() - 1 {
            // On retourne la fréquence brute du bin
            let bin_freq = max_idx as f32 * self.sample_rate / self.fft_size as f32;
            return Some(bin_freq);
        }

        // 3. Récupérer les valeurs pour l'interpolation parabolique
        let alpha = magnitudes[max_idx - 1]; // Voisin de gauche
        let beta = magnitudes[max_idx];      // Sommet local
        let gamma = magnitudes[max_idx + 1]; // Voisin de droite

        // 4. Calculer le décalage (offset 'p')
        let denominator = alpha - 2.0 * beta + gamma;

        let p = if denominator.abs() > 1e-6 {
            0.5 * (alpha - gamma) / denominator
        } else {
            0.0 // Sécurité pour éviter une division par zéro
        };

        // 5. Calculer le "vrai" bin exact (qui est un nombre à virgule)
        let exact_bin = max_idx as f32 + p;

        // 6. Convertir le bin exact en Hertz
        let frequency = exact_bin * self.sample_rate / self.fft_size as f32;

        Some(frequency)
    }

    pub fn hz_to_note(&self, frequency: f32) -> Note {
        // 1. Définir les noms des notes dans une octave
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

        // 2. Calculer le nombre de demi-tons par rapport au La 440Hz (Note MIDI 69)
        // On utilise log2 pour l'échelle musicale
        let midi_note = 12.0 * (frequency / 440.0).log2() + 69.0;

        // 3. Arrondir à la note la plus proche
        let closest_midi = midi_note.round() as i32;

        // 4. Calculer l'erreur en "cents" (100 cents = 1 demi-ton)
        // Un accordage est parfait à 0.0 cents.
        let cents = (midi_note - closest_midi as f32) * 100.0;

        // 5. Extraire le nom de la note et l'octave
        // En MIDI, l'octave 0 commence à la note 12
        let note_index = (closest_midi % 12) as usize;
        let octave = (closest_midi / 12) - 1;

        Note {
            name: note_names[note_index].to_string(),
            octave,
            cents,
        }
    }


    fn print_tuner_status(res: Note) {
    let indicator = if res.cents.abs() < 5.0 {
        "✅ ACCORDÉ"
    } else if res.cents > 0.0 {
        "Too High ⬆️"
    } else {
        "Too Low  ⬇️"
    };

    println!(
        "Note: {}{} | Erreur: {:+5.1} cents | {}",
        res.name, res.octave, res.cents, indicator
    );
    }

    // Ta fonction "maître" appelée à chaque nouveau buffer audio
    pub fn detect_pitch(&self, audio_data: &[f32])/* -> Option<Note>*/ {
        let sample_rate = 44100.0;
        let fft_size = 2048;

        // 2. Initialisation de ton analyseur (une seule fois !)
        let mut analyzer = FrequencyAnalyzer::new(sample_rate, fft_size);

        // 3. Préparation des buffers de travail (pour éviter les allocations dans la boucle)
        let mut complex_buffer = vec![Complex { re: 0.0, im: 0.0 }; fft_size];
        let mut magnitudes = vec![0.0; fft_size / 2];

        println!("Accordeur prêt ! Jouez une note...");

        // --- ICI COMMENCE TA BOUCLE TEMPS RÉEL (ex: callback CPAL ou Rodio) ---
        // Supposons que 'audio_in' est le bloc de f32 reçu du micro
        /*
        loop {
            let mut audio_in = capture_audio_buffer(fft_size); // Fonction fictive représentant ton micro

            // ÉTAPE A : Pré-traitement (Windowing)
            analyzer.apply_window(&mut audio_in);

            // ÉTAPE B : Calcul du spectre (FFT)
            // On passe les buffers réutilisables pour ne rien allouer
            analyzer.compute_fft_magnitude(&audio_in, &mut complex_buffer, &mut magnitudes);

            // ÉTAPE C : Détection de la fréquence précise
            if let Some(freq) = analyzer.find_precise_frequency(&magnitudes) {

                // ÉTAPE D : Traduction musicale
                let result = analyzer.hz_to_note(freq);

                // ÉTAPE E : Affichage (on pourrait ici envoyer ça à une interface graphique)
                print_tuner_status(result);
            }
        }
        */
    }
}
