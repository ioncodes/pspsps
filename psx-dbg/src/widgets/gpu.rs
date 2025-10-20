use crate::widgets::{SharedContext, Widget};
use egui::Ui;
use psx_core::gpu::{VRAM_HEIGHT, VRAM_WIDTH};

pub struct GpuWidget {
    vram_texture: Option<egui::TextureHandle>,
    display_texture: Option<egui::TextureHandle>,
    magnifier_texture: Option<egui::TextureHandle>,
    show_popup: bool,
}

impl GpuWidget {
    pub fn new() -> Self {
        Self {
            vram_texture: None,
            display_texture: None,
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
        let vram_frame = &shared_context.state.gpu.vram_frame;
        let vram_width = shared_context.state.gpu.vram_width;
        let vram_height = shared_context.state.gpu.vram_height;

        let display_frame = &shared_context.state.gpu.display_frame;
        let display_width = shared_context.state.gpu.display_width;
        let display_height = shared_context.state.gpu.display_height;

        // Display VRAM texture
        ui.heading("VRAM");
        if vram_frame.len() == VRAM_WIDTH * VRAM_HEIGHT && vram_width > 0 && vram_height > 0 {
            // Convert VRAM to displayable format
            let mut vram_pixels = Vec::with_capacity(vram_width * vram_height);
            for y in 0..vram_height {
                for x in 0..vram_width {
                    let idx = y * VRAM_WIDTH + x;
                    let (r, g, b) = vram_frame[idx];
                    vram_pixels.push(egui::Color32::from_rgb(r, g, b));
                }
            }

            let vram_color_image = egui::ColorImage {
                size: [vram_width, vram_height],
                pixels: vram_pixels,
                source_size: egui::Vec2::new(vram_width as f32, vram_height as f32),
            };

            // Update or create VRAM texture
            let vram_texture = self.vram_texture.get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "gpu_vram_frame",
                    vram_color_image.clone(),
                    egui::TextureOptions::NEAREST,
                )
            });
            vram_texture.set(vram_color_image.clone(), egui::TextureOptions::NEAREST);

            // Display VRAM image
            ui.add(egui::Image::new(egui::ImageSource::Texture(
                egui::load::SizedTexture::new(
                    vram_texture.id(),
                    egui::vec2(vram_width as f32, vram_height as f32),
                ),
            )));
        } else {
            ui.label("Waiting for frame...");
        }

        ui.separator();

        // Display the actual display frame
        ui.heading("Display");
        if !display_frame.is_empty() && display_width > 0 && display_height > 0 {
            // Convert display frame to displayable format
            let mut display_pixels = Vec::with_capacity(display_width * display_height);
            for y in 0..display_height {
                for x in 0..display_width {
                    let idx = y * display_width + x;
                    let (r, g, b) = display_frame[idx];
                    display_pixels.push(egui::Color32::from_rgb(r, g, b));
                }
            }

            let display_color_image = egui::ColorImage {
                size: [display_width, display_height],
                pixels: display_pixels,
                source_size: egui::Vec2::new(display_width as f32, display_height as f32),
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
                    egui::vec2(display_width as f32, display_height as f32),
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
                    self.show_magnifier(ui, texture_id, display_width, display_height);
                }
            }
        } else {
            ui.label("Waiting for frame...");
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
