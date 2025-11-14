use psx_core::psx::Psx;
use psx_core::sio::joy::ControllerState;
use std::collections::HashMap;
use std::io::Read;

const FRAME_INTERVAL: usize = 60 * 2;
const MAX_FRAMES: usize = FRAME_INTERVAL * 30;

const CONTROLLER_INTERVAL: usize = 60 * 5;

fn save_frame_as_image(frame: &[(u8, u8, u8)], width: u32, height: u32, name: &str, frame_number: usize) {
    let mut img_buffer = image::RgbImage::new(width, height);

    for (i, &(r, g, b)) in frame.iter().enumerate() {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        img_buffer.put_pixel(x, y, image::Rgb([r, g, b]));
    }

    let filename = format!("screenshots/{}/{}.png", name, frame_number);
    img_buffer.save(filename).expect("Failed to save image");
}

fn remove_single_color_screenshots(rom_name: &str) {
    let screenshots_dir = format!("screenshots/{}", rom_name);
    let dir_path = std::path::Path::new(&screenshots_dir);

    let entries = match std::fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read directory {}: {}", screenshots_dir, e);
            return;
        }
    };

    let mut removed_count = 0;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("png") {
            continue;
        }

        // Load the image
        let img = match image::open(&path) {
            Ok(img) => img.to_rgb8(),
            Err(e) => {
                eprintln!("Failed to load image {:?}: {}", path, e);
                continue;
            }
        };

        // Check if all pixels are the same color
        if img.pixels().count() == 0 {
            continue;
        }

        let first_pixel = img.pixels().next().unwrap();
        let all_same = img.pixels().all(|p| p == first_pixel);

        if all_same {
            let (r, g, b) = (first_pixel[0], first_pixel[1], first_pixel[2]);
            println!("Deleting {:?} (single color: RGB({}, {}, {}))", path, r, g, b);

            if let Err(e) = std::fs::remove_file(&path) {
                eprintln!("Failed to delete {:?}: {}", path, e);
            } else {
                removed_count += 1;
            }
        }
    }

    println!("Removed {} single-color screenshot(s)", removed_count);
}

fn load_rom(rom_path: &str) -> Result<Vec<u8>, String> {
    let path = std::path::Path::new(rom_path);

    // Check if it's a zip file
    if path.extension().and_then(|s| s.to_str()) == Some("zip") {
        println!("Detected ZIP file, extracting first .bin file...");

        let file = std::fs::File::open(rom_path)
            .map_err(|e| format!("Failed to open ZIP file: {}", e))?;

        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

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

        let mut bin_file = archive.by_name(first_bin)
            .map_err(|e| format!("Failed to extract {}: {}", first_bin, e))?;

        let mut rom_data = Vec::new();
        bin_file.read_to_end(&mut rom_data)
            .map_err(|e| format!("Failed to read {}: {}", first_bin, e))?;

        println!("Extracted {} bytes from {}", rom_data.len(), first_bin);

        Ok(rom_data)
    } else {
        // Not a zip, read directly
        std::fs::read(rom_path)
            .map_err(|e| format!("Failed to read ROM file: {}", e))
    }
}

fn remove_duplicate_screenshots(rom_name: &str) {
    let screenshots_dir = format!("screenshots/{}", rom_name);
    let dir_path = std::path::Path::new(&screenshots_dir);

    let entries = match std::fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read directory {}: {}", screenshots_dir, e);
            return;
        }
    };

    // Create hasher
    let hasher = image_hasher::HasherConfig::new()
        .hash_size(16, 16)
        .preproc_dct()
        .to_hasher();

    // Map to store hash -> first file with that hash
    let mut hash_map: HashMap<image_hasher::ImageHash, std::path::PathBuf> = HashMap::new();
    let mut files_to_process = Vec::new();

    // Collect all PNG files
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("png") {
            continue;
        }

        files_to_process.push(path);
    }

    // Sort by filename to ensure consistent ordering (keep lower frame numbers)
    files_to_process.sort();

    let mut removed_count = 0;

    for path in files_to_process {
        // Load and hash the image
        let img = match image::open(&path) {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Failed to load image {:?}: {}", path, e);
                continue;
            }
        };

        let hash = hasher.hash_image(&img);

        // Check if we've seen this hash before
        if let Some(original_path) = hash_map.get(&hash) {
            // This is a duplicate
            println!(
                "Deleting {:?} (duplicate of {:?})",
                path.file_name().unwrap(),
                original_path.file_name().unwrap()
            );

            if let Err(e) = std::fs::remove_file(&path) {
                eprintln!("Failed to delete {:?}: {}", path, e);
            } else {
                removed_count += 1;
            }
        } else {
            // First occurrence of this hash
            hash_map.insert(hash, path);
        }
    }

    println!("Removed {} duplicate screenshot(s)", removed_count);
}

fn main() {
    let bios_path = std::env::args().nth(1).expect("Please provide a path to the BIOS");
    let rom_path = std::env::args().nth(2).expect("Please provide a path to the ROM");

    let rom_name = std::path::Path::new(&rom_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap();
    std::fs::create_dir_all(format!("screenshots/{}", &rom_name)).expect("Failed to create screenshots directory");

    let bios = std::fs::read(bios_path).expect("Failed to read BIOS file");
    let mut psx = Psx::new(&bios);

    let rom = load_rom(&rom_path).expect("Failed to load ROM file");
    psx.load_cdrom(rom);

    let mut frames = 0;
    let mut buttons_pressed = false;

    loop {
        match psx.step() {
            Ok((_, true)) => {
                let (frame, width, height) = psx.frame();
                frames += 1;

                if frames % FRAME_INTERVAL == 0 && frames > FRAME_INTERVAL * 3 {
                    save_frame_as_image(&frame, width as u32, height as u32, rom_name, frames);
                    println!("Saved frame {}", frames);
                }

                if frames % CONTROLLER_INTERVAL == 0 {
                    if !buttons_pressed {
                        buttons_pressed = true;
                        
                        psx.set_controller_state(ControllerState {
                            circle: true,
                            start: true,
                            ..Default::default()
                        });
                    } else {
                        buttons_pressed = false;
                        psx.set_controller_state(ControllerState::default());
                    }
                }
            }
            Ok(_) => {}
            Err(_) => {
                println!("Emulation error occurred");
                break;
            }
        }

        if frames >= MAX_FRAMES {
            break;
        }
    }

    println!("Screenshots collected!");

    remove_single_color_screenshots(rom_name);
    remove_duplicate_screenshots(rom_name);
}
