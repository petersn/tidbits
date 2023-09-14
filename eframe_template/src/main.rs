use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Template",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

struct App;

impl App {
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self {}
    }
}

impl eframe::App for App {
    fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(_ctx, |ui| {
            ui.label("Hello, World!");
        });
    }
}
