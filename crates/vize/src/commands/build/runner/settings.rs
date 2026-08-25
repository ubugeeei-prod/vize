//! Per-file compile settings and template-syntax mapping for the build command.

use std::path::{Path, PathBuf};
use vize_atelier_core::TemplateSyntaxMode;
use vize_s0::String;
use vize_s0::config::{ConfigFeatureFlags, VueVersion};
use vize_s0::hash::hash_bytes;

use crate::commands::build::{BuildArgs, ScriptExtension};
use crate::commands::davinci_ice;

pub(super) struct BuildConfigSettings {
    pub(super) compiler_template_syntax: Option<&'static str>,
    pub(super) features: ConfigFeatureFlags,
    pub(super) vapor: Option<bool>,
    pub(super) custom_elements: Vec<String>,
    pub(super) dialect: Option<VueVersion>,
    pub(super) host_compiler: Option<bool>,
}

pub(super) fn load_build_config(no_config: bool, config: Option<&Path>) -> BuildConfigSettings {
    if no_config {
        return BuildConfigSettings {
            compiler_template_syntax: None,
            features: ConfigFeatureFlags::default(),
            vapor: None,
            custom_elements: Vec::new(),
            dialect: None,
            host_compiler: None,
        };
    }
    BuildConfigSettings {
        compiler_template_syntax: crate::config::load_compiler_template_syntax(config),
        features: crate::config::load_config_with_features_and_source(config).features,
        vapor: crate::config::load_compiler_vapor(config),
        custom_elements: crate::config::load_compiler_custom_elements(config),
        dialect: crate::config::load_compiler_vue_version(config),
        host_compiler: crate::config::load_compiler_host_compiler(config),
    }
}

/// Compile a single `.vue` file with profiling information.
pub(super) struct CompileFileSettings {
    pub(super) ssr: bool,
    pub(super) vapor: bool,
    pub(super) custom_renderer: bool,
    pub(super) custom_elements: Vec<String>,
    pub(super) template_syntax: TemplateSyntaxMode,
    pub(super) experimental_in_tag_comments: bool,
    pub(super) experimental_patterned_template: bool,
    /// Vue dialect from `vue.version`; defaults to [`VueVersion::V3`] and is
    /// threaded into each file's compile options.
    pub(super) dialect: VueVersion,
    pub(super) script_ext: ScriptExtension,
    pub(super) record_profile_totals: bool,
    pub(super) davinci: DavinciBuildSettings,
}

/// The build's slice of the Davinci ICE machinery (P2-13): the plan the
/// compile path is described by, where a `repro.folio` goes, the per-pass
/// dump controls, and the TS-23 panic injection.
pub(super) struct DavinciBuildSettings {
    /// The selected backend's legacy plan (`legacy_plan::{DOM,SSR,VAPOR}`)
    /// in the canonical pipeline-grammar spelling.
    pub(super) plan_string: String,
    /// The plan's stage name - the attribution for a panic the driver did
    /// not see (an unattributable real-compile ICE).
    pub(super) stage: &'static str,
    /// Mode name recorded in `[repro.config]`.
    pub(super) mode: &'static str,
    /// Parsed `--davinci-inject-panic` spec: `(file stem, pass name)`.
    pub(super) inject: Option<(String, String)>,
    /// Where a failing file's `repro.folio` is written (the output dir).
    pub(super) repro_dir: PathBuf,
}

impl DavinciBuildSettings {
    /// The pass to panic in when compiling `path`, if injection targets it.
    pub(super) fn injected_pass_for(&self, path: &Path) -> Option<&str> {
        let (stem, pass) = self.inject.as_ref()?;
        (path.file_stem().and_then(|name| name.to_str()) == Some(stem.as_str()))
            .then_some(pass.as_str())
    }
}

impl CompileFileSettings {
    /// Resolve per-file compile settings from CLI arguments and config file values.
    pub(super) fn resolve(args: &BuildArgs, build_config: BuildConfigSettings) -> Self {
        let ssr = args.ssr;
        let vapor = args.vapor || build_config.vapor.unwrap_or(false);
        let (plan, mode) = davinci_ice::compile_plan(ssr, vapor);
        let inject = args.davinci_inject_panic.as_deref().map(|spec| {
            davinci_ice::parse_inject_spec(spec, plan).unwrap_or_else(|error| {
                eprintln!("\x1b[31mError:\x1b[0m --davinci-inject-panic: {error}");
                std::process::exit(1);
            })
        });
        // The dump directory is created even though no pages can be written
        // until P2-12b gives the compile path a folio-printable stage
        // artifact: an existing empty directory is an observable "nothing
        // was dumped", where a missing one is indistinguishable from an
        // ignored flag. Pinned by `davinci_repro_cli.rs`.
        if let Some(dir) = args.folio_dir.as_deref()
            && let Err(error) = std::fs::create_dir_all(dir)
        {
            eprintln!(
                "\x1b[31mError:\x1b[0m --folio-dir: cannot create {}: {error}",
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
            experimental_in_tag_comments: build_config.features.experimental_in_tag_comments,
            experimental_patterned_template: build_config.features.experimental_patterned_template,
            dialect: build_config.dialect.unwrap_or_default(),
            script_ext: args.script_ext,
            record_profile_totals: args.profile,
            davinci: DavinciBuildSettings {
                plan_string: davinci_ice::plan_string(plan),
                stage: plan.stage,
                mode,
                inject,
                repro_dir: args.output.clone(),
            },
        }
    }

    /// Packs every compile option that can change stats output into a tiny cache key.
    ///
    /// `record_profile_totals` is intentionally excluded: enabling profiling changes
    /// accounting side effects, not parse/compile output. Script extension is included
    /// because preserving TypeScript can change generated code size. The dialect is
    /// included because it can change parse/compile output on legacy-capable builds.
    pub(super) fn cache_bits(&self) -> u16 {
        u16::from(self.ssr)
            | (u16::from(self.vapor) << 1)
            | (u16::from(self.custom_renderer) << 2)
            | (u16::from(template_syntax_bits(self.template_syntax)) << 3)
            | match self.script_ext {
                ScriptExtension::Preserve => 1 << 5,
                ScriptExtension::Downcompile => 0,
            }
            | (u16::from(dialect_bits(self.dialect)) << 6)
            | (u16::from(self.experimental_in_tag_comments) << 9)
            | (u16::from(self.experimental_patterned_template) << 10)
    }

    pub(super) fn custom_elements_hash(&self) -> u64 {
        if self.custom_elements.is_empty() {
            return 0;
        }

        let byte_len = self
            .custom_elements
            .iter()
            .map(|pattern| u64::BITS as usize / 8 + pattern.len())
            .sum();
        let mut bytes = Vec::with_capacity(byte_len);
        for pattern in &self.custom_elements {
            bytes.extend_from_slice(&(pattern.len() as u64).to_le_bytes());
            bytes.extend_from_slice(pattern.as_bytes());
        }
        hash_bytes(&bytes)
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
