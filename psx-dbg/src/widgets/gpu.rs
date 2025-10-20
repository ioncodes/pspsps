use crate::widgets::{SharedContext, Widget};
use egui::Ui;
use psx_core::gpu::{VRAM_HEIGHT, VRAM_WIDTH};

pub struct GpuWidget {
    vram_texture: Option<egui::TextureHandle>,
}

impl GpuWidget {
    pub fn new() -> Self {
        Self { vram_texture: None }
    }
}

impl Widget for GpuWidget {
    fn title(&self) -> &str {
        "GPU"
    }

    fn ui(&mut self, ui: &mut Ui, shared_context: &mut SharedContext) {
        let gpu_state = &shared_context.state.gpu;

        // Display GPU status information
        ui.heading("GPU Status");
        ui.horizontal(|ui| {
            ui.label(format!(
                "Resolution: {}x{}",
                gpu_state.horizontal_resolution, gpu_state.vertical_resolution
            ));
            ui.separator();
            ui.label(format!("FIFO: {}", gpu_state.fifo_len));
            ui.separator();
            ui.label(format!("GP1 Status: 0x{:08X}", gpu_state.gp1_status));
        });
        ui.separator();

        // Display VRAM texture
        ui.heading("VRAM");
        if gpu_state.vram_frame.len() == VRAM_WIDTH * VRAM_HEIGHT
            && gpu_state.vram_width > 0
            && gpu_state.vram_height > 0
        {
            // Convert VRAM to displayable format
            let mut vram_pixels = Vec::with_capacity(gpu_state.vram_width * gpu_state.vram_height);
            for y in 0..gpu_state.vram_height {
                for x in 0..gpu_state.vram_width {
                    let idx = y * VRAM_WIDTH + x;
                    let (r, g, b) = gpu_state.vram_frame[idx];
                    vram_pixels.push(egui::Color32::from_rgb(r, g, b));
                }
            }

            let vram_color_image = egui::ColorImage {
                size: [gpu_state.vram_width, gpu_state.vram_height],
                pixels: vram_pixels,
                source_size: egui::Vec2::new(
                    gpu_state.vram_width as f32,
                    gpu_state.vram_height as f32,
                ),
            };

            // Update or create VRAM texture
            let vram_texture = self.vram_texture.get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "gpu_gpu_state.vram_frame",
                    vram_color_image.clone(),
                    egui::TextureOptions::NEAREST,
                )
            });
            vram_texture.set(vram_color_image.clone(), egui::TextureOptions::NEAREST);

            // Display VRAM image
            ui.add(egui::Image::new(egui::ImageSource::Texture(
                egui::load::SizedTexture::new(
                    vram_texture.id(),
                    egui::vec2(gpu_state.vram_width as f32, gpu_state.vram_height as f32),
                ),
            )));
        } else {
            ui.label("Waiting for frame...");
        }
    }
}
