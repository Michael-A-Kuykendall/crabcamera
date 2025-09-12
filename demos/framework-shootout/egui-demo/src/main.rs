use eframe::egui;
use std::time::Instant;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_title("CrabCamera egui Demo - Professional Camera Controls"),
        ..Default::default()
    };

    eframe::run_native(
        "CrabCamera egui Demo",
        options,
        Box::new(|_cc| Ok(Box::new(CameraDemo::default()))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WhiteBalance {
    Auto,
    Daylight,
    Cloudy,
    Tungsten,
    Fluorescent,
}

impl WhiteBalance {
    fn as_str(&self) -> &'static str {
        match self {
            WhiteBalance::Auto => "Auto",
            WhiteBalance::Daylight => "Daylight",
            WhiteBalance::Cloudy => "Cloudy",
            WhiteBalance::Tungsten => "Tungsten",
            WhiteBalance::Fluorescent => "Fluorescent",
        }
    }
}

struct CameraDemo {
    focus: f32,
    iso: u32,
    exposure: f32,
    white_balance: WhiteBalance,
    fps: f32,
    last_frame: Instant,
    photos_captured: u32,
    camera_connected: bool,
    preview_width: u32,
    preview_height: u32,
}

impl Default for CameraDemo {
    fn default() -> Self {
        Self {
            focus: 50.0,
            iso: 400,
            exposure: 1.0 / 60.0,
            white_balance: WhiteBalance::Auto,
            fps: 0.0,
            last_frame: Instant::now(),
            photos_captured: 0,
            camera_connected: true, // Mock connection
            preview_width: 1280,
            preview_height: 720,
        }
    }
}

impl eframe::App for CameraDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update FPS calculation
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame);
        self.fps = 1.0 / elapsed.as_secs_f32();
        self.last_frame = now;

        // Request continuous repaints for FPS counter
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Title and status
            ui.vertical_centered(|ui| {
                ui.heading("CrabCamera Professional Controls");
                
                let status_text = if self.camera_connected {
                    format!("Camera Connected | {}x{} | {:.1} FPS", 
                           self.preview_width, self.preview_height, self.fps)
                } else {
                    "Camera Disconnected".to_string()
                };
                
                ui.colored_label(
                    if self.camera_connected { egui::Color32::GREEN } else { egui::Color32::RED },
                    status_text
                );
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                // Left side - Camera Preview
                ui.vertical(|ui| {
                    ui.set_width(450.0);
                    
                    // Camera preview area
                    let (rect, _response) = ui.allocate_exact_size(
                        egui::Vec2::new(400.0, 300.0),
                        egui::Sense::hover()
                    );
                    
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_rgb(30, 30, 30)
                    );
                    
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(70, 130, 180))
                    );
                    
                    // Preview text
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "📷 Live Camera Preview",
                        egui::FontId::proportional(24.0),
                        egui::Color32::WHITE
                    );
                });

                ui.add_space(30.0);

                // Right side - Controls
                ui.vertical(|ui| {
                    ui.set_width(300.0);
                    
                    ui.heading("Camera Controls");
                    ui.add_space(15.0);

                    // Focus Control
                    ui.group(|ui| {
                        ui.label("Focus Control");
                        ui.horizontal(|ui| {
                            if ui.add(egui::Slider::new(&mut self.focus, 0.0..=100.0)
                                .text("")).changed() {
                                println!("Focus changed to: {:.1}%", self.focus);
                            }
                            ui.label(format!("{:.0}%", self.focus));
                        });
                    });

                    ui.add_space(10.0);

                    // ISO Control
                    ui.group(|ui| {
                        ui.label("ISO Sensitivity");
                        let mut iso_f32 = self.iso as f32;
                        ui.horizontal(|ui| {
                            if ui.add(egui::Slider::new(&mut iso_f32, 100.0..=3200.0)
                                .step_by(100.0)
                                .text("")).changed() {
                                self.iso = iso_f32 as u32;
                                println!("ISO changed to: {}", self.iso);
                            }
                            ui.label(format!("ISO {}", self.iso));
                        });
                    });

                    ui.add_space(10.0);

                    // Exposure Control
                    ui.group(|ui| {
                        ui.label("Shutter Speed");
                        ui.horizontal(|ui| {
                            if ui.add(egui::Slider::new(&mut self.exposure, 1.0/2000.0..=1.0/15.0)
                                .text("")).changed() {
                                println!("Exposure changed to: 1/{:.0}s", 1.0 / self.exposure);
                            }
                            ui.label(format!("1/{:.0}s", 1.0 / self.exposure));
                        });
                    });

                    ui.add_space(10.0);

                    // White Balance Control
                    ui.group(|ui| {
                        ui.label("White Balance");
                        egui::ComboBox::from_id_source("white_balance")
                            .selected_text(self.white_balance.as_str())
                            .show_ui(ui, |ui| {
                                for wb in [WhiteBalance::Auto, WhiteBalance::Daylight, 
                                          WhiteBalance::Cloudy, WhiteBalance::Tungsten, 
                                          WhiteBalance::Fluorescent] {
                                    if ui.selectable_value(&mut self.white_balance, wb, wb.as_str()).changed() {
                                        println!("White balance changed to: {:?}", wb);
                                    }
                                }
                            });
                    });

                    ui.add_space(20.0);

                    // Capture Button
                    let capture_button = egui::Button::new("📸 Capture Photo")
                        .min_size(egui::Vec2::new(200.0, 50.0));
                    
                    if ui.add(capture_button).clicked() {
                        self.photos_captured += 1;
                        println!("Photo captured! Total: {}", self.photos_captured);
                    }

                    ui.add_space(20.0);

                    // Performance Metrics
                    ui.group(|ui| {
                        ui.label("Performance Metrics");
                        ui.separator();
                        ui.label(format!("Photos Captured: {}", self.photos_captured));
                        ui.label(format!("Memory Usage: 18.2 MB"));
                        ui.label(format!("CPU Usage: 8.7%"));
                        ui.label(format!("Frame Latency: 16.7ms"));
                        ui.label(format!("Real FPS: {:.1}", self.fps));
                    });
                });
            });
        });
    }
}