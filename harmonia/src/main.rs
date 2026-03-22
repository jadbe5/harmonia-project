mod gui;
use gui::app::HarmoniaApp;

fn main() -> eframe::Result<()> {
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
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(HarmoniaApp::default()) as Box<dyn eframe::App>)
        }),
    )

}

/*
mod analyser;
use analyser::*;
mod input;
use input::{AudioConfig, AudioInput};
use rustfft::num_complex::Complex;


fn main() -> Result<(), Box<dyn std::error::Error>>
{

    let sample_rate = 44100.0;
    let fft_size = 4096;


    let mut analyzer = FrequencyAnalyzer::new(sample_rate, fft_size);


    let mut complex_buffer = vec![Complex { re: 0.0, im: 0.0 }; fft_size];
    let mut magnitudes = vec![0.0; fft_size / 2];

    println!("Accordeur prêt ! Jouez une note...");

    let audio = AudioInput::start(AudioConfig::default())?;

    println!("capture audio demarree");

    loop {

        let chunk = audio.recv()?;

        let mut audio_in = chunk.samples;

        analyzer.apply_window(&mut audio_in);

        analyzer.compute_fft_magnitude(&audio_in, &mut complex_buffer, &mut magnitudes);

        if let Some(freq) = analyzer.find_precise_frequency(&magnitudes) {

            let result = analyzer.hz_to_note(freq);
            println!("result = {}", result.name);
        }
    }
}
*/
