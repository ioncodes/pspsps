use crate::widgets::{SharedContext, Widget};
use egui::Ui;

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

        if frame.len() == 256 * 240 {
            // Convert RGB tuples to RGBA pixels for egui
            let pixels: Vec<egui::Color32> = frame
                .iter()
                .map(|(r, g, b)| egui::Color32::from_rgb(*r, *g, *b))
                .collect();

            let color_image = egui::ColorImage {
                size: [256, 240],
                pixels,
                source_size: egui::Vec2::new(256.0, 240.0),
            };

            // Update or create texture
            let texture = self.texture.get_or_insert_with(|| {
                ui.ctx()
                    .load_texture("gpu_frame", color_image.clone(), Default::default())
            });

            // Update the texture with new data
            texture.set(color_image, Default::default());

            // Display the image
            ui.image(&*texture);
        } else {
            ui.label("Waiting for GPU frame data...");
        }
    }
}
