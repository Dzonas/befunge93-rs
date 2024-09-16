fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Befunge93-rs",
        native_options,
        Box::new(|cc| Ok(Box::new(Befunge93App::new(cc)))),
    )
}

#[derive(Default)]
struct Befunge93App {}

impl Befunge93App {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for Befunge93App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}
