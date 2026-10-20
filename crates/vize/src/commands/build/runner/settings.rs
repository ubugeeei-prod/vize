//! Per-file compile settings and template-syntax mapping for the build command.

use std::path::{Path, PathBuf};
use vize_atelier_core::{TemplateSyntaxMode, WhitespaceStrategy};
use vize_l0::String;
use vize_l0::config::{
    CompilerConfigEntry, ConfigExperimentalVueFlags, ConfigFeatureFlags, VueVersion,
};
use vize_l0::hash::hash_bytes;

use crate::commands::build::{BuildArgs, ScriptExtension};
use crate::commands::davinci_ice;
use crate::lint_plan::matcher::LintPlanScope;

pub(super) struct BuildConfigSettings {
    pub(super) project_root: Option<PathBuf>,
    pub(super) compiler_template_syntax: Option<&'static str>,
    pub(super) compiler_whitespace: Option<&'static str>,
    pub(super) features: ConfigFeatureFlags,
    pub(super) experimental_vue: ConfigExperimentalVueFlags,
    pub(super) ssr: Option<bool>,
    pub(super) vapor: Option<bool>,
    pub(super) custom_elements: Vec<String>,
    pub(super) dialect: Option<VueVersion>,
    pub(super) host_compiler: Option<bool>,
    compiler_entries: Vec<CompilerConfigEntry>,
    config_dir: PathBuf,
}

pub(super) fn load_build_config(no_config: bool, config: Option<&Path>) -> BuildConfigSettings {
    if no_config {
        return BuildConfigSettings {
            project_root: None,
            compiler_template_syntax: None,
            compiler_whitespace: None,
            features: ConfigFeatureFlags::default(),
            experimental_vue: ConfigExperimentalVueFlags::default(),
            ssr: None,
            vapor: None,
            custom_elements: Vec::new(),
            dialect: None,
            host_compiler: None,
            compiler_entries: Vec::new(),
            config_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        };
    }
    let loaded =
        crate::config::try_load_project_config_with_source(config).unwrap_or_else(|error| {
            eprintln!("\x1b[31mError:\x1b[0m {error}");
            std::process::exit(1);
        });
    let document = loaded.document;
    let compiler_entries = document.compiler_entries();
    let config_dir = loaded
        .project_root
        .clone()
        .or_else(|| {
            loaded
                .source_path
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let features = document.clone().into_config_and_features().1;
    BuildConfigSettings {
        project_root: loaded.project_root,
        compiler_template_syntax: document.compiler_template_syntax(),
        compiler_whitespace: document.compiler_whitespace(),
        experimental_vue: document.experimental_vue_flags(),
        ssr: document.compiler_ssr(),
        vapor: document.compiler_vapor(),
        dialect: features.vue_version,
        features,
        host_compiler: document.compiler_host_compiler(),
        custom_elements: document.into_compiler_custom_elements(),
        compiler_entries,
        config_dir,
    }
}

struct CompilerScope {
    matcher: LintPlanScope,
    template_syntax: Option<&'static str>,
    whitespace: Option<String>,
    vapor: Option<bool>,
    custom_elements: Vec<String>,
}

pub(super) struct ResolvedCompilerFileSettings<'a> {
    pub(super) template_syntax: TemplateSyntaxMode,
    pub(super) whitespace: WhitespaceStrategy,
    pub(super) legacy_line_breaks: bool,
    pub(super) vue2_preserve: bool,
    pub(super) vapor: bool,
    pub(super) custom_elements: &'a [String],
}

/// Compile a single `.vue` file with profiling information.
pub(super) struct CompileFileSettings {
    pub(super) ssr: bool,
    pub(super) vapor: bool,
    pub(super) custom_renderer: bool,
    pub(super) custom_elements: Vec<String>,
    pub(super) template_syntax: TemplateSyntaxMode,
    pub(super) whitespace: WhitespaceStrategy,
    pub(super) legacy_line_breaks: bool,
    /// Vue 2.7 `preserve` text rules (#8259); implies `whitespace: Preserve`.
    pub(super) vue2_preserve: bool,
    pub(super) experimental_in_tag_comments: bool,
    pub(super) experimental_patterned_template: bool,
    pub(super) experimental_self_component: bool,
    /// Vue dialect from `vue.version`; defaults to [`VueVersion::V3`] and is
    /// threaded into each file's compile options.
    pub(super) dialect: VueVersion,
    pub(super) script_ext: ScriptExtension,
    pub(super) record_profile_totals: bool,
    pub(super) davinci: DavinciBuildSettings,
    /// Project-preprocessed `<style lang="scss|…">` blocks (#8905).
    pub(super) preprocessed_styles: super::preprocess::PreprocessedStyles,
    compiler_scopes: Vec<CompilerScope>,
    template_syntax_from_cli: bool,
    vapor_from_cli: bool,
    custom_elements_from_cli: bool,
}

/// The build's slice of the Davinci ICE machinery (P2-13): the plan the
/// compile path is described by, where a `repro.folio` goes, the per-pass
/// dump controls, and the TS-23 panic injection.
pub(super) struct DavinciBuildSettings {
    /// Opt-in destination for the stages from this exact product compile.
    pub(super) dump_dir: Option<PathBuf>,
    pub(super) dump_after_change: bool,
    /// The selected backend's legacy plan (`legacy_plan::{DOM,SSR,VAPOR}`)
    /// in the canonical pipeline-grammar spelling.
    pub(super) plan_string: String,
    /// P2-3 budget observer output for the selected plan. This is named
    /// "planned groups" at reporting sites because the real compile path still
    /// runs legacy stages until the L2 backend becomes a production dependency.
    pub(super) planned_groups: u32,
    /// Pass entries described by the selected plan.
    pub(super) planned_passes: u32,
    /// The plan's stage name - the attribution for a panic the driver did
    /// not see (an unattributable real-compile ICE).
    pub(super) stage: &'static str,
    /// Mode name recorded in `[repro.config]`.
    pub(super) mode: &'static str,
    /// Parsed `--davinci-inject-panic` spec: `(file stem, injection)`.
    pub(super) inject: Option<(String, davinci_ice::Injection)>,
    /// Where a failing file's `repro.folio` is written (the output dir).
    pub(super) repro_dir: PathBuf,
}

impl DavinciBuildSettings {
    /// The injection targeting `path`, if any (its trigger, when set, is
    /// checked against the source by the caller).
    pub(super) fn injection_for(&self, path: &Path) -> Option<&davinci_ice::Injection> {
        let (stem, injection) = self.inject.as_ref()?;
        (path.file_stem().and_then(|name| name.to_str()) == Some(stem.as_str()))
            .then_some(injection)
    }
}

impl CompileFileSettings {
    /// Resolve per-file compile settings from CLI arguments and config file values.
    pub(super) fn resolve(args: &BuildArgs, build_config: BuildConfigSettings) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let compiler_scopes = build_config
            .compiler_entries
            .iter()
            .map(|entry| CompilerScope {
                matcher: LintPlanScope::new(
                    entry.base_path.as_deref(),
                    entry.files.as_deref(),
                    &entry.ignores,
                    &build_config.config_dir,
                    &cwd,
                ),
                template_syntax: entry.template_syntax,
                whitespace: entry.whitespace.clone(),
                vapor: entry.vapor,
                custom_elements: entry.custom_elements.clone(),
            })
            .collect();
        let ssr = args.ssr || build_config.ssr.unwrap_or(false);
        let vapor = args.vapor || build_config.vapor.unwrap_or(false);
        let (plan, mode) = davinci_ice::compile_plan(ssr, vapor);
        let budget = davinci_ice::plan_budget(plan);
        let inject = args.davinci_inject_panic.as_deref().map(|spec| {
            davinci_ice::parse_inject_spec(spec, plan).unwrap_or_else(|error| {
                eprintln!("\x1b[31mError:\x1b[0m --davinci-inject-panic: {error}");
                std::process::exit(1);
            })
        });
        // Create the requested output root before compiling. Each successful
        // file writes a feed with the backend selection from that same run.
        if let Some(dir) = args.dump_dir.as_deref()
            && let Err(error) = std::fs::create_dir_all(dir)
        {
            eprintln!(
                "\x1b[31mError:\x1b[0m --dump-dir: cannot create {}: {error}",
                dir.display()
            );
            std::process::exit(1);
        }
        Self {
            ssr,
            vapor,
            custom_renderer: args.custom_renderer,
            custom_elements: resolve_custom_elements(args, build_config.custom_elements),
            template_syntax: args
                .template_syntax
                .map(Into::into)
                .unwrap_or_else(|| template_syntax_mode(build_config.compiler_template_syntax)),
            whitespace: if matches!(
                build_config.compiler_whitespace,
                Some("preserve" | "vue2-preserve")
            ) {
                WhitespaceStrategy::Preserve
            } else {
                WhitespaceStrategy::Condense
            },
            legacy_line_breaks: build_config.compiler_whitespace == Some("vue2-line-breaks"),
            vue2_preserve: build_config.compiler_whitespace == Some("vue2-preserve"),
            experimental_in_tag_comments: build_config.features.experimental_in_tag_comments,
            experimental_patterned_template: build_config.features.experimental_patterned_template,
            experimental_self_component: build_config.experimental_vue.self_component,
            dialect: build_config.dialect.unwrap_or_default(),
            script_ext: args.script_ext,
            record_profile_totals: args.profile,
            davinci: DavinciBuildSettings {
                dump_dir: args.dump_dir.clone(),
                dump_after_change: args.dump_after_change,
                plan_string: davinci_ice::plan_string(plan),
                planned_groups: budget.walks,
                planned_passes: budget.passes,
                stage: plan.stage,
                mode,
                inject,
                repro_dir: args.output.clone(),
            },
            preprocessed_styles: super::preprocess::PreprocessedStyles::default(),
            compiler_scopes,
            template_syntax_from_cli: args.template_syntax.is_some(),
            vapor_from_cli: args.vapor,
            custom_elements_from_cli: !args.custom_elements.is_empty(),
        }
    }

    pub(super) fn compiler_for_path(&self, path: &Path) -> ResolvedCompilerFileSettings<'_> {
        let mut resolved = ResolvedCompilerFileSettings {
            template_syntax: self.template_syntax,
            whitespace: self.whitespace,
            legacy_line_breaks: self.legacy_line_breaks,
            vue2_preserve: self.vue2_preserve,
            vapor: self.vapor,
            custom_elements: &self.custom_elements,
        };
        for scope in &self.compiler_scopes {
            if !scope.matcher.matches(path) {
                continue;
            }
            if !self.template_syntax_from_cli
                && let Some(template_syntax) = scope.template_syntax
            {
                resolved.template_syntax = template_syntax_mode(Some(template_syntax));
            }
            if let Some(whitespace) = scoped_whitespace(scope.whitespace.as_deref()) {
                resolved.whitespace = whitespace.0;
                resolved.legacy_line_breaks = whitespace.1;
                resolved.vue2_preserve = whitespace.2;
            }
            if !self.vapor_from_cli
                && let Some(vapor) = scope.vapor
            {
                resolved.vapor = vapor;
            }
            if !self.custom_elements_from_cli && !scope.custom_elements.is_empty() {
                resolved.custom_elements = &scope.custom_elements;
            }
        }
        resolved
    }

    /// Packs every compile option that can change stats output into a tiny cache key.
    ///
    /// `record_profile_totals` is intentionally excluded: enabling profiling changes
    /// accounting side effects, not parse/compile output. Script extension is included
    /// because preserving TypeScript can change generated code size. The dialect is
    /// included because it can change parse/compile output on legacy-capable builds.
    pub(super) fn cache_bits(&self, path: &Path) -> u16 {
        let compiler = self.compiler_for_path(path);
        u16::from(self.ssr)
            | (u16::from(compiler.vapor) << 1)
            | (u16::from(self.custom_renderer) << 2)
            | (u16::from(template_syntax_bits(compiler.template_syntax)) << 3)
            | match self.script_ext {
                ScriptExtension::Preserve => 1 << 5,
                ScriptExtension::Downcompile => 0,
            }
            | (u16::from(dialect_bits(self.dialect)) << 6)
            | (u16::from(self.experimental_in_tag_comments) << 9)
            | (u16::from(self.experimental_patterned_template) << 10)
            | (u16::from(self.experimental_self_component) << 11)
            | (u16::from(matches!(compiler.whitespace, WhitespaceStrategy::Preserve)) << 12)
            | (u16::from(compiler.legacy_line_breaks) << 13)
            | (u16::from(compiler.vue2_preserve) << 14)
    }

    /// Scope the configured template whitespace mode around one compile.
    pub(super) fn with_whitespace<R>(&self, path: &Path, compile: impl FnOnce() -> R) -> R {
        let compiler = self.compiler_for_path(path);
        if compiler.vue2_preserve {
            vize_atelier_core::parser::with_vue2_preserve_whitespace(compile)
        } else {
            vize_atelier_core::parser::with_whitespace_mode(
                compiler.whitespace,
                compiler.legacy_line_breaks,
                compile,
            )
        }
    }

    pub(super) fn custom_elements_hash(&self, path: &Path) -> u64 {
        let custom_elements = self.compiler_for_path(path).custom_elements;
        if custom_elements.is_empty() {
            return 0;
        }

        let byte_len = custom_elements
            .iter()
            .map(|pattern| u64::BITS as usize / 8 + pattern.len())
            .sum();
        let mut bytes = Vec::with_capacity(byte_len);
        for pattern in custom_elements {
            bytes.extend_from_slice(&(pattern.len() as u64).to_le_bytes());
            bytes.extend_from_slice(pattern.as_bytes());
        }
        hash_bytes(&bytes)
    }
}

fn scoped_whitespace(value: Option<&str>) -> Option<(WhitespaceStrategy, bool, bool)> {
    match value {
        Some("preserve") => Some((WhitespaceStrategy::Preserve, false, false)),
        Some("condense") => Some((WhitespaceStrategy::Condense, false, false)),
        Some("vue2-line-breaks") => Some((WhitespaceStrategy::Condense, true, false)),
        Some("vue2-preserve") => Some((WhitespaceStrategy::Preserve, false, true)),
        _ => None,
    }
}

fn resolve_custom_elements(args: &BuildArgs, from_config: Vec<String>) -> Vec<String> {
    if args.custom_elements.is_empty() {
        from_config
    } else {
        args.custom_elements
            .iter()
            .map(|pattern| String::from(pattern.as_str()))
            .collect()
    }
}

fn dialect_bits(dialect: VueVersion) -> u8 {
    match dialect {
        VueVersion::V3 => 0,
        VueVersion::V2_7 => 1,
        VueVersion::V2 => 2,
        VueVersion::V1 => 3,
        VueVersion::V0_11 => 4,
        VueVersion::V0_10 => 5,
    }
}

fn template_syntax_bits(template_syntax: TemplateSyntaxMode) -> u8 {
    match template_syntax {
        TemplateSyntaxMode::Standard => 0,
        TemplateSyntaxMode::Strict => 1,
        TemplateSyntaxMode::Quirks => 2,
        _ => 3,
    }
}

pub(super) fn template_syntax_mode(template_syntax: Option<&str>) -> TemplateSyntaxMode {
    match template_syntax {
        Some("strict") => TemplateSyntaxMode::Strict,
        Some("quirks") => TemplateSyntaxMode::Quirks,
        Some("standard") | None => TemplateSyntaxMode::Standard,
        Some(_) => TemplateSyntaxMode::Standard,
    }
}

#[cfg(test)]
mod tests {
    //! Stats cache must distinguish actual output-affecting whitespace modes.

    use super::super::{cache::StatsCompileCache, compile_stats::compile_file_stats_with_cache};
    use super::{BuildConfigSettings, CompileFileSettings, load_build_config};
    use crate::commands::build::{BuildArgs, config::CompileStats};
    use vize_atelier_core::{WhitespaceStrategy, parser::with_whitespace_mode};
    use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
    use vize_l0::config::{CompilerConfigEntry, ConfigExperimentalVueFlags, ConfigFeatureFlags};

    #[test]
    fn entry_compiler_settings_apply_only_to_matching_files() {
        let project = tempfile::tempdir().unwrap();
        let legacy = project.path().join("src/legacy/Old.vue");
        let modern = project.path().join("src/New.vue");
        let config = BuildConfigSettings {
            project_root: Some(project.path().to_path_buf()),
            compiler_template_syntax: None,
            compiler_whitespace: Some("preserve"),
            features: ConfigFeatureFlags::default(),
            experimental_vue: ConfigExperimentalVueFlags::default(),
            ssr: None,
            vapor: None,
            custom_elements: Vec::new(),
            dialect: None,
            host_compiler: None,
            compiler_entries: vec![CompilerConfigEntry {
                base_path: None,
                files: Some(vec!["src/legacy/**".into()]),
                ignores: Vec::new(),
                template_syntax: Some("strict"),
                whitespace: Some("condense".into()),
                vapor: Some(true),
                custom_elements: vec!["legacy-*".into()],
            }],
            config_dir: project.path().to_path_buf(),
        };
        let settings = CompileFileSettings::resolve(&BuildArgs::default(), config);
        let legacy_settings = settings.compiler_for_path(&legacy);
        assert_eq!(legacy_settings.whitespace, WhitespaceStrategy::Condense);
        assert_eq!(
            legacy_settings.template_syntax,
            vize_atelier_core::TemplateSyntaxMode::Strict
        );
        assert!(legacy_settings.vapor);
        assert_eq!(legacy_settings.custom_elements, ["legacy-*"]);
        let modern_settings = settings.compiler_for_path(&modern);
        assert_eq!(modern_settings.whitespace, WhitespaceStrategy::Preserve);
        assert_eq!(
            modern_settings.template_syntax,
            vize_atelier_core::TemplateSyntaxMode::Standard
        );
        assert!(!modern_settings.vapor);
        assert!(modern_settings.custom_elements.is_empty());
        assert_ne!(settings.cache_bits(&legacy), settings.cache_bits(&modern));

        let source = include_str!(
            "../../../../../../tests/_fixtures/differential/compiler/cli-whitespace-7880/App.vue.txt"
        );
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let compile = |path| {
            settings
                .with_whitespace(path, || {
                    compile_sfc(&descriptor, SfcCompileOptions::default())
                })
                .unwrap()
                .code
        };
        assert_ne!(compile(&modern), compile(&legacy));
    }

    #[test]
    fn shared_stats_cache_keeps_whitespace_modes_and_restores_thread_local_parser_state() {
        let source = include_str!(
            "../../../../../../tests/_fixtures/differential/compiler/cli-whitespace-7880/App.vue.txt"
        );
        let project = tempfile::tempdir().unwrap();
        let path = project.path().join("App.vue");
        std::fs::write(&path, source).unwrap();
        let cache = StatsCompileCache::default();
        let stats = CompileStats::new(7);
        let mut lengths = Vec::new();
        for (mode, hit, entries) in [
            (None, false, 1),
            (Some("preserve"), false, 2),
            (Some("condense"), true, 2),
            (Some("vue2-line-breaks"), false, 3),
            (Some("preserve"), true, 3),
            (None, true, 3),
            (Some("vue2-line-breaks"), true, 3),
        ] {
            let mut config = load_build_config(true, None);
            config.compiler_whitespace = mode;
            let settings = CompileFileSettings::resolve(&BuildArgs::default(), config);
            let (actual, profile) =
                compile_file_stats_with_cache(&path, &settings, &stats, &cache).unwrap();
            assert_eq!(cache.entries.lock().unwrap().len(), entries);
            if hit {
                assert_eq!(profile.parse_time, std::time::Duration::ZERO);
                assert_eq!(profile.compile_time, std::time::Duration::ZERO);
            }
            let descriptor = parse_sfc(
                source,
                SfcParseOptions {
                    filename: "App.vue".into(),
                    ..Default::default()
                },
            )
            .unwrap();
            let mut options = SfcCompileOptions::default();
            options.parse.filename = "App.vue".into();
            options.script.id = Some(path.to_string_lossy().as_ref().into());
            options.template.id = Some("App.vue".into());
            options.style.id = "App.vue".into();
            let expected =
                with_whitespace_mode(settings.whitespace, settings.legacy_line_breaks, || {
                    compile_sfc(&descriptor, options)
                })
                .unwrap();
            assert!(expected.errors.is_empty());
            assert_eq!(actual, expected.code.len());
            lengths.push(actual);
            // The existing RAII scope must leave an ordinary subsequent compile condensed.
            let ordinary = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
            assert_eq!(
                ordinary.code,
                with_whitespace_mode(WhitespaceStrategy::Condense, false, || compile_sfc(
                    &descriptor,
                    SfcCompileOptions::default()
                ))
                .unwrap()
                .code
            );
        }
        assert_ne!(lengths[0], lengths[1]);
        assert_eq!(lengths[0], lengths[2]);
        assert_eq!(lengths[1], lengths[4]);
        assert_eq!(lengths[0], lengths[5]);
        assert_eq!(lengths[3], lengths[6]);
        assert_eq!(cache.entries.lock().unwrap().len(), 3);
    }
}
