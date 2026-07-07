//! IDE command - Editor integration and LSP server
//!
//! This command provides:
//! - LSP server (default, alias for `vize lsp`)
//! - Editor extension installation for VSCode and Zed

use clap::{Args, Subcommand};
use std::path::PathBuf;
use std::process::Command;

mod editor_files;

use crate::commands::lsp::{LspTransport, transport_from_port};
use editor_files::{copy_dir_all, find_vscode_vsix, find_zed_extension_source};

const VSCODE_EXTENSION_ID: &str = "ubugeeei.vize";
const LEGACY_VSCODE_EXTENSION_ID: &str = "vize.vize";

#[derive(Args)]
pub struct IdeArgs {
    #[command(subcommand)]
    pub command: Option<IdeCommands>,

    /// Use stdio for communication (default, when no subcommand)
    #[arg(long, default_value = "true")]
    pub stdio: bool,

    /// TCP port for socket communication
    #[arg(long)]
    pub port: Option<u16>,

    /// Enable debug logging
    #[arg(long)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum IdeCommands {
    /// Install or manage VSCode extension
    Vscode(EditorArgs),
    /// Install or manage Zed extension
    Zed(EditorArgs),
}

#[derive(Args)]
pub struct EditorArgs {
    /// Install the extension
    #[arg(long)]
    pub install: bool,

    /// Uninstall the extension
    #[arg(long)]
    pub uninstall: bool,

    /// Show extension status
    #[arg(long)]
    pub status: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EditorOperation {
    Install,
    Status,
    Uninstall,
}

pub fn run(args: IdeArgs) {
    match args.command {
        Some(IdeCommands::Vscode(editor_args)) => run_vscode(editor_args),
        Some(IdeCommands::Zed(editor_args)) => run_zed(editor_args),
        None => run_lsp(args),
    }
}

/// Run LSP server (default behavior)
fn run_lsp(args: IdeArgs) {
    let result = match transport_from_port(args.port) {
        LspTransport::Stdio => vize_maestro::serve_blocking(),
        LspTransport::Tcp(port) => vize_maestro::serve_tcp_blocking(port),
    };

    if let Err(e) = result {
        eprintln!("LSP server error: {}", e);
        std::process::exit(1);
    }
}

/// Handle VSCode extension operations
fn run_vscode(args: EditorArgs) {
    match editor_operation(&args) {
        EditorOperation::Install => vscode_install(),
        EditorOperation::Status => vscode_status(),
        EditorOperation::Uninstall => vscode_uninstall(),
    }
}

/// Handle Zed extension operations
fn run_zed(args: EditorArgs) {
    match editor_operation(&args) {
        EditorOperation::Install => zed_install(),
        EditorOperation::Status => zed_status(),
        EditorOperation::Uninstall => zed_uninstall(),
    }
}

fn editor_operation(args: &EditorArgs) -> EditorOperation {
    if args.uninstall {
        EditorOperation::Uninstall
    } else if args.status {
        EditorOperation::Status
    } else {
        EditorOperation::Install
    }
}

// =============================================================================
// VSCode Extension
// =============================================================================

fn vscode_install() {
    println!("Installing Vize VSCode extension...");

    // Try to find the VSIX file
    let vsix_path = find_vscode_vsix();

    match vsix_path {
        Some(path) => {
            println!("Found extension: {}", path.display());
            install_vsix(&path);
        }
        None => {
            // Try to build from source
            println!("VSIX not found, building from source...");
            if build_vscode_extension() {
                if let Some(path) = find_vscode_vsix() {
                    install_vsix(&path);
                } else {
                    eprintln!("Failed to find built VSIX");
                    std::process::exit(1);
                }
            } else {
                println!("Source build unavailable; installing published extension...");
                install_published_vscode_extension();
            }
        }
    }
}

fn vscode_uninstall() {
    println!("Uninstalling Vize VSCode extension...");

    let status = Command::new("code")
        .args(["--uninstall-extension", VSCODE_EXTENSION_ID])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("✓ Vize extension uninstalled from VSCode");
        }
        Ok(_) => {
            eprintln!("Extension not installed or already removed");
        }
        Err(e) => {
            eprintln!("Failed to run 'code' command: {}", e);
            eprintln!("Make sure VSCode is installed and 'code' is in your PATH");
            std::process::exit(1);
        }
    }
}

fn vscode_status() {
    let output = Command::new("code").args(["--list-extensions"]).output();

    match output {
        Ok(out) => {
            #[allow(clippy::disallowed_types)]
            let extensions = std::string::String::from_utf8_lossy(&out.stdout);
            if vscode_extension_is_installed(&extensions) {
                println!("✓ Vize extension is installed in VSCode");
            } else {
                println!("✗ Vize extension is not installed in VSCode");
            }
        }
        Err(e) => {
            eprintln!("Failed to check VSCode extensions: {}", e);
            eprintln!("Make sure VSCode is installed and 'code' is in your PATH");
        }
    }
}

fn vscode_extension_is_installed(extensions: &str) -> bool {
    extensions.lines().any(|line| {
        let extension = line.trim();
        extension.eq_ignore_ascii_case(VSCODE_EXTENSION_ID)
            || extension.eq_ignore_ascii_case(LEGACY_VSCODE_EXTENSION_ID)
    })
}

fn build_vscode_extension() -> bool {
    // Try to find the extension source
    let source_dir = PathBuf::from("editors/vscode");
    if !source_dir.exists() {
        return false;
    }

    println!("Building VSCode extension...");

    // Run pnpm install and build
    let install_status = Command::new("pnpm")
        .args(["install"])
        .current_dir(&source_dir)
        .status();

    if !install_status.map(|s| s.success()).unwrap_or(false) {
        return false;
    }

    // Package the extension
    let package_status = Command::new("pnpm")
        .args(["run", "package"])
        .current_dir(&source_dir)
        .status();

    package_status.map(|s| s.success()).unwrap_or(false)
}

fn install_vsix(path: &std::path::Path) {
    println!("Installing VSIX: {}", path.display());

    let status = Command::new("code")
        .args(["--install-extension", &path.to_string_lossy()])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("✓ Vize extension installed successfully!");
            println!("  Restart VSCode to activate the extension.");
        }
        Ok(_) => {
            eprintln!("Failed to install extension");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run 'code' command: {}", e);
            eprintln!("Make sure VSCode is installed and 'code' is in your PATH");
            std::process::exit(1);
        }
    }
}

fn install_published_vscode_extension() {
    let status = Command::new("code")
        .args(["--install-extension", VSCODE_EXTENSION_ID])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("✓ Vize extension installed successfully!");
            println!("  Restart VSCode to activate the extension.");
        }
        Ok(_) => {
            eprintln!(
                "Failed to install published extension: {}",
                VSCODE_EXTENSION_ID
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run 'code' command: {}", e);
            eprintln!("Make sure VSCode is installed and 'code' is in your PATH");
            std::process::exit(1);
        }
    }
}

// =============================================================================
// Zed Extension
// =============================================================================

fn zed_install() {
    println!("Installing Vize Zed extension...");

    let extensions_dir = get_zed_extensions_dir();

    match extensions_dir {
        Some(dir) => {
            let vize_dir = dir.join("vize");
            let Some(source_dir) = find_zed_extension_source() else {
                eprintln!("Could not find editors/zed extension source");
                eprintln!(
                    "Please run from the vize repository or install from Zed's extension gallery"
                );
                std::process::exit(1);
            };

            if vize_dir.exists()
                && let Err(e) = std::fs::remove_dir_all(&vize_dir)
            {
                eprintln!("Failed to replace existing extension: {}", e);
                std::process::exit(1);
            }

            if let Err(e) = copy_dir_all(&source_dir, &vize_dir) {
                eprintln!("Failed to install extension: {}", e);
                std::process::exit(1);
            }

            println!("✓ Vize extension installed to: {}", vize_dir.display());
            println!("  Note: Configure Vize features explicitly in Zed settings.");
            println!();
            println!("  Start with lint-only mode:");
            println!("  {{");
            println!(
                "    \"languages\": {{ \"Vue\": {{ \"language_servers\": [\"vize\", \"...\"] }} }},"
            );
            println!(
                "    \"lsp\": {{ \"vize\": {{ \"initialization_options\": {{ \"lint\": true }} }} }}"
            );
            println!("  }}");
        }
        None => {
            eprintln!("Could not find Zed extensions directory");
            eprintln!("Make sure Zed is installed");
            std::process::exit(1);
        }
    }
}

fn zed_uninstall() {
    println!("Uninstalling Vize Zed extension...");

    let extensions_dir = get_zed_extensions_dir();

    match extensions_dir {
        Some(dir) => {
            let vize_dir = dir.join("vize");

            if vize_dir.exists() {
                if let Err(e) = std::fs::remove_dir_all(&vize_dir) {
                    eprintln!("Failed to remove extension: {}", e);
                    std::process::exit(1);
                }
                println!("✓ Vize extension removed from Zed");
            } else {
                println!("Extension not installed");
            }
        }
        None => {
            eprintln!("Could not find Zed extensions directory");
        }
    }
}

fn zed_status() {
    let extensions_dir = get_zed_extensions_dir();

    match extensions_dir {
        Some(dir) => {
            let vize_dir = dir.join("vize");
            if vize_dir.exists() {
                println!("✓ Vize extension is installed in Zed");
                println!("  Location: {}", vize_dir.display());
            } else {
                println!("✗ Vize extension is not installed in Zed");
            }
        }
        None => {
            println!("✗ Zed extensions directory not found");
        }
    }
}

fn get_zed_extensions_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().map(|d| d.join("Zed/extensions/installed"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::data_dir().map(|d| d.join("zed/extensions/installed"))
    }

    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir().map(|d| d.join("Zed/extensions/installed"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

#[cfg(test)]
mod tests;
