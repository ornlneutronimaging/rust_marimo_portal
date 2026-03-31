use eframe::egui;
use std::fs;
use std::path::Path;

const TOP_DIR: &str = "/Volumes/JeanHardDrive/SNS/VENUS";

fn main() -> eframe::Result {
    let mut options = eframe::NativeOptions::default();
    options.viewport = options.viewport.with_inner_size(egui::vec2(500.0, 350.0));
    eframe::run_native(
        "Marimo Application Launcher",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    )
}

struct MyApp {
    folders: Vec<String>,
    selected: Option<usize>,
    py_files: Vec<String>,
    selected_py: Option<usize>,
    description: Option<String>,
}

impl MyApp {
    fn new() -> Self {
        let mut folders = Vec::new();
        if let Ok(entries) = fs::read_dir(Path::new(TOP_DIR)) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        folders.push(name.to_string());
                    }
                }
            }
        }
        folders.sort();
        Self {
            folders,
            selected: None,
            py_files: Vec::new(),
            selected_py: None,
            description: None,
        }
    }

    fn refresh_py_files(&mut self) {
        self.py_files.clear();
        self.selected_py = None;
        if let Some(i) = self.selected {
            let notebooks_dir = Path::new(TOP_DIR)
                .join(&self.folders[i])
                .join("shared")
                .join("notebooks");
            if let Ok(entries) = fs::read_dir(&notebooks_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "py" {
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    self.py_files.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
            self.py_files.sort();
        }
    }

    fn refresh_description(&mut self) {
        self.description = None;
        if let (Some(folder_idx), Some(py_idx)) = (self.selected, self.selected_py) {
            let file_path = Path::new(TOP_DIR)
                .join(&self.folders[folder_idx])
                .join("shared")
                .join("notebooks")
                .join(&self.py_files[py_idx]);
            if let Ok(content) = fs::read_to_string(&file_path) {
                for line in content.lines().take(20) {
                    let trimmed = line.trim();
                    if trimmed.starts_with("description") {
                        if let Some(value) = trimmed.split('=').nth(1) {
                            let value = value.trim().trim_matches('"').trim_matches('\'');
                            if !value.is_empty() {
                                self.description = Some(value.to_string());
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::BLACK;
        visuals.window_fill = egui::Color32::BLACK;
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(20.0));
        style.spacing.item_spacing = egui::vec2(8.0, 12.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
        style.spacing.combo_height = 250.0;
        ctx.set_style(style);

        if self.selected_py.is_some() {
            egui::TopBottomPanel::bottom("bottom_panel")
                .frame(egui::Frame::new().fill(egui::Color32::BLACK).inner_margin(12.0))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if ui.button("Launch this application").clicked() {
                            // TODO: launch command
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            let label_width = 80.0;
            let combo_width = ui.available_width() - label_width - 20.0;

            // IPTS selector
            let prev_selected = self.selected;
            let current_label = match self.selected {
                Some(i) => self.folders[i].as_str(),
                None => "Select a folder...",
            };
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(label_width, ui.spacing().interact_size.y),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| { ui.strong("IPTS"); },
                );
                egui::ComboBox::from_id_salt("ipts_combo")
                    .selected_text(current_label)
                    .width(combo_width)
                    .show_ui(ui, |ui| {
                        for (i, folder) in self.folders.iter().enumerate() {
                            ui.selectable_value(&mut self.selected, Some(i), folder);
                        }
                    });
            });

            if self.selected != prev_selected {
                self.refresh_py_files();
                self.description = None;
            }

            // Notebook selector or "no application" message
            if self.selected.is_some() && self.py_files.is_empty() {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.add_space(label_width + 8.0);
                    ui.colored_label(egui::Color32::from_rgb(180, 180, 100), "No application found");
                });
            } else if !self.py_files.is_empty() {
                let prev_selected_py = self.selected_py;
                let current_py_label = match self.selected_py {
                    Some(i) => self.py_files[i].as_str(),
                    None => "Select a notebook...",
                };
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(label_width, ui.spacing().interact_size.y),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| { ui.strong("Notebook"); },
                    );
                    egui::ComboBox::from_id_salt("py_combo")
                        .selected_text(current_py_label)
                        .width(combo_width)
                        .show_ui(ui, |ui| {
                            for (i, file) in self.py_files.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_py, Some(i), file);
                            }
                        });
                });

                if self.selected_py != prev_selected_py {
                    self.refresh_description();
                }

                if self.selected_py.is_some() {
                    // Description box
                    if let Some(desc) = &self.description {
                        ui.add_space(5.0);
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(25, 25, 30))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 100)))
                            .corner_radius(6.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(desc);
                            });
                    }

                }
            }
        });
    }
}
