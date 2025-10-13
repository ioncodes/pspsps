use crate::widgets::{SharedContext, Widget};
use egui::Ui;
use psx_core::gpu::{MAX_SCREEN_HEIGHT, MAX_SCREEN_WIDTH};

pub struct GpuWidget {
    texture: Option<egui::TextureHandle>,
    magnifier_texture: Option<egui::TextureHandle>,
    show_popup: bool,
}

impl GpuWidget {
    pub fn new() -> Self {
        Self {
            texture: None,
            magnifier_texture: None,
            show_popup: false,
        }
    }
}

impl Widget for GpuWidget {
    fn title(&self) -> &str {
        "GPU"
    }

    fn ui(&mut self, ui: &mut Ui, shared_context: &mut SharedContext) {
        let frame = &shared_context.state.gpu.frame;
        let width = shared_context.state.gpu.width;
        let height = shared_context.state.gpu.height;

        if frame.len() == MAX_SCREEN_WIDTH * MAX_SCREEN_HEIGHT && width > 0 && height > 0 {
            // Crop the frame buffer to the actual display resolution
            let mut cropped_pixels = Vec::with_capacity(width * height);
            for y in 0..height {
                for x in 0..width {
                    let idx = y * MAX_SCREEN_WIDTH + x;
                    let (r, g, b) = frame[idx];
                    cropped_pixels.push(egui::Color32::from_rgb(r, g, b));
                }
            }

            let color_image = egui::ColorImage {
                size: [width, height],
                pixels: cropped_pixels,
                source_size: egui::Vec2::new(width as f32, height as f32),
            };

            // Update or create texture with nearest neighbor filtering
            let texture = self.texture.get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "gpu_frame",
                    color_image.clone(),
                    egui::TextureOptions::NEAREST,
                )
            });

            // Update the texture with new data
            texture.set(color_image.clone(), egui::TextureOptions::NEAREST);

            // Display the image with click detection
            let image_response = ui.add(
                egui::Image::new(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                    texture.id(),
                    egui::vec2(width as f32, height as f32),
                )))
                .sense(egui::Sense::click()),
            );

            if image_response.clicked() {
                self.show_popup = !self.show_popup;
            }

            // Show popup window with 2x scaled texture
            if self.show_popup {
                if self.magnifier_texture.is_none() {
                    self.magnifier_texture = Some(ui.ctx().load_texture(
                        "magnifier",
                        color_image.clone(),
                        egui::TextureOptions::NEAREST,
                    ));
                }

                if let Some(magnifier_texture) = &mut self.magnifier_texture {
                    magnifier_texture.set(color_image.clone(), egui::TextureOptions::NEAREST);
                    let texture_id = magnifier_texture.id();
                    self.show_magnifier(ui, texture_id, width, height);
                }
            }
        } else {
            ui.label("Waiting for GPU frame data...");
        }
    }
}

impl GpuWidget {
    fn show_magnifier(
        &mut self, ui: &mut Ui, texture_id: egui::TextureId, width: usize, height: usize,
    ) {
        egui::Area::new("gpu_magnifier_area".into())
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
