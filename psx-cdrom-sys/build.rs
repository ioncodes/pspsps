use std::env;
use std::path::PathBuf;

fn main() {
    let cdrom_path = PathBuf::from("../submodules/cdrom");

    cc::Build::new()
        .file(cdrom_path.join("audio.c"))
        .file(cdrom_path.join("cdrom.c"))
        .file(cdrom_path.join("cue.c"))
        .file(cdrom_path.join("disc.c"))
        .file(cdrom_path.join("impl.c"))
        .file(cdrom_path.join("list.c"))
        .file(cdrom_path.join("queue.c"))
        .include(&cdrom_path)
        .warnings(false)
        .compile("cdrom");

    println!("cargo:rerun-if-changed={}", cdrom_path.join("audio.c").display());
    println!("cargo:rerun-if-changed={}", cdrom_path.join("cdrom.c").display());
    println!("cargo:rerun-if-changed={}", cdrom_path.join("cue.c").display());
    println!("cargo:rerun-if-changed={}", cdrom_path.join("disc.c").display());
    println!("cargo:rerun-if-changed={}", cdrom_path.join("impl.c").display());
    println!("cargo:rerun-if-changed={}", cdrom_path.join("list.c").display());
    println!("cargo:rerun-if-changed={}", cdrom_path.join("queue.c").display());
    println!("cargo:rerun-if-changed={}", cdrom_path.join("cdrom.h").display());

    let bindings = bindgen::Builder::default()
        .header(cdrom_path.join("cdrom.h").to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("psx_cdrom_create")
        .allowlist_function("psx_cdrom_init")
        .allowlist_function("psx_cdrom_destroy")
        .allowlist_function("psx_cdrom_reset")
        .allowlist_function("psx_cdrom_read8")
        .allowlist_function("psx_cdrom_read16")
        .allowlist_function("psx_cdrom_read32")
        .allowlist_function("psx_cdrom_write8")
        .allowlist_function("psx_cdrom_write16")
        .allowlist_function("psx_cdrom_write32")
        .allowlist_function("psx_cdrom_update")
        .allowlist_function("psx_cdrom_open")
        .allowlist_function("psx_cdrom_get_audio_samples")
        .allowlist_type("psx_cdrom_t")
        .opaque_type("psx_cdrom_t")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
