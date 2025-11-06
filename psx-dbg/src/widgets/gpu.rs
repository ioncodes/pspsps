use crate::widgets::{SharedContext, Widget};
use egui::Ui;
use psx_core::gpu::{MAX_SCREEN_WIDTH, MAX_SCREEN_HEIGHT};

pub struct GpuWidget {
    texture: Option<egui::TextureHandle>,
}

impl GpuWidget {
    pub fn new() -> Self {
        Self { texture: None }
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
            texture.set(color_image, egui::TextureOptions::NEAREST);

            // Display the image at its actual resolution
            ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                texture.id(),
                egui::vec2(width as f32, height as f32),
            )));
        } else {
            ui.label("Waiting for GPU frame data...");
        }
    }
}
