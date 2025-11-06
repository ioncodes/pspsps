use crate::widgets::{SharedContext, Widget};
use egui::Ui;
use psx_core::gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};

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

        if frame.len() == SCREEN_WIDTH * SCREEN_HEIGHT {
            let pixels: Vec<egui::Color32> = frame
                .iter()
                .map(|(r, g, b)| egui::Color32::from_rgb(*r, *g, *b))
                .collect();

            let color_image = egui::ColorImage {
                size: [SCREEN_WIDTH, SCREEN_HEIGHT],
                pixels,
                source_size: egui::Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
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

            // Display the image at 2x scale (512x480)
            ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                texture.id(),
                egui::vec2(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32) * 2.0,
            )));
        } else {
            ui.label("Waiting for GPU frame data...");
        }
    }
}
