//! Declared `vue.directive` contracts (#13074) in every template lane.

use crate::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult, SfcParseOptions,
    SfcScriptOutputMode, TemplateCompileOptions, compile_sfc_for_adapter_with_experimental_options,
    parse_sfc,
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_l0::config::{DirectiveContract, DirectiveModifier};

fn track() -> DirectiveContract {
    let modifier = |name: &str, optional, default| DirectiveModifier {
        name: name.into(),
        optional,
        default,
    };
    DirectiveContract {
        plugin: "@acme/tracking".into(),
        name: "track".into(),
        modifiers: vec![
            modifier("once", true, false),
            modifier("passive", true, true),
            modifier("strict", false, false),
        ],
    }
}

#[derive(Clone, Copy)]
enum Lane {
    Vdom,
    Ssr,
    Vapor,
}

fn compile(source: &str, lane: Lane, directives: Vec<DirectiveContract>) -> SfcCompileResult {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    compile_result(&descriptor, lane, directives).unwrap()
}

fn compile_result(
    descriptor: &crate::SfcDescriptor<'_>,
    lane: Lane,
    directives: Vec<DirectiveContract>,
) -> Result<SfcCompileResult, crate::SfcError> {
    compile_sfc_for_adapter_with_experimental_options(
        descriptor,
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: "Track.vue".into(),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                ssr: matches!(lane, Lane::Ssr),
                ..Default::default()
            },
            vapor: matches!(lane, Lane::Vapor),
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions {
            directives,
            ..CodegenOptions::default()
        },
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    )
}

/// The generated line that applies `v-track`.
fn track_line(code: &str) -> &str {
    code.lines()
        .map(str::trim)
        .find(|line| {
            !line.starts_with("const ")
                && (line.contains("_directive_track")
                    || line.contains("_resolveDirective(\"track\")"))
        })
        .unwrap_or_default()
}

/// The errors of one compile, whether it failed or reported them.
fn errors(source: &str, lane: Lane) -> Vec<crate::SfcError> {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    match compile_result(&descriptor, lane, vec![track()]) {
        Ok(result) => result.errors,
        Err(error) => vec![error],
    }
}

const SOURCE: &str = "<template>\n  <a v-track.strict=\"e\">go</a>\n</template>\n";

#[test]
fn declared_defaults_follow_the_authored_modifiers_in_every_lane() {
    let cases = [
        (
            Lane::Vdom,
            "[_directive_track, _ctx.e, void 0, { strict: true, passive: true }]",
        ),
        (
            Lane::Ssr,
            "_push(`<a${_ssrRenderAttrs(_mergeProps(_attrs, _ssrGetDirectiveProps(_ctx, _resolveDirective(\"track\"), _ctx.e, void 0, { strict: true, passive: true })))}>go</a>`)",
        ),
        (
            Lane::Vapor,
            "_withVaporDirectives(n0, [[_directive_track, () => (_ctx.e), undefined, { \"strict\": true, \"passive\": true }]])",
        ),
    ];
    for (lane, expected) in cases {
        let result = compile(SOURCE, lane, vec![track()]);
        assert_eq!(result.errors.len(), 0, "{:?}", result.errors);
        assert_eq!(track_line(&result.code), expected, "{}", result.code);
    }
}

#[test]
fn a_table_the_template_does_not_bind_leaves_the_output_byte_identical() {
    let mut other = track();
    other.name = "other".into();
    for lane in [Lane::Vdom, Lane::Ssr, Lane::Vapor] {
        let plain = compile(SOURCE, lane, Vec::new());
        let declared = compile(SOURCE, lane, vec![other.clone()]);
        assert_eq!(declared.code, plain.code);
        assert_eq!(declared.errors.len(), 0);
    }
}

#[test]
fn a_binding_that_breaks_its_contract_fails_the_template_in_every_lane() {
    let source = "<template>\n  <a v-track.bogus>go</a>\n</template>\n";
    for lane in [Lane::Vdom, Lane::Ssr, Lane::Vapor] {
        let found = errors(source, lane);
        let errors: Vec<_> = found
            .iter()
            .map(|error| {
                let loc = error
                    .loc
                    .as_ref()
                    .map(|loc| (loc.start, loc.end, loc.start_line, loc.start_column));
                (error.code.as_deref(), error.message.as_str(), loc)
            })
            .collect();
        assert_eq!(
            errors,
            [(
                Some("PLUGIN_DIRECTIVE_CONTRACT"),
                "v-track does not declare the modifier `bogus` (declared by @acme/tracking)",
                Some((16, 29, 2, 6)),
            )]
        );
    }
}
