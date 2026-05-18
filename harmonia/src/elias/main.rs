mod gui;
use gui::app::HarmoniaApp;

mod analyser;
use analyser::FrequencyAnalyzer;
mod input;
use input::{AudioConfig, AudioInput};
mod notes;
mod comparaison;

use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> eframe::Result<()> {
    let fft_size = 4096;

    // Fréquence partagée entre le thread audio et le GUI
    let shared_freq: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
    let shared_freq_audio = Arc::clone(&shared_freq);

    // Thread audio : capture + analyse en arrière-plan
    thread::spawn(move || {
        let audio = match AudioInput::start(AudioConfig::default()) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Erreur audio : {}", e);
                return;
            }
        };

        println!("Capture audio démarrée");

        let alpha: f32 = 0.25;
        let mut smoothed_freq: Option<f32> = None;
        let mut analyzer: Option<FrequencyAnalyzer> = None;
        let mut current_sample_rate: u32 = 0;

        loop {
            let chunk = match audio.recv() {
                Ok(c) => c,
                Err(_) => break,
            };

            // Très important : on utilise le vrai sample rate fourni par le micro.
            // Si le micro est en 48000 Hz mais qu'on force 44100 Hz, les fréquences sont fausses.
            if analyzer.is_none() || current_sample_rate != chunk.sample_rate {
                current_sample_rate = chunk.sample_rate;
                analyzer = Some(FrequencyAnalyzer::new(chunk.sample_rate as f32, fft_size));
                println!("Sample rate audio utilisé : {} Hz", chunk.sample_rate);
            }

            let analyzer = match analyzer.as_ref() {
                Some(a) => a,
                None => continue,
            };

            let raw_freq = analyzer.find_frequency_from_samples(&chunk.samples);

            smoothed_freq = match (raw_freq, smoothed_freq) {
                (Some(new), Some(prev)) => Some(alpha * new + (1.0 - alpha) * prev),
                (Some(new), None) => Some(new),
                (None, _) => None,
            };

            if let Some(f) = smoothed_freq {
                println!("{:.2} Hz", f);
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
