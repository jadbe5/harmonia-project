/*mod gui;
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
}*/



mod notes;
mod comparaison;

use notes::GUITAR;

fn main()
{
    println!("Liste des cordes de guitare :");
    for s in GUITAR.iter()
    {
        println!("{} -> {} Hz", s.name, s.frequency);
    }

    let freq = 111.0;
    let result = comparaison::compare_frequency(freq);
    println!("{}", result);
    let freq2 = 82.0;
    let result = comparaison::compare_frequency(freq2);
    println!("{}", result);
}
