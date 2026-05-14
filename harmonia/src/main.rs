mod gui;
use gui::app::HarmoniaApp;

mod analyser;
use analyser::FrequencyAnalyzer;
mod input;
use input::{AudioConfig, AudioInput};
use rustfft::num_complex::Complex;
mod notes;
mod comparaison;

use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> eframe::Result<()> {
    let sample_rate = 44100.0;
    let fft_size = 4096;

    // Fréquence partagée entre le thread audio et le GUI
    let shared_freq: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
    let shared_freq_audio = Arc::clone(&shared_freq);

    // Thread audio : capture + FFT en arrière-plan
    thread::spawn(move || {
        let mut analyzer = FrequencyAnalyzer::new(sample_rate, fft_size);
        let mut complex_buffer = vec![Complex { re: 0.0, im: 0.0 }; fft_size];
        let mut magnitudes = vec![0.0f32; fft_size / 2];

        let audio = match AudioInput::start(AudioConfig::default()) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Erreur audio : {}", e);
                return;
            }
        };

        println!("Capture audio démarrée");

        // EMA : lissage exponentiel de la fréquence détectée
        // alpha proche de 0 = très lisse mais lent ; proche de 1 = réactif mais instable
        let alpha: f32 = 0.2;
        let mut smoothed_freq: Option<f32> = None;

        loop {
            let chunk = match audio.recv() {
                Ok(c) => c,
                Err(_) => break,
            };

            let mut audio_in = chunk.samples;
            analyzer.apply_window(&mut audio_in);
            analyzer.compute_fft_magnitude(&audio_in, &mut complex_buffer, &mut magnitudes);

            // Plage guitare standard : E2 (82 Hz) → E4 (330 Hz) + marges
            let raw_freq = analyzer.find_precise_frequency(&magnitudes, 60.0, 420.0);

            // Applique l'EMA : si un son est détecté, on lisse ; sinon on réinitialise
            smoothed_freq = match (raw_freq, smoothed_freq) {
                (Some(new), Some(prev)) => Some(alpha * new + (1.0 - alpha) * prev),
                (Some(new), None)       => Some(new), // première détection : pas de lissage
                (None, _)               => None,       // silence : on remet à zéro
            };

            if let Some(f) = smoothed_freq {
                println!("{:.2}Hz", f);
            }

            if let Ok(mut lock) = shared_freq_audio.lock() {
                *lock = smoothed_freq;
            }
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 480.0])
            .with_min_inner_size([860.0, 480.0])
            .with_max_inner_size([860.0, 480.0])
            .with_title("TrueTone")
            .with_resizable(false)
            .with_fullscreen(false),
        ..Default::default()
    };

    eframe::run_native(
        "TrueTone",
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(HarmoniaApp::new(shared_freq)) as Box<dyn eframe::App>)
        }),
    )
}
