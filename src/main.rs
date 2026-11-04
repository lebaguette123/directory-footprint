use eframe::egui;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Directory Footprint",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc))))
    ).expect("TODO: panic message");
}

#[derive(Default)]
struct MyApp{}
impl MyApp{
    fn new(cc: &eframe::CreationContext<'_>) -> Self{
        Self::default()
    }
}

impl eframe::App for MyApp{
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame){
        egui::CentralPanel::default().show(ui, |panel_ui|{
            panel_ui.heading("Hello World!");
        });
    }
}
