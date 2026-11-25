//! Format one source text the way `vize fmt --write` formats a file.
//!
//! The project configuration, the per-file formatter selection and the
//! per-file policies are the command's own, so a host formatting generated
//! code gets the bytes `vize fmt` would write for a file of that name.

use super::{format_source::format_file_source, lint_policy, patterns};
use crate::config::{self, LoadedFormatterSnapshot};
use std::path::{Path, PathBuf};
use vize_glyph::{Allocator, FormatOptions, FormatResult, ImportSortOptions, VueVersion};
use vize_l0::{String, cstr};

/// Where the formatter settings for a source come from.
#[derive(Debug, Clone, Copy)]
pub enum FormatConfig<'a> {
    /// The configuration `vize fmt` run from the file's directory selects.
    Discover,
    /// An explicit Vite config file or project directory, as `--config` takes.
    Path(&'a Path),
    /// Formatter defaults only, as `--no-config` uses.
    Defaults,
}

/// The resolved settings a caller may adjust before formatting.
#[non_exhaustive]
pub struct FormatSettings {
    pub options: FormatOptions,
    pub vue_version: VueVersion,
    pub sort_imports: Option<ImportSortOptions>,
}

/// Format `source` as the contents of `filename`.
///
/// `filename` selects the formatter by extension, the configuration by its
/// directory and the per-file whitespace and lint policies; the file itself
/// is never read or written. `adjust` applies caller overrides on top of the
/// configured settings, as command-line flags do.
pub fn format_source(
    source: &str,
    filename: &Path,
    config: FormatConfig<'_>,
    adjust: impl FnOnce(&mut FormatSettings) -> Result<(), String>,
) -> Result<FormatResult, String> {
    let path = absolute(filename);
    if !patterns::is_format_target(&path) {
        return Err(cstr!(
            "{}: unsupported file type; expected {}",
            filename.display(),
            patterns::FORMAT_EXTENSIONS_DISPLAY
        ));
    }
    super::file_support::validate(&path)
        .map_err(|message| cstr!("{}: {message}", filename.display()))?;
    let snapshot = load_snapshot(&path, config)
        .map_err(|error| cstr!("Invalid formatter configuration: {error}"))?;
    let sort_imports = vize_glyph::resolve_sort_imports(snapshot.sort_imports.as_ref())
        .map_err(|error| cstr!("Invalid formatter configuration: {error}"))?;
    let mut options = FormatOptions::from_profile(&snapshot.formatter_profile)
        .map_err(|error| cstr!("Invalid formatter configuration: {error}"))?;
    // The command skips this pass only when it reports without writing.
    options.skip_script_stabilization = false;
    let mut settings = FormatSettings {
        options,
        vue_version: snapshot.loaded.features.vue_version.unwrap_or_default(),
        sort_imports,
    };
    adjust(&mut settings)?;
    let config_dir = config_dir(&snapshot);
    lint_policy::validate_selected_files(
        std::slice::from_ref(&path),
        &snapshot,
        &settings.options,
        &config_dir,
    )
    .map_err(|error| cstr!("Invalid formatter configuration: {error}"))?;
    let preserve_text = config::matcher::CompilerWhitespacePlan::new(
        snapshot.compiler_whitespace,
        &snapshot.compiler_entries,
        &config_dir,
    )
    .preserves_text_for(&path);
    let allocator = Allocator::with_capacity(source.len() * 2);
    format_file_source(
        &path,
        source,
        &settings.options,
        &allocator,
        (
            settings.vue_version,
            settings.sort_imports.as_ref(),
            preserve_text,
        ),
    )
    .map_err(|error| cstr!("{}: {error}", filename.display()))
}

/// The directory the command resolves configured paths against.
pub(super) fn config_dir(snapshot: &LoadedFormatterSnapshot) -> PathBuf {
    snapshot
        .project_root
        .clone()
        .or_else(|| {
            snapshot
                .loaded
                .source_path
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Concurrent first calls would each start a Node evaluation of the same
/// config; one at a time, later calls reuse the validated stored evaluation.
static CONFIG_LOAD: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn load_snapshot(path: &Path, config: FormatConfig<'_>) -> Result<LoadedFormatterSnapshot, String> {
    if matches!(config, FormatConfig::Defaults) {
        return Ok(LoadedFormatterSnapshot::default());
    }
    // `vize fmt` reuses evaluations whose recorded inputs are unchanged; a
    // host formatting file after file needs that even more than one run does.
    config::enable_config_evaluation_cache();
    let _serialized = CONFIG_LOAD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match config {
        FormatConfig::Defaults => Ok(LoadedFormatterSnapshot::default()),
        FormatConfig::Path(config) => {
            config::try_load_formatter_snapshot(Some(config)).map_err(String::from)
        }
        FormatConfig::Discover => {
            // A generator may format a file whose directory does not exist yet;
            // the nearest existing ancestor governs it.
            let directory = path
                .ancestors()
                .skip(1)
                .find(|directory| directory.is_dir())
                .unwrap_or(path);
            config::try_discover_formatter_snapshot(directory).map_err(String::from)
        }
    }
}

fn absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|directory| directory.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}
