use std::path::{Path, PathBuf};

pub(super) fn zed_uninstall() {
    println!("Removing the prepared Vize Zed extension source...");

    match get_zed_extensions_dir() {
        Some(dir) => {
            let vize_dir = zed_dev_extension_dir(&dir);

            if vize_dir.exists() {
                if let Err(error) = std::fs::remove_dir_all(&vize_dir) {
                    eprintln!("Failed to remove extension: {error}");
                    std::process::exit(1);
                }
                println!("✓ Prepared Vize extension source removed");
                println!(
                    "  Use Zed's extension settings to remove an already activated extension."
                );
            } else {
                println!("Prepared extension source not found");
            }
        }
        None => eprintln!("Could not find Zed extensions directory"),
    }
}

pub(super) fn zed_status() {
    match get_zed_extensions_dir() {
        Some(dir) => {
            let installed_dir = dir.join("vize");
            let vize_dir = zed_dev_extension_dir(&dir);
            if installed_dir.join("extension.wasm").is_file() {
                println!("✓ Vize extension is active in Zed");
                println!("  Location: {}", installed_dir.display());
                return;
            }
            if vize_dir.exists() {
                println!("○ Vize extension source is prepared for Zed");
                println!("  Source: {}", vize_dir.display());
                println!("  Run 'Zed: Install Dev Extension' in Zed and select this directory.");
            } else {
                println!("✗ Vize extension is not active or prepared for Zed");
            }
        }
        None => println!("✗ Zed extensions directory not found"),
    }
}

pub(super) fn zed_dev_extension_dir(installed_dir: &Path) -> PathBuf {
    installed_dir
        .parent()
        .unwrap_or(installed_dir)
        .join("dev")
        .join("vize")
}

pub(super) fn get_zed_extensions_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().map(|dir| dir.join("Zed/extensions/installed"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::data_dir().map(|dir| dir.join("zed/extensions/installed"))
    }

    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir().map(|dir| dir.join("Zed/extensions/installed"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}
