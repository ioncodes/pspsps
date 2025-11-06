mod input;
mod renderer;

use clap::Parser;
use psx_core::psx::Psx;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

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
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_state.handle_keyboard_event(&event);

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
                if let Some(psx) = &mut self.psx {
                    // Update controller state
                    let controller_state = self.input_state.get_controller_state();
                    psx.set_controller_state(controller_state);

                    // Run emulation for one frame
                    const VBLANK_CYCLES: usize = 564_480;

                    for _ in 0..VBLANK_CYCLES {
                        let _ = psx.step();
                    }

                    // Get frame from GPU
                    let (width, height) = psx.cpu.mmu.gpu.gp.resolution();
                    let frame = psx.cpu.mmu.gpu.display_frame();

                    // Render
                    if let Some(renderer) = &mut self.renderer {
                        if let Err(e) = renderer.render(width, height, &frame) {
                            eprintln!("Render error: {:?}", e);
                        }
                    }
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
            let cdrom_data = fs::read(cdrom_path).expect("Failed to read CD-ROM file");
            psx.load_cdrom(cdrom_data);
            println!("Loaded CD-ROM: {:?}", cdrom_path);
        }

        // Load sideload EXE if provided
        if let Some(sideload_path) = &args.sideload {
            let exe_data = fs::read(sideload_path).expect("Failed to read sideload EXE file");
            psx.sideload_exe(exe_data);
            println!("Loaded sideload EXE: {:?}", sideload_path);
        }

        Self {
            window: None,
            renderer: None,
            psx: Some(psx),
            input_state: input::InputState::new(),
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
}

fn main() {
    let args = Args::parse();
    println!("Starting pspsps with BIOS: {:?}", args.bios);

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(args);

    event_loop.run_app(&mut app).expect("Failed to run event loop");
}
