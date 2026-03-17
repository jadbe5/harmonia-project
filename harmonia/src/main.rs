mod analyser;
use analyser::*;
mod input;
use input::{AudioConfig, AudioInput};
use rustfft::num_complex::Complex;



fn test() -> Result<(), Box<dyn std::error::Error>>
{
    let audio = AudioInput::start(AudioConfig::default())?;

    println!("capture audio demarree");

    loop
    {
        let chunk = audio.recv()?;

        println!(
            "chunk recu | samples={} | sample_rate={} | niveau={:.5} | seuil={:.5}",
            chunk.samples.len(),
            chunk.sample_rate,
            chunk.level,
            chunk.threshold
        );
    }

    let mut analyzer = FrequencyAnalyzer::new(44100.0, 2048);
    println!("Analyseur compilé avec succès !");
    // C'est ici que tu injecteras ton flux audio venant du micro.
}




fn main() -> Result<(), Box<dyn std::error::Error>>
{
    //test();
    let sample_rate = 44100.0;
    let fft_size = 4096;

    // 2. Initialisation de ton analyseur (une seule fois !)
    let mut analyzer = FrequencyAnalyzer::new(sample_rate, fft_size);

    // 3. Préparation des buffers de travail (pour éviter les allocations dans la boucle)
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

