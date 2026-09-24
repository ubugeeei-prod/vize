//! Replaying a `repro.folio` (TS-23), and the content-seeded injection that
//! `vize reduce` shrinks against (P3-14).
//!
//! A plain injected panic (`inject-panic=<pass>`) fires whatever the
//! artifact says, so a reducer could delete the whole artifact and still
//! "reproduce" — a vacuous oracle. `inject-when=<tag>` makes the seeded
//! crash **content-dependent**: the injected pass panics only while the
//! artifact's S1 surface tree still contains an element named `<tag>`, the
//! shape of a real construct-triggered compiler bug. It is a test seed, like
//! `inject-panic` itself, and is only ever read from a repro's config.

use std::panic::{AssertUnwindSafe, catch_unwind};

use vize_davinci::folio::repro::ReproFolio;
use vize_davinci::pass::parse_pipelines;
use vize_s0::{Allocator, String, cstr};

use super::{
    ARTIFACT_STAGE_SOURCE, CONFIG_INJECT, CONFIG_MODE, IceFailure, mode_flags, panic_reason,
    run_injected, silence_panics,
};

/// `[repro.config]` key making an injected panic content-dependent: the
/// tag of the element whose presence triggers it (P3-14's seeded crash).
pub(crate) const CONFIG_INJECT_WHEN: &str = "inject-when";

/// Whether `source`'s S1 surface tree holds an element named `tag`,
/// anywhere (SFC blocks parse as markup, so this sees template content).
pub(crate) fn has_element(source: &str, tag: &str) -> bool {
    fn walk(children: &[vize_s1::SurfaceChild<'_>], tag: &str) -> bool {
        children.iter().any(|child| match child {
            vize_s1::SurfaceChild::Element(element) => {
                element.tag() == tag || walk(&element.children, tag)
            }
            _ => false,
        })
    }
    let allocator = Allocator::default();
    let (tree, _errors) = vize_s1::parse(&allocator, source);
    walk(&tree.children, tag)
}

/// Replay a parsed repro: `Ok(Some(_))` reproduced a failure, `Ok(None)`
/// completed without one, `Err` means this repro cannot be replayed at all.
pub(crate) fn replay(folio: &ReproFolio) -> Result<Option<IceFailure>, String> {
    if let Some(pass) = folio.config.get(CONFIG_INJECT) {
        if let Some(tag) = folio.config.get(CONFIG_INJECT_WHEN)
            && !has_element(folio.artifact.as_str(), tag.as_str())
        {
            return Ok(None);
        }
        return Ok(run_injected(folio.pipeline.as_str(), pass.as_str()).err());
    }
    if folio.artifact_stage.as_str() != ARTIFACT_STAGE_SOURCE {
        return Err(cstr!(
            "cannot replay artifact stage `{}`; only `{ARTIFACT_STAGE_SOURCE}` replays today",
            folio.artifact_stage
        ));
    }
    let mode = folio.config.get(CONFIG_MODE).map_or("dom", |m| m.as_str());
    let Some((ssr, vapor)) = mode_flags(mode) else {
        return Err(cstr!("unknown mode `{mode}` in [repro.config]"));
    };
    silence_panics();
    let segments = parse_pipelines(folio.pipeline.as_str())
        .map_err(|error| cstr!("invalid repro pipeline: {error:?}"))?;
    let stage = segments.first().map_or("", |segment| segment.stage);
    match catch_unwind(AssertUnwindSafe(|| {
        compile_source(folio.artifact.as_str(), ssr, vapor);
    })) {
        Ok(()) => Ok(None),
        // The same attribution rule the record side used for a real-compile
        // panic: the plan's stage, no pass.
        Err(payload) => Ok(Some(IceFailure {
            stage: String::from(stage),
            pass: String::default(),
            reason: panic_reason(payload),
        })),
    }
}

/// Compile an embedded source with the recorded mode's defaults. Diagnostics
/// are irrelevant to a replay - only a panic matters - so results and errors
/// are discarded alike.
pub(crate) fn compile_source(source: &str, ssr: bool, vapor: bool) {
    use vize_atelier_core::{CodegenOptions, options::CustomElementMatcher};
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
        TemplateCompileOptions,
        compile_sfc_with_custom_elements_template_syntax_and_codegen_options, parse_sfc,
    };

    let parse_options = || SfcParseOptions {
        filename: "repro.vue".into(),
        ..Default::default()
    };
    let Ok(descriptor) = parse_sfc(source, parse_options()) else {
        return;
    };
    let has_scoped = descriptor.styles.iter().any(|style| style.scoped);
    let options = SfcCompileOptions {
        parse: parse_options(),
        script: ScriptCompileOptions::default(),
        template: TemplateCompileOptions {
            id: Some("repro.vue".into()),
            scoped: has_scoped,
            ssr,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: "repro.vue".into(),
            scoped: has_scoped,
            ..Default::default()
        },
        vapor,
        scope_id: None,
    };
    let _ = compile_sfc_with_custom_elements_template_syntax_and_codegen_options(
        &descriptor,
        options,
        vize_atelier_core::TemplateSyntaxMode::Standard,
        CustomElementMatcher::from_patterns(Vec::new()),
        CodegenOptions::default(),
    );
}
