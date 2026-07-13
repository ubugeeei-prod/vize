//! Multi-source Atlas preparation for the production build command.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
    time::Duration,
    time::Instant,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileRequest, SfcCompileSettings, SfcCroquisMode,
    SfcCroquisSettings, SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
};
use vize_atlas::{Compilation, CompilationSnapshot, SourceId};
use vize_carton::profiler::global_profiler;
use vize_carton::{String, ToCompactString, cstr, profile};
use vize_relief::VueDialectInput;

use crate::commands::build::config::{CompileError, ErrorPhase};
use crate::commands::build::{ScriptExtension, config::CompileStats};

use super::settings::CompileFileSettings;

/// One source already read into the shared immutable compilation snapshot.
pub(super) struct PreparedSource {
    pub(super) path: PathBuf,
    pub(super) source: SourceId,
    pub(super) read_time: Duration,
}

/// Sources, providers, and typed requests captured once for parallel workers.
pub(super) struct BuildArtifactGraph {
    pub(super) snapshot: CompilationSnapshot,
    pub(super) sources: Vec<PreparedSource>,
}

impl BuildArtifactGraph {
    pub(super) fn prepare(
        paths: &[PathBuf],
        settings: CompileFileSettings,
        stats: &CompileStats,
    ) -> Result<(Self, Vec<CompileError>), String> {
        let reads: Vec<_> = paths
            .par_iter()
            .map(|path| read_source(path.clone()))
            .collect();
        let mut compilation = Compilation::new();
        vize_atelier_sfc::register_atlas_providers(&mut compilation)
            .map_err(|error| cstr!("failed to register SFC artifact providers: {error}"))?;
        vize_atelier_dom::register_atlas_provider(&mut compilation)
            .map_err(|error| cstr!("failed to register DOM backend: {error}"))?;
        vize_atelier_ssr::register_atlas_provider(&mut compilation)
            .map_err(|error| cstr!("failed to register SSR backend: {error}"))?;
        vize_atelier_vapor::register_atlas_provider(&mut compilation)
            .map_err(|error| cstr!("failed to register Vapor backend: {error}"))?;
        compilation
            .set_input::<VueDialectInput>(settings.dialect)
            .map_err(|error| cstr!("failed to install Vue dialect input: {error}"))?;
        let mut requests = SfcCompileSettings::default();
        let mut croquis = SfcCroquisSettings::default();
        let mut sources = Vec::with_capacity(paths.len());
        let mut errors = Vec::new();

        for read in reads {
            match read {
                Ok((path, text, read_time)) => {
                    stats.total_bytes.fetch_add(text.len(), Ordering::Relaxed);
                    let mode = if settings.dialect.is_legacy() {
                        SfcCroquisMode::LegacyVue2
                    } else if text.contains("<script") && !text.contains("<script setup") {
                        SfcCroquisMode::OptionsApi
                    } else {
                        SfcCroquisMode::Full
                    };
                    let source = compilation
                        .add_source(path.to_string_lossy().into_owned(), text)
                        .map_err(|error| cstr!("failed to register build source: {error}"))?;
                    requests.insert(source, compile_request(&path, settings));
                    croquis.insert(source, mode);
                    croquis.insert_resolved_filename(source, path.to_string_lossy().into_owned());
                    sources.push(PreparedSource {
                        path,
                        source,
                        read_time,
                    });
                }
                Err(error) => errors.push(error),
            }
        }
        requests
            .install(&mut compilation)
            .map_err(|error| cstr!("failed to install SFC compile settings: {error}"))?;
        croquis
            .install(&mut compilation)
            .map_err(|error| cstr!("failed to install SFC semantic settings: {error}"))?;

        Ok((
            Self {
                snapshot: compilation.snapshot(),
                sources,
            },
            errors,
        ))
    }
}

fn read_source(path: PathBuf) -> Result<(PathBuf, String, Duration), CompileError> {
    let started = Instant::now();
    match profile!("cli.build.file.read", fs::read_to_string(&path)) {
        Ok(source) => {
            global_profiler().record_fs_read_to_string(source.len());
            Ok((path, source.into(), started.elapsed()))
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            Err(CompileError {
                path,
                error: cstr!("Failed to read file: {}", error),
                phase: ErrorPhase::Read,
            })
        }
    }
}

fn compile_request(path: &Path, settings: CompileFileSettings) -> SfcCompileRequest {
    let filename: String = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("anonymous.vue")
        .into();
    let source_id = path.to_string_lossy().as_ref().to_compact_string();
    let is_ts = matches!(settings.script_ext, ScriptExtension::Preserve);
    SfcCompileRequest::new(
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: filename.clone(),
                ..Default::default()
            },
            script: ScriptCompileOptions {
                id: Some(source_id),
                is_ts,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename.clone()),
                ssr: settings.ssr,
                is_ts,
                custom_renderer: settings.custom_renderer,
                compiler_options: Some(vize_atelier_dom::DomCompilerOptions {
                    experimental_in_tag_comments: settings.experimental_in_tag_comments,
                    experimental_patterned_template: settings.experimental_patterned_template,
                    ..Default::default()
                }),
                dialect: settings.dialect,
                ..Default::default()
            },
            style: StyleCompileOptions {
                id: filename,
                ..Default::default()
            },
            vapor: settings.vapor,
            scope_id: None,
        },
        settings.template_syntax,
    )
    .with_inferred_scoped_from_descriptor()
}

#[cfg(test)]
mod tests {
    use vize_atelier_sfc::{SfcCompileProduct, SfcDescriptorProduct};
    use vize_carton::config::VueVersion;
    use vize_relief::TemplateSyntaxMode;
    use vize_relief::{ReliefProduct, TransformedReliefProduct};
    use vize_rendu::RenduProduct;

    use super::*;

    #[test]
    fn production_build_reuses_one_descriptor_parse_in_the_compile_query() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("App.vue");
        fs::write(
            &path,
            "<template><main>{{ message }}</main></template><style scoped>main{color:red}</style>",
        )
        .unwrap();
        let stats = CompileStats::new(1);
        let settings = CompileFileSettings {
            ssr: false,
            vapor: false,
            custom_renderer: false,
            template_syntax: TemplateSyntaxMode::Standard,
            experimental_in_tag_comments: false,
            experimental_patterned_template: false,
            dialect: VueVersion::V3,
            script_ext: ScriptExtension::Downcompile,
            record_profile_totals: false,
        };
        let (graph, errors) = BuildArtifactGraph::prepare(&[path], settings, &stats).unwrap();
        assert!(errors.is_empty());
        let source = graph.sources[0].source;
        let mut session = graph.snapshot.query_session();

        let descriptor = session.query::<SfcDescriptorProduct>(source).unwrap();
        assert!(descriptor.trace().executed::<SfcDescriptorProduct>());
        let compiled = session.query::<SfcCompileProduct>(source).unwrap();

        assert!(compiled.trace().cache_hit::<SfcDescriptorProduct>());
        assert!(!compiled.trace().executed::<SfcDescriptorProduct>());
        assert!(compiled.plan().contains::<ReliefProduct>());
        assert!(compiled.plan().contains::<TransformedReliefProduct>());
        assert!(compiled.plan().contains::<RenduProduct>());
        assert!(compiled.value().code.contains("message"));
    }
}
