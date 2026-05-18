mod gui;
use gui::app::HarmoniaApp;

mod analyser;
use analyser::FrequencyAnalyzer;
mod input;
use input::{AudioConfig, AudioInput};
mod notes;
mod comparaison;

use comparaison::{analyze_frequency, compare_frequency};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> eframe::Result<()> {
    let fft_size = 4096;

    let shared_freq: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
    let shared_freq_audio = Arc::clone(&shared_freq);

    thread::spawn(move || {
        let audio = match AudioInput::start(AudioConfig::default()) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Erreur audio : {}", e);
                return;
            }
        };

        println!("Capture audio démarrée");

        let mut analyzer: Option<FrequencyAnalyzer> = None;
        let mut analyzer_sample_rate: u32 = 0;

        let alpha: f32 = 0.35;
        let mut smoothed_freq: Option<f32> = None;

        loop {
            let chunk = match audio.recv() {
                Ok(c) => c,
                Err(_) => break,
            };

            if analyzer.is_none() || analyzer_sample_rate != chunk.sample_rate {
                analyzer = Some(FrequencyAnalyzer::new(chunk.sample_rate as f32, fft_size));
                analyzer_sample_rate = chunk.sample_rate;
                println!("Sample rate audio utilisé : {} Hz", chunk.sample_rate);
            }

            let analyzer = match analyzer.as_ref() {
                Some(a) => a,
                None => continue,
            };

            let raw_freq = if chunk.level >= chunk.threshold {
                analyzer.find_frequency(&chunk.samples)
            } else {
                None
            };

            smoothed_freq = match (raw_freq, smoothed_freq) {
                (Some(new), Some(prev)) => {
                    if new > prev * 1.8 || new < prev * 0.55 {
                        Some(new)
                    } else {
                        Some(alpha * new + (1.0 - alpha) * prev)
                    }
                }
                (Some(new), None) => Some(new),
                (None, _) => None,
            };

            if let Some(f) = smoothed_freq {
                if let Some(result) = analyze_frequency(f) {
                    println!(
                        "{} | cible {:.2} Hz | détecté {:.2} Hz | écart {:+.1} cents",
                        compare_frequency(f),
                        result.target_frequency,
                        result.detected_frequency,
                        result.cents
                    );
                }
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
