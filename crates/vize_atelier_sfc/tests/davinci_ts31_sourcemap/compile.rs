//! Compile TS-31 fixtures with maps on and off.
//!
//! Every entry point compiles each fixture twice and asserts the emitted
//! JavaScript is byte-identical (TS-11 stays empty: maps are additive
//! artifacts), then returns the text the map indexes plus the parsed map.

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CodegenMode};
use vize_atelier_dom::DomCompilerOptions;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, compile_sfc_with_template_syntax_and_codegen_options,
    parse_sfc, types::ScriptCompileOptions,
};
use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};
use vize_atelier_vapor::{VaporCompilerExperimentalOptions, VaporCompilerOptions};
use vize_s0::Allocator;

use super::battery::{Backend, Fixture};

/// Filename recorded in every template map's `sources`.
pub const TEMPLATE_FILENAME: &str = "Fixture.vue";
/// Filename recorded in every SFC map's `sources`.
pub const SFC_FILENAME: &str = "/app/src/Fixture.vue";

/// The generated text a map indexes, plus the map document (if any).
pub struct Compiled {
    pub generated: std::string::String,
    pub map: Option<serde_json::Value>,
}

pub fn compile(backend: Backend, fixture: &Fixture) -> Compiled {
    match backend {
        Backend::Dom => compile_dom(fixture),
        Backend::Vapor => compile_vapor(fixture),
        Backend::Ssr => compile_ssr(fixture),
        Backend::LegacySfc => compile_sfc(fixture),
    }
}

fn parse_map(map: Option<vize_s0::String>) -> Option<serde_json::Value> {
    map.map(|json| serde_json::from_str(json.as_str()).expect("map is valid JSON"))
}

fn compile_dom(fixture: &Fixture) -> Compiled {
    let run = |source_map: bool| {
        let allocator = Allocator::new();
        let (_, errors, result) =
            vize_atelier_dom::compile_template_with_template_syntax_and_codegen_options(
                &allocator,
                &fixture.source,
                DomCompilerOptions {
                    mode: CodegenMode::Module,
                    prefix_identifiers: true,
                    source_map,
                    ..Default::default()
                },
                TemplateSyntaxMode::Standard,
                CodegenOptions {
                    filename: TEMPLATE_FILENAME.into(),
                    ..Default::default()
                },
            );
        assert_eq!(errors.len(), 0, "{}: DOM errors {errors:?}", fixture.name);
        result
    };
    let (off, on) = (run(false), run(true));
    assert_eq!(
        (off.preamble.as_str(), off.code.as_str(), off.map.is_none()),
        (on.preamble.as_str(), on.code.as_str(), on.map.is_some()),
        "{}: DOM source maps must be additive (TS-11)",
        fixture.name
    );
    Compiled {
        generated: on.code.as_str().into(),
        map: parse_map(on.map),
    }
}

fn compile_vapor(fixture: &Fixture) -> Compiled {
    let run = |source_map: bool| {
        let allocator = Allocator::new();
        let result = vize_atelier_vapor::compile_vapor_with_experimental_options(
            &allocator,
            &fixture.source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
            VaporCompilerExperimentalOptions {
                source_map,
                source_map_filename: Some(TEMPLATE_FILENAME.into()),
                ..Default::default()
            },
        );
        assert_eq!(
            result.error_messages.len(),
            0,
            "{}: Vapor errors {:?}",
            fixture.name,
            result.error_messages
        );
        result
    };
    let (off, on) = (run(false), run(true));
    assert_eq!(
        (&off.code, &off.templates, off.map.is_none()),
        (&on.code, &on.templates, on.map.is_some()),
        "{}: Vapor source maps must be additive (TS-11)",
        fixture.name
    );
    Compiled {
        generated: on.code.as_str().into(),
        map: parse_map(on.map),
    }
}

fn compile_ssr(fixture: &Fixture) -> Compiled {
    let run = |source_map: bool| {
        let allocator = Allocator::new();
        let (_, errors, result) =
            vize_atelier_ssr::compile_ssr_with_template_syntax_and_experimental_options(
                &allocator,
                &fixture.source,
                SsrCompilerOptions::default(),
                TemplateSyntaxMode::Standard,
                SsrCompilerExperimentalOptions {
                    source_map,
                    source_map_filename: Some(TEMPLATE_FILENAME.into()),
                    ..Default::default()
                },
            );
        assert_eq!(errors.len(), 0, "{}: SSR errors {errors:?}", fixture.name);
        result
    };
    let (off, on) = (run(false), run(true));
    assert_eq!(
        (off.preamble.as_str(), off.code.as_str(), off.map.is_none()),
        (on.preamble.as_str(), on.code.as_str(), on.map.is_some()),
        "{}: SSR source maps must be additive (TS-11)",
        fixture.name
    );
    Compiled {
        generated: on.code.as_str().into(),
        map: parse_map(on.map),
    }
}

fn compile_sfc(fixture: &Fixture) -> Compiled {
    let parse = || SfcParseOptions {
        filename: SFC_FILENAME.into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(&fixture.source, parse()).expect("SFC fixture parses");
    let run = |source_map: bool| {
        let options = SfcCompileOptions {
            parse: parse(),
            script: ScriptCompileOptions {
                id: Some(SFC_FILENAME.into()),
                inline_template: false,
                ..Default::default()
            },
            ..Default::default()
        };
        compile_sfc_with_template_syntax_and_codegen_options(
            &descriptor,
            options,
            TemplateSyntaxMode::Standard,
            CodegenOptions {
                source_map,
                ..Default::default()
            },
        )
        .expect("SFC fixture compiles")
    };
    let (off, on) = (run(false), run(true));
    assert_eq!(
        (off.code.as_str(), off.map.is_none()),
        (on.code.as_str(), on.map.is_some()),
        "{}: SFC source maps must be additive (TS-11)",
        fixture.name
    );
    Compiled {
        generated: on.code.as_str().into(),
        map: on.map,
    }
}
