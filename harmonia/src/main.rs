mod analyser;
use analyser::*;

fn main() {
    let mut analyzer = FrequencyAnalyzer::new(44100.0, 2048);
    println!("Analyseur compilé avec succès !");
    // C'est ici que tu injecteras ton flux audio venant du micro.
}
