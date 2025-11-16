mod input;
mod overlay;
mod renderer;

use clap::Parser;
use psx_core::psx::Psx;
use psx_core::sio::joy::ControllerState;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

fn load_rom(rom_path: &PathBuf) -> Result<Vec<u8>, String> {
    let path = rom_path.as_path();

    // Check if it's a zip file
    if path.extension().and_then(|s| s.to_str()) == Some("zip") {
        println!("Detected ZIP file, extracting first .bin file...");

        let file = std::fs::File::open(path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

        // Get all .bin files and sort them
        let mut bin_files: Vec<String> = archive
            .file_names()
            .filter(|name| name.to_lowercase().ends_with(".bin"))
            .map(|s| s.to_string())
            .collect();

        bin_files.sort();

        if bin_files.is_empty() {
            return Err("No .bin files found in ZIP archive".to_string());
        }

        let first_bin = &bin_files[0];
        println!("Extracting: {}", first_bin);

        let mut bin_file = archive
            .by_name(first_bin)
            .map_err(|e| format!("Failed to extract {}: {}", first_bin, e))?;

        let mut rom_data = Vec::new();
        bin_file
            .read_to_end(&mut rom_data)
            .map_err(|e| format!("Failed to read {}: {}", first_bin, e))?;

        println!("Extracted {} bytes from {}", rom_data.len(), first_bin);

        Ok(rom_data)
    } else {
        // Not a zip, read directly
        fs::read(path).map_err(|e| format!("Failed to read ROM file: {}", e))
    }
}

#[derive(Parser, Debug)]
#[command(name = "pspsps")]
#[command(about = "a cute psx emulator", long_about = None)]
struct Args {
    #[arg(short, long)]
    bios: PathBuf,

    #[arg(short, long)]
    cdrom: Option<PathBuf>,

    #[arg(short, long)]
    sideload: Option<PathBuf>,
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<renderer::Renderer>,
    psx: Option<Psx>,
    input_state: input::InputState,
    controller_state: ControllerState,
    frame_count: usize,
    fps_timer: std::time::Instant,
    current_fps: f64,
    paused: bool,
    window_scale_factor: f32,
    now_playing: Option<String>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("pspsps - a cute psx emulator")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 960));

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create window"),
            );

            // Initialize renderer
            let renderer =
                pollster::block_on(renderer::Renderer::new(window.clone())).expect("Failed to create renderer");

            self.window = Some(window);
            self.renderer = Some(renderer);

            if let Some(window) = &self.window {
                self.window_scale_factor = window.scale_factor() as f32;
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    if let Some(renderer) = &mut self.renderer {
                        renderer.resize(new_size);
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_state.handle_keyboard_event(&event);
                self.controller_state = self.input_state.get_controller_state();

                // Handle F5 to toggle pause
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F5)
                    && event.state == winit::event::ElementState::Pressed
                {
                    self.paused = !self.paused;
                }

                // Handle screenshot on F12
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F12)
                    && event.state == winit::event::ElementState::Pressed
                {
                    if let Some(psx) = &self.psx {
                        let (width, height) = psx.cpu.mmu.gpu.gp.resolution();
                        let frame = psx.cpu.mmu.gpu.display_frame();
                        self.save_screenshot(width, height, &frame);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    self.window_scale_factor = window.scale_factor() as f32;
                }

                let mut should_update_title = false;

                if let Some(psx) = &mut self.psx {
                    let controller_state = self.input_state.get_controller_state();
                    self.controller_state = controller_state;

                    // Only run emulation when not paused
                    if !self.paused {
                        // Update controller state
                        psx.set_controller_state(controller_state);

                        // Run emulation until frame completes
                        loop {
                            match psx.step() {
                                Ok((_, frame_complete)) => {
                                    if frame_complete {
                                        break;
                                    }
                                }
                                Err(_) => {
                                    eprintln!("Error during emulation step");
                                    break;
                                }
                            }
                        }

                        // Update FPS tracking
                        self.frame_count += 1;
                        let elapsed = self.fps_timer.elapsed().as_secs_f64();
                        if elapsed >= 1.0 {
                            self.current_fps = self.frame_count as f64 / elapsed;
                            self.frame_count = 0;
                            self.fps_timer = std::time::Instant::now();
                            should_update_title = true;
                        }
                    }

                    // Get frame from GPU
                    let (width, height) = psx.cpu.mmu.gpu.gp.resolution();
                    let mut frame = psx.cpu.mmu.gpu.display_frame();

                    // Darken frame when paused to make it clear emulation is stopped
                    if self.paused {
                        for pixel in &mut frame {
                            pixel.0 = pixel.0 / 3;
                            pixel.1 = pixel.1 / 3;
                            pixel.2 = pixel.2 / 3;
                        }
                    }

                    // Render
                    if let Some(renderer) = &mut self.renderer {
                        let overlay = if self.paused {
                            Some(renderer::PauseOverlayState {
                                controller_state: self.controller_state,
                                scale_factor: self.window_scale_factor,
                                now_playing: self.now_playing.clone(),
                            })
                        } else {
                            None
                        };

                        if let Err(e) = renderer.render(width, height, &frame, overlay) {
                            eprintln!("Render error: {:?}", e);
                        }
                    }
                }

                if should_update_title {
                    self.update_window_title();
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn new(args: Args) -> Self {
    // Load BIOS
        let bios = fs::read(&args.bios).expect("Failed to read BIOS file");

        // Create PSX instance
        let mut psx = Psx::new(&bios);

        // Load CD-ROM if provided
        if let Some(cdrom_path) = &args.cdrom {
            let cdrom_data = load_rom(cdrom_path).expect("Failed to load CD-ROM file");
            psx.load_cdrom(cdrom_data);
            println!("Loaded CD-ROM: {:?}", cdrom_path);
        }

        // Load sideload EXE if provided
        if let Some(sideload_path) = &args.sideload {
            let exe_data = fs::read(sideload_path).expect("Failed to read sideload EXE file");
            psx.sideload_exe(exe_data);
            println!("Loaded sideload EXE: {:?}", sideload_path);
        }

        let now_playing = args
            .cdrom
            .as_ref()
            .and_then(|path| Self::media_display_name(path))
            .or_else(|| args.sideload.as_ref().and_then(|path| Self::media_display_name(path)));

        Self {
            window: None,
            renderer: None,
            psx: Some(psx),
            input_state: input::InputState::new(),
            controller_state: ControllerState::default(),
            frame_count: 0,
            fps_timer: std::time::Instant::now(),
            current_fps: 0.0,
            paused: true, // Start paused
            window_scale_factor: 1.0,
            now_playing,
        }
    }

    fn update_window_title(&self) {
        if let Some(window) = &self.window {
            let title = format!("pspsps - a cute psx emulator - {:.2} FPS", self.current_fps);
            window.set_title(&title);
        }
    }

    fn save_screenshot(&self, width: usize, height: usize, frame: &[(u8, u8, u8)]) {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("screenshot_{}.png", timestamp);

        // Convert RGB to RGBA
        let mut rgba_data = Vec::with_capacity(width * height * 4);
        for (r, g, b) in frame {
            rgba_data.push(*r);
            rgba_data.push(*g);
            rgba_data.push(*b);
            rgba_data.push(255);
        }

        if let Err(e) = image::save_buffer(
            &filename,
            &rgba_data,
            width as u32,
            height as u32,
            image::ColorType::Rgba8,
        ) {
            eprintln!("Failed to save screenshot: {}", e);
        } else {
            println!("Screenshot saved: {}", filename);
        }
    }

    fn media_display_name(path: &Path) -> Option<String> {
        path.file_stem()
            .or_else(|| path.file_name())
            .map(|stem| stem.to_string_lossy().trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

fn main() {
    let args = Args::parse();
    println!("Starting pspsps with BIOS: {:?}", args.bios);

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(args);

    event_loop.run_app(&mut app).expect("Failed to run event loop");
}
