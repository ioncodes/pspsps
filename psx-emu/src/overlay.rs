use std::time::Instant;

use egui::epaint::ClippedPrimitive;
use egui::{Align2, Area, Color32, FontId, Frame, Grid, Id, Margin, RichText, Ui};
use egui_wgpu::{Renderer as EguiRenderer, ScreenDescriptor};
use psx_core::sio::joy::ControllerState;
use wgpu::{self, CommandBuffer, CommandEncoder, Device, Queue, TextureFormat};

#[derive(Clone)]
pub struct PauseOverlayState {
    pub controller_state: ControllerState,
    pub scale_factor: f32,
    pub now_playing: Option<String>,
}

pub struct PreparedOverlay {
    pub paint_jobs: Vec<ClippedPrimitive>,
    pub screen_descriptor: ScreenDescriptor,
    pub textures_delta: egui::TexturesDelta,
}

pub struct PauseOverlay {
    ctx: egui::Context,
    renderer: EguiRenderer,
    start_time: Instant,
}

impl PauseOverlay {
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        Self {
            ctx: egui::Context::default(),
            renderer: EguiRenderer::new(device, surface_format, None, 1, false),
            start_time: Instant::now(),
        }
    }

    pub fn prepare(
        &mut self, device: &Device, queue: &Queue, surface_size: (u32, u32), overlay: PauseOverlayState,
    ) -> Option<PreparedOverlay> {
        if surface_size.0 == 0 || surface_size.1 == 0 {
            return None;
        }

        let scale_factor = overlay.scale_factor.max(0.5);
        let screen_rect = egui::Rect::from_min_max(
            egui::Pos2::new(0.0, 0.0),
            egui::Pos2::new(
                surface_size.0 as f32 / scale_factor,
                surface_size.1 as f32 / scale_factor,
            ),
        );

        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(screen_rect);
        raw_input.time = Some(self.start_time.elapsed().as_secs_f64());
        if let Some(info) = raw_input.viewports.get_mut(&raw_input.viewport_id) {
            info.native_pixels_per_point = Some(scale_factor);
            info.inner_rect = Some(screen_rect);
        }

        let full_output = self.ctx.run(raw_input, |ctx| {
            Self::render_pause_overlay(ctx, &overlay);
        });

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, image_delta);
        }

        let paint_jobs = self.ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        Some(PreparedOverlay {
            paint_jobs,
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [surface_size.0, surface_size.1],
                pixels_per_point: scale_factor,
            },
            textures_delta: full_output.textures_delta,
        })
    }

    pub fn upload_buffers(
        &mut self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, overlay: &PreparedOverlay,
    ) -> Vec<CommandBuffer> {
        self.renderer
            .update_buffers(device, queue, encoder, &overlay.paint_jobs, &overlay.screen_descriptor)
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass<'static>, overlay: &PreparedOverlay) {
        self.renderer
            .render(render_pass, &overlay.paint_jobs, &overlay.screen_descriptor);
    }

    pub fn free_textures(&mut self, overlay: &PreparedOverlay) {
        for id in &overlay.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }

    fn render_pause_overlay(ctx: &egui::Context, overlay: &PauseOverlayState) {
        let controller = overlay.controller_state;
        Area::new(Id::new("psx_pause_overlay"))
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_black_alpha(220))
                    .corner_radius(18.0)
                    .inner_margin(Margin::symmetric(32, 24))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Paused").font(FontId::proportional(30.0)).strong());
                            ui.label(
                                RichText::new("Press F5 to resume · F12 for a screenshot")
                                    .color(Color32::from_gray(200)),
                            );
                            ui.add_space(12.0);

                            ui.label(RichText::new("Controller inputs").strong());
                            ui.add_space(4.0);

                            Grid::new("controller_grid")
                                .num_columns(2)
                                .spacing([28.0, 6.0])
                                .show(ui, |ui| {
                                    Self::render_binding_row(ui, "D-Pad Up", "Arrow Up", controller.d_up);
                                    Self::render_binding_row(ui, "D-Pad Down", "Arrow Down", controller.d_down);
                                    Self::render_binding_row(ui, "D-Pad Left", "Arrow Left", controller.d_left);
                                    Self::render_binding_row(ui, "D-Pad Right", "Arrow Right", controller.d_right);
                                    Self::render_binding_row(ui, "Cross", "Z / Y", controller.cross);
                                    Self::render_binding_row(ui, "Circle", "X", controller.circle);
                                    Self::render_binding_row(ui, "Square", "A", controller.square);
                                    Self::render_binding_row(ui, "Triangle", "S", controller.triangle);
                                    Self::render_binding_row(ui, "L1", "Q", controller.l1);
                                    Self::render_binding_row(ui, "L2", "W", controller.l2);
                                    Self::render_binding_row(ui, "R1", "E", controller.r1);
                                    Self::render_binding_row(ui, "R2", "R", controller.r2);
                                    Self::render_binding_row(ui, "Start", "Enter", controller.start);
                                    Self::render_binding_row(ui, "Select", "Backspace", controller.select);
                                });
                        });
                    });
            });

        if let Some(now_playing) = overlay.now_playing.as_deref() {
            Area::new(Id::new("psx_currently_playing_overlay"))
                .anchor(Align2::CENTER_TOP, [0.0, 25.0])
                .show(ctx, |ui| {
                    Frame::new()
                        .fill(Color32::from_black_alpha(220))
                        .corner_radius(18.0)
                        .inner_margin(Margin::symmetric(24, 16))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("Currently playing: {}", now_playing))
                                    .font(FontId::proportional(20.0))
                                    .color(Color32::from_gray(230)),
                            );
                        });
                });
        }
    }

    fn render_binding_row(ui: &mut Ui, label: &str, key_hint: &str, active: bool) {
        let color = if active {
            Color32::from_rgb(120, 220, 120)
        } else {
            Color32::from_gray(200)
        };

        ui.label(RichText::new(label).color(color).strong());
        ui.label(RichText::new(key_hint).color(color));
        ui.end_row();
    }
}
