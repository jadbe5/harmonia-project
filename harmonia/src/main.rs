
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Harmonia",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(HarmoniaApp::default())),
    )
}
