mod scanner;
mod app;

use eframe::egui;
use crate::app::MyApp;

fn main() {
    let native_options = eframe::NativeOptions {
      viewport: egui::ViewportBuilder::default()
          .with_inner_size([900.0, 650.0])
          .with_min_inner_size([700.0, 480.0]),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Directory Footprint",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc))))
    ).expect("Failed to start eframe");
}


