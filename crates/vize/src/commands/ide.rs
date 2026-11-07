//! IDE command - Editor integration and LSP server
//!
//! This command provides:
//! - LSP server (default, alias for `vize lsp`)
//! - Editor extension installation for VSCode and Zed

use clap::{Args, Subcommand};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use vize_l0::{String, cstr};

mod zed_archive;
#[cfg(test)]
use zed_archive::validate_zed_archive_member;
use zed_archive::{MAX_ZED_ARCHIVE_BYTES, unpack_zed_extension_archive};
mod zed_installation;
use zed_installation::{get_zed_extensions_dir, zed_dev_extension_dir, zed_status, zed_uninstall};

mod editor_files;

use crate::commands::lsp::{LspTransport, transport_from_port};
use editor_files::{copy_dir_all, find_vscode_vsix, find_zed_extension_source};

const VSCODE_EXTENSION_ID: &str = "ubugeeei.vize";
const LEGACY_VSCODE_EXTENSION_ID: &str = "vize.vize";
const RELEASE_REPOSITORY: &str = "https://github.com/ubugeeei-prod/vize";
const ZED_RELEASE_ASSET: &str = "zed-vize-extension.tar.gz";

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
            if Path::new("editors/vscode").is_dir() {
                println!("VSIX not found, building from source...");
            }
            if Path::new("editors/vscode").is_dir() && build_vscode_extension() {
                if let Some(path) = find_vscode_vsix() {
                    install_vsix(&path);
                } else {
                    eprintln!("Failed to find built VSIX");
                    std::process::exit(1);
                }
            } else {
                println!(
                    "Installing published extension version {}...",
                    env!("CARGO_PKG_VERSION")
                );
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
    let output = Command::new("code")
        .args(["--list-extensions", "--show-versions"])
        .output();

    match output {
        Ok(out) => {
            #[expect(clippy::disallowed_types, reason = "from_utf8_lossy yields std Cow")]
            let extensions = std::string::String::from_utf8_lossy(&out.stdout);
            match vscode_extension_version(&extensions) {
                Some(Some(version)) => {
                    println!("✓ Vize extension {version} is installed in VSCode");
                    if version != env!("CARGO_PKG_VERSION") {
                        println!(
                            "⚠ Installed extension {version} does not match Vize CLI {}",
                            env!("CARGO_PKG_VERSION")
                        );
                    }
                }
                Some(None) => {
                    println!("✓ Vize extension is installed in VSCode (version unavailable)");
                }
                None => println!("✗ Vize extension is not installed in VSCode"),
            }
        }
        Err(e) => {
            eprintln!("Failed to check VSCode extensions: {}", e);
            eprintln!("Make sure VSCode is installed and 'code' is in your PATH");
        }
    }
}

#[cfg(test)]
fn vscode_extension_is_installed(extensions: &str) -> bool {
    vscode_extension_version(extensions).is_some()
}

fn vscode_extension_version(extensions: &str) -> Option<Option<&str>> {
    extensions.lines().find_map(|line| {
        let extension = line.trim();
        let (id, version) = extension
            .rsplit_once('@')
            .map_or((extension, None), |(id, version)| (id, Some(version)));
        (id.eq_ignore_ascii_case(VSCODE_EXTENSION_ID)
            || id.eq_ignore_ascii_case(LEGACY_VSCODE_EXTENSION_ID))
        .then_some(version)
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
    let extension = vscode_extension_spec(env!("CARGO_PKG_VERSION"));
    let status = Command::new("code")
        .args(["--install-extension", extension.as_str(), "--force"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("✓ Vize extension installed successfully!");
            println!("  Restart VSCode to activate the extension.");
        }
        Ok(_) => {
            eprintln!("Failed to install published extension: {}", extension);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run 'code' command: {}", e);
            eprintln!("Make sure VSCode is installed and 'code' is in your PATH");
            std::process::exit(1);
        }
    }
}

fn vscode_extension_spec(version: &str) -> String {
    cstr!("{VSCODE_EXTENSION_ID}@{version}")
}

// =============================================================================
// Zed Extension
// =============================================================================

fn zed_install() {
    println!("Installing Vize Zed extension...");

    let extensions_dir = get_zed_extensions_dir();

    match extensions_dir {
        Some(dir) => {
            let vize_dir = zed_dev_extension_dir(&dir);
            let downloaded;
            let source_dir = if let Some(source_dir) = find_zed_extension_source() {
                source_dir
            } else {
                println!(
                    "Downloading the matching Zed extension from Vize {}...",
                    env!("CARGO_PKG_VERSION")
                );
                downloaded = match download_zed_extension(env!("CARGO_PKG_VERSION")) {
                    Ok(downloaded) => downloaded,
                    Err(error) => {
                        eprintln!("Failed to download the Zed extension: {error}");
                        eprintln!(
                            "Download {} and select its extracted zed directory with 'Zed: Install Dev Extension'.",
                            zed_release_asset_url(env!("CARGO_PKG_VERSION"))
                        );
                        std::process::exit(1);
                    }
                };
                downloaded.path().join("zed")
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

            println!(
                "✓ Vize extension source prepared at: {}",
                vize_dir.display()
            );
            println!("  In Zed, run 'Zed: Install Dev Extension' and select this directory.");
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

fn zed_release_asset_url(version: &str) -> String {
    cstr!("{RELEASE_REPOSITORY}/releases/download/v{version}/{ZED_RELEASE_ASSET}")
}

fn download_zed_extension(version: &str) -> Result<tempfile::TempDir, String> {
    let temporary = tempfile::Builder::new()
        .prefix("vize-zed-extension-")
        .tempdir()
        .map_err(|error| cstr!("could not create a temporary directory: {error}"))?;
    let archive = temporary.path().join(ZED_RELEASE_ASSET);
    let source = temporary.path().join("zed");

    let curl = std::env::var_os("VIZE_IDE_CURL").unwrap_or_else(|| OsString::from("curl"));
    let url = zed_release_asset_url(version);
    let maximum_archive_bytes = cstr!("{MAX_ZED_ARCHIVE_BYTES}");
    let download = Command::new(curl)
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-redirs",
            "5",
        ])
        .arg("--max-filesize")
        .arg(maximum_archive_bytes.as_str())
        .arg("--output")
        .arg(&archive)
        .arg(url.as_str())
        .status()
        .map_err(|error| cstr!("could not run curl: {error}"))?;
    if !download.success() {
        return Err(cstr!("curl exited with {download} while fetching {url}"));
    }

    let metadata = std::fs::metadata(&archive)
        .map_err(|error| cstr!("could not inspect the downloaded extension: {error}"))?;
    if !metadata.is_file() || metadata.len() > MAX_ZED_ARCHIVE_BYTES {
        return Err(cstr!(
            "downloaded extension is not a regular archive within the {MAX_ZED_ARCHIVE_BYTES}-byte limit"
        ));
    }
    let bytes = std::fs::read(&archive)
        .map_err(|error| cstr!("could not read the downloaded extension: {error}"))?;
    unpack_zed_extension_archive(&bytes, temporary.path())?;
    if !std::fs::symlink_metadata(source.join("extension.toml"))
        .is_ok_and(|metadata| metadata.file_type().is_file())
    {
        return Err(cstr!(
            "release archive does not contain a regular zed/extension.toml"
        ));
    }

    Ok(temporary)
}

#[cfg(test)]
mod tests;
