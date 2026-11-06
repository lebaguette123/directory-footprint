use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::scanner::{scan_directory, ScanResult};

#[derive(Default)]
pub struct MyApp{
    current_dir: Option<PathBuf>,
    scan_result: Option<ScanResult>,
    scan_error: Option<String>,
    is_scanning: bool,
    scan_receiver: Option<Receiver<Result<ScanResult, std::io::Error>>>,
}
impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self{
        let mut visuals = egui::Visuals::dark();

        visuals.panel_fill =
            egui::Color32::from_rgb(15, 20, 28);

        visuals.faint_bg_color =
            egui::Color32::from_rgb(29, 42, 58);

        visuals.override_text_color = Some(
            egui::Color32::from_rgb(227, 234, 242)
        );

        visuals.weak_text_color = Some(
            egui::Color32::from_rgb(162, 175, 191)
        );

        visuals.hyperlink_color = egui::Color32::from_rgb(125, 211, 252);

        let button_fill = egui::Color32::from_rgb(27, 42, 59);
        let button_hover = egui::Color32::from_rgb(38, 60, 82);
        let button_pressed = egui::Color32::from_rgb(31, 52, 71);

        let button_border = egui::Color32::from_rgb(51, 65, 85);
        let button_text = egui::Color32::from_rgb(226, 232, 240);
        let accent = egui::Color32::from_rgb(125, 211, 252);

        visuals.widgets.inactive.weak_bg_fill = button_fill;
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, button_border);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0, button_text);

        visuals.widgets.hovered.weak_bg_fill = button_hover;
        visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.0, accent);

        visuals.widgets.active.weak_bg_fill = button_pressed;
        visuals.widgets.active.bg_stroke =
            egui::Stroke::new(1.0, accent);

        cc.egui_ctx.set_theme(egui::Theme::Dark);
        cc.egui_ctx.set_visuals(visuals);

        Self::default()
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame){
        if let Some(receiver) = self.scan_receiver.take(){
            match receiver.try_recv(){
                Ok(result) => {
                    self.is_scanning = false;

                    match result{
                        Ok(scan_result ) => {
                            self.scan_result = Some(scan_result);
                            self.scan_error = None;
                        },
                        Err(e) => {
                            self.scan_result = None;
                            self.scan_error = Some(format!("Error scanning directory: {}", e));
                        }
                    }
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.scan_receiver = Some(receiver);
                },
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_scanning = false;
                    self.scan_error = Some("Scan thread disconnected unexpectedly".to_string());
                }

            }
        }

        if self.is_scanning {
            ui.ctx().request_repaint_after(
                std::time::Duration::from_millis(100)
            );
        }

        egui::CentralPanel::default().show(ui, |panel_ui|{
            panel_ui.heading("Directory Footprint");
            panel_ui.separator();

            egui::Frame::new()
                .fill(egui::Color32::from_rgb(24, 35, 55))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(51, 65, 85)
                ))
                .inner_margin(12.0)
                .show(panel_ui, |section_ui| {
                    section_ui.take_available_width();
                    section_ui.label(
                        egui::RichText::new("TARGET DIRECTORY")
                            .size(11.0)
                            .strong()
                            .color(egui::Color32::from_rgb(125,211,252))
                    );
                    section_ui.add_space(6.0);

                    let row_size = egui::vec2(
                        section_ui.available_width(),
                        40.0
                    );

                    section_ui.allocate_ui_with_layout(
                        row_size,
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui|{
                            let scan_enabled = self.current_dir.is_some() && !self.is_scanning;

                            if ui.add_enabled(scan_enabled, egui::Button::new("Scan")).clicked(){
                                if let Some(dir) = self.current_dir.clone(){
                                    let (sender, receiver) = channel();

                                    let ctx = ui.ctx().clone();

                                    self.scan_receiver = Some(receiver);
                                    self.is_scanning = true;

                                    std::thread::spawn(move ||{
                                        let result = scan_directory(&dir);
                                        let _ = sender.send(result);
                                        ctx.request_repaint();
                                    });
                                }
                            }

                            if ui.add_enabled(!self.is_scanning, egui::Button::new("Browse")).clicked(){
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    if self.current_dir.as_ref() != Some(&path) {
                                        self.scan_result = None;
                                        self.scan_error = None;
                                    }

                                    self.current_dir = Some(path);
                                }
                            }

                            let path_width = ui.available_width();

                            ui.allocate_ui_with_layout(
                                egui::vec2(path_width, 34.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |path_ui|{
                                    egui::Frame::new()
                                        .fill(egui::Color32::from_rgb(10, 16, 27))
                                        .inner_margin(8.0)
                                        .show(path_ui, |frame_ui| {
                                            frame_ui.take_available_width();

                                            if let Some(dir) = &self.current_dir {
                                                frame_ui.add(
                                                    egui::Label::new(dir.display().to_string()).truncate()
                                                );
                                            } else {
                                                frame_ui.label(
                                                    egui::RichText::new("No directory selected")
                                                        .color(egui::Color32::from_rgb(148, 163, 184))
                                                );
                                            }
                                        });
                                },
                            );
                        },
                    );
                });

            if self.is_scanning {
                panel_ui.add_space(8.0);
                panel_ui.horizontal(|ui| {
                    ui.add(
                        egui::Spinner::new()
                            .size(16.0)
                            .color(egui::Color32::from_rgb(125, 211, 252))
                    );

                    ui.label(
                        egui::RichText::new("Scanning directory...")
                            .color(egui::Color32::from_rgb(148, 163, 184))
                    );
                });
            }

            if let Some(error) = &self.scan_error{
                panel_ui.colored_label(egui::Color32::RED, error);
            }

            panel_ui.add_space(12.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false]).show(panel_ui, |content_ui|{
                if let Some(result) = &self.scan_result{
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(24, 35, 55))
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgb(51, 65, 85)
                        ))
                        .inner_margin(12.0)
                        .show(content_ui, |section_ui| {
                            section_ui.take_available_width();
                            section_ui.heading(
                                egui::RichText::new("Scan Results")
                                    .color(egui::Color32::from_rgb(125, 211, 252))
                            );
                            section_ui.add_space(8.0);
                            section_ui.columns(3, |columns| {
                                columns[0].vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new(format_size(result.total_size()))
                                            .size(24.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(226, 232, 240))
                                    );

                                    ui.label(
                                        egui::RichText::new("TOTAL STORAGE")
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(148, 163, 184))
                                    );
                                });

                                columns[1].vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new(result.file_count().to_string())
                                            .size(24.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(226, 232, 240))
                                    );

                                    ui.label(
                                        egui::RichText::new("FILES")
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(148, 163, 184))
                                    );
                                });

                                columns[2].vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new(result.dir_count().to_string())
                                            .size(24.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(226, 232, 240))
                                    );

                                    ui.label(
                                        egui::RichText::new("DIRECTORIES")
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(148, 163, 184))
                                    );
                                });
                            });
                        });

                    content_ui.add_space(12.0);

                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(24, 35, 55))
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgb(51, 65, 85)
                        ))
                        .inner_margin(12.0)
                        .show(content_ui, |section_ui| {
                            section_ui.take_available_width();
                            section_ui.heading(
                                egui::RichText::new("Largest Files")
                                    .color(egui::Color32::from_rgb(125, 211, 252))
                            );

                            section_ui.add_space(10.0);

                            TableBuilder::new(section_ui)
                                .id_salt("largest_files")
                                .striped(true)
                                .vscroll(false)
                                .auto_shrink([false, true]).column(Column::remainder())
                                .column(Column::exact(120.0)).header(24.0, |mut header|{
                                header.col(|ui|{
                                    ui.strong("File");
                                });

                                header.col(|ui|{
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui|{
                                            ui.strong("Size");
                                        },
                                    );
                                });
                            }).body(|mut body|{
                                for (path, size) in result.largest_files(){
                                    body.row(24.0, |mut row|{
                                        row.col(|ui|{
                                            ui.label(
                                                path.file_name()
                                                    .unwrap_or(path.as_os_str())
                                                    .to_string_lossy()
                                                    .to_string(),
                                            );
                                        });

                                        row.col(|ui|{
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui|{
                                                    ui.label(format_size(*size));
                                                },
                                            );
                                        });
                                    });
                                }
                            });
                        });

                    content_ui.add_space(12.0);
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(24, 35, 55))
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgb(51, 65, 85)
                        ))
                        .inner_margin(12.0)
                        .show(content_ui, |section_ui| {
                            section_ui.take_available_width();
                            section_ui.heading(
                                egui::RichText::new("Storage by Extension")
                                    .color(egui::Color32::from_rgb(125, 211, 252))
                            );

                            section_ui.add_space(10.0);
                            let mut extensions: Vec<_> = result.extension_usage().iter().collect();
                            extensions.sort_unstable_by(|a, b| b.1.cmp(a.1));

                            TableBuilder::new(section_ui)
                                .id_salt("extension_usage")
                                .striped(true)
                                .vscroll(false)
                                .auto_shrink([false, true])
                                .column(Column::remainder())
                                .column(Column::exact(120.0)).header(24.0, |mut header|{
                                header.col(|ui|{
                                    ui.strong("Extension");
                                });
                                header.col(|ui|{
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui|{
                                            ui.strong("Size");
                                        },
                                    );
                                });
                            }).body(|mut body|{
                               for (extension, size) in extensions{
                                   body.row(24.0, |mut row|{
                                       row.col(|ui|{
                                             ui.label(extension);
                                       });
                                       row.col(|ui| {
                                           ui.with_layout(
                                               egui::Layout::right_to_left(egui::Align::Center),
                                               |ui| {
                                                   ui.label(format_size(*size));
                                               },
                                           );
                                       });
                                   });
                               }
                            });
                        });
                } else if self.scan_error.is_none(){
                    content_ui.vertical_centered(|ui|{
                       ui.add_space(48.0);

                        ui.label(
                            egui::RichText::new("NO SCAN RESULTS")
                                .size(16.0)
                                .strong()
                                .color(egui::Color32::from_rgb(125,211,252))
                        );

                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new(
                                "Select a directory above and press Scan to inspect its contents."
                            ).color(egui::Color32::from_rgb(148, 163, 184))
                        );
                    });
                }
            });

        });
    }
}

fn format_size(bytes: u64) -> String{
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1{
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0{
        format!("{} {}", bytes, units[unit_index])
    } else {
        format!("{:.2} {}", size, units[unit_index])
    }
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_format_size(){
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(2048), "2.00 KiB");
        assert_eq!(format_size(5_242_880), "5.00 MiB");
        assert_eq!(format_size(10_737_418_240), "10.00 GiB");
        assert_eq!(format_size(1_099_511_627_776), "1.00 TiB");
    }
}