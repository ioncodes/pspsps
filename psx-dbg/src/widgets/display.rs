use crate::widgets::{SharedContext, Widget};
use egui::Ui;

pub struct DisplayWidget {
    display_texture: Option<egui::TextureHandle>,
    magnifier_texture: Option<egui::TextureHandle>,
    show_popup: bool,
}

impl DisplayWidget {
    pub fn new() -> Self {
        Self {
            display_texture: None,
            magnifier_texture: None,
            show_popup: false,
        }
    }
}

impl Widget for DisplayWidget {
    fn title(&self) -> &str {
        "Display"
    }

    fn ui(&mut self, ui: &mut Ui, shared_context: &mut SharedContext) {
        let gpu_state = &shared_context.state.gpu;

        // Display the actual display frame
        if !gpu_state.display_frame.is_empty()
            && gpu_state.display_width > 0
            && gpu_state.display_height > 0
        {
            // Convert display frame to displayable format
            let mut display_pixels =
                Vec::with_capacity(gpu_state.display_width * gpu_state.display_height);
            for y in 0..gpu_state.display_height {
                for x in 0..gpu_state.display_width {
                    let idx = y * gpu_state.display_width + x;
                    let (r, g, b) = gpu_state.display_frame[idx];
                    display_pixels.push(egui::Color32::from_rgb(r, g, b));
                }
            }

            let display_color_image = egui::ColorImage {
                size: [gpu_state.display_width, gpu_state.display_height],
                pixels: display_pixels,
                source_size: egui::Vec2::new(
                    gpu_state.display_width as f32,
                    gpu_state.display_height as f32,
                ),
            };

            // Update or create display texture
            let display_texture = self.display_texture.get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "gpu_display_frame",
                    display_color_image.clone(),
                    egui::TextureOptions::NEAREST,
                )
            });
            display_texture.set(display_color_image.clone(), egui::TextureOptions::NEAREST);

            // Display the display frame with click detection for magnifier
            let image_response = ui.add(
                egui::Image::new(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                    display_texture.id(),
                    egui::vec2(
                        gpu_state.display_width as f32,
                        gpu_state.display_height as f32,
                    ),
                )))
                .sense(egui::Sense::click()),
            );

            if image_response.clicked() {
                self.show_popup = !self.show_popup;
            }

            // Show popup window with 2x scaled display texture
            if self.show_popup {
                if self.magnifier_texture.is_none() {
                    self.magnifier_texture = Some(ui.ctx().load_texture(
                        "magnifier",
                        display_color_image.clone(),
                        egui::TextureOptions::NEAREST,
                    ));
                }

                if let Some(magnifier_texture) = &mut self.magnifier_texture {
                    magnifier_texture
                        .set(display_color_image.clone(), egui::TextureOptions::NEAREST);
                    let texture_id = magnifier_texture.id();
                    self.show_magnifier(
                        ui,
                        texture_id,
                        gpu_state.display_width,
                        gpu_state.display_height,
                    );
                }
            }
        } else {
            ui.label("Waiting for frame...");
        }
    }
}

impl DisplayWidget {
    fn show_magnifier(
        &mut self, ui: &mut Ui, texture_id: egui::TextureId, width: usize, height: usize,
    ) {
        egui::Area::new("display_magnifier_area".into())
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::NONE
                    .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                    .show(ui, |ui| {
                        ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                            texture_id,
                            egui::vec2(width as f32 * 2.0, height as f32 * 2.0),
                        )));
                    });
            });
    }
}
