use crate::widgets::{SharedContext, Widget};
use egui::Ui;
use psx_core::gpu::{VRAM_HEIGHT, VRAM_WIDTH};

pub struct GpuWidget {
    vram_texture: Option<egui::TextureHandle>,
    magnified_texture: Option<egui::TextureHandle>,
}

impl GpuWidget {
    pub fn new() -> Self {
        Self {
            vram_texture: None,
            magnified_texture: None,
        }
    }
}

impl Widget for GpuWidget {
    fn title(&self) -> &str {
        "GPU"
    }

    fn ui(&mut self, ui: &mut Ui, shared_context: &mut SharedContext) {
        let gpu_state = &shared_context.state.gpu;

        ui.heading("GPU Status");
        ui.horizontal(|ui| {
            ui.label("FPS:");
            ui.monospace(format!("{:.2}", gpu_state.fps));
        });
        ui.horizontal(|ui| {
            ui.label("GPUSTAT:");
            ui.monospace(format!("{:08X}", gpu_state.gp1_status.0));
        });

        ui.horizontal(|ui| {
            ui.label(format!(
                "Resolution: {}x{} {} (interlaced: {})",
                gpu_state.gp1_status.hres(),
                gpu_state.gp1_status.vres(),
                gpu_state.gp1_status.video_mode(),
                gpu_state.gp1_status.vertical_interlace()
            ));
            ui.checkbox(&mut gpu_state.gp1_status.display_enable(), "Display Enable");
        });

        ui.horizontal(|ui| {
            ui.label(format!(
                "Ready CMD: {}",
                gpu_state.gp1_status.ready_to_receive_cmd_word()
            ));
            ui.separator();
            ui.label(format!(
                "Ready VRAM to CPU: {}",
                gpu_state.gp1_status.ready_to_send_vram_to_cpu()
            ));
            ui.separator();
            ui.label(format!(
                "Ready DMA: {}",
                gpu_state.gp1_status.ready_to_receive_dma_block()
            ));
        });

        ui.separator();

        // Dump buttons
        ui.horizontal(|ui| {
            if ui.button("Dump VRAM to PNG").clicked() {
                if let Err(e) = Self::save_vram_to_png(gpu_state) {
                    shared_context.toasts.add(egui_toast::Toast {
                        text: format!("Failed to save VRAM: {}", e).into(),
                        kind: egui_toast::ToastKind::Error,
                        options: egui_toast::ToastOptions::default()
                            .duration(Some(std::time::Duration::from_secs(3))),
                        style: Default::default(),
                    });
                } else {
                    shared_context.toasts.add(egui_toast::Toast {
                        text: "VRAM saved to vram_dump.png".into(),
                        kind: egui_toast::ToastKind::Success,
                        options: egui_toast::ToastOptions::default()
                            .duration(Some(std::time::Duration::from_secs(3))),
                        style: Default::default(),
                    });
                }
            }

            if ui.button("Dump Display to PNG").clicked() {
                if let Err(e) = Self::save_display_to_png(gpu_state) {
                    shared_context.toasts.add(egui_toast::Toast {
                        text: format!("Failed to save display: {}", e).into(),
                        kind: egui_toast::ToastKind::Error,
                        options: egui_toast::ToastOptions::default()
                            .duration(Some(std::time::Duration::from_secs(3))),
                        style: Default::default(),
                    });
                } else {
                    shared_context.toasts.add(egui_toast::Toast {
                        text: "Display saved to display_dump.png".into(),
                        kind: egui_toast::ToastKind::Success,
                        options: egui_toast::ToastOptions::default()
                            .duration(Some(std::time::Duration::from_secs(3))),
                        style: Default::default(),
                    });
                }
            }
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

            // Display VRAM image with hover detection
            let vram_image =
                egui::Image::new(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                    vram_texture.id(),
                    egui::vec2(gpu_state.vram_width as f32, gpu_state.vram_height as f32),
                )));

            let response = ui.add(vram_image);

            // Show magnified view on hover
            if let Some(hover_pos) = response.hover_pos() {
                let rect = response.rect;
                // Calculate the pixel position in the VRAM texture
                let relative_pos = hover_pos - rect.min;
                let vram_x = (relative_pos.x / rect.width() * gpu_state.vram_width as f32) as usize;
                let vram_y =
                    (relative_pos.y / rect.height() * gpu_state.vram_height as f32) as usize;

                // Define magnification area (100x100 pixels centered on hover position)
                let mag_size = 100;
                let half_mag = mag_size / 2;

                // Calculate bounds with clamping
                let start_x = vram_x
                    .saturating_sub(half_mag)
                    .min(gpu_state.vram_width.saturating_sub(mag_size));
                let start_y = vram_y
                    .saturating_sub(half_mag)
                    .min(gpu_state.vram_height.saturating_sub(mag_size));
                let end_x = (start_x + mag_size).min(gpu_state.vram_width);
                let end_y = (start_y + mag_size).min(gpu_state.vram_height);

                // Extract magnified region
                let mut mag_pixels = Vec::new();
                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let idx = y * VRAM_WIDTH + x;
                        if idx < gpu_state.vram_frame.len() {
                            let (r, g, b) = gpu_state.vram_frame[idx];
                            mag_pixels.push(egui::Color32::from_rgb(r, g, b));
                        }
                    }
                }

                let mag_width = end_x - start_x;
                let mag_height = end_y - start_y;

                if mag_width > 0 && mag_height > 0 && mag_pixels.len() == mag_width * mag_height {
                    let mag_color_image = egui::ColorImage {
                        size: [mag_width, mag_height],
                        pixels: mag_pixels,
                        source_size: egui::Vec2::new(mag_width as f32, mag_height as f32),
                    };

                    // Update or create magnified texture
                    let mag_texture = self.magnified_texture.get_or_insert_with(|| {
                        ui.ctx().load_texture(
                            "gpu_magnified_vram",
                            mag_color_image.clone(),
                            egui::TextureOptions::NEAREST,
                        )
                    });
                    mag_texture.set(mag_color_image.clone(), egui::TextureOptions::NEAREST);

                    // Show magnified view in a popup window near cursor
                    let popup_size = 200.0;
                    egui::Area::new(egui::Id::new("vram_magnifier"))
                        .fixed_pos(hover_pos + egui::vec2(15.0, 15.0))
                        .order(egui::Order::Foreground)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                ui.label(format!("Position: ({}, {})", vram_x, vram_y));
                                ui.add(egui::Image::new(egui::ImageSource::Texture(
                                    egui::load::SizedTexture::new(
                                        mag_texture.id(),
                                        egui::vec2(popup_size, popup_size),
                                    ),
                                )));
                            });
                        });
                }
            }
        } else {
            ui.label("Waiting for frame...");
        }
    }
}

impl GpuWidget {
    fn save_vram_to_png(gpu_state: &crate::states::gpu::GpuState) -> Result<(), String> {
        use image::{RgbImage, Rgb};

        if gpu_state.vram_frame.is_empty() || gpu_state.vram_width == 0 || gpu_state.vram_height == 0 {
            return Err("No VRAM frame available".to_string());
        }

        let mut img = RgbImage::new(gpu_state.vram_width as u32, gpu_state.vram_height as u32);

        for y in 0..gpu_state.vram_height {
            for x in 0..gpu_state.vram_width {
                let idx = y * VRAM_WIDTH + x;
                if idx < gpu_state.vram_frame.len() {
                    let (r, g, b) = gpu_state.vram_frame[idx];
                    img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
                }
            }
        }

        img.save("vram_dump.png").map_err(|e| e.to_string())?;
        Ok(())
    }

    fn save_display_to_png(gpu_state: &crate::states::gpu::GpuState) -> Result<(), String> {
        use image::{RgbImage, Rgb};

        if gpu_state.display_frame.is_empty() || gpu_state.display_width == 0 || gpu_state.display_height == 0 {
            return Err("No display frame available".to_string());
        }

        let mut img = RgbImage::new(gpu_state.display_width as u32, gpu_state.display_height as u32);

        for y in 0..gpu_state.display_height {
            for x in 0..gpu_state.display_width {
                let idx = y * gpu_state.display_width + x;
                let (r, g, b) = gpu_state.display_frame[idx];
                img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
            }
        }

        img.save("display_dump.png").map_err(|e| e.to_string())?;
        Ok(())
    }
}
