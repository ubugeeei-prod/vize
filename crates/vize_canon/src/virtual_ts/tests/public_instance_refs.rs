//! Unknown template names of a component without an Options API default
//! export read the public instance (`__Ctx`, Vue's `ComponentPublicInstance`),
//! so a member a global mixin or plugin declares through
//! `ComponentCustomProperties` resolves with its declared type, and a name
//! nothing declares reports once per authored occurrence from the instance.

use super::{TemplateGlobal, VirtualTsOptions};
use vize_croquis::{Analyzer, AnalyzerOptions};

const PRELUDE: &str = "  type __VizeTemplateInstance = { [__K in keyof __Ctx]: __Ctx[__K] };";
const MEMBER_ALIAS: &str = "  type __VizeTemplateInstanceMember<__K extends string> = __K extends keyof __VizeTemplateInstance ? __VizeTemplateInstance[__K] : any;";
const INSTANCE: &str =
    "  const __vize_public_instance = undefined as unknown as __VizeTemplateInstance;";

fn setup_output(
    script: &str,
    template: &str,
    options: &VirtualTsOptions,
) -> (vize_carton::String, Vec<crate::virtual_ts::VizeMapping>) {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output =
        super::generate_virtual_ts_with_offsets(&summary, Some(script), Some(&root), 0, 0, options);
    let spans = output.mapping.spans().to_vec();
    (output.code, spans)
}

fn count_lines(code: &str, expected: &str) -> usize {
    code.lines().filter(|line| *line == expected).count()
}

fn parses(code: &str) {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::ts()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "the generated module must parse: {:?}\n{code}",
        parsed.diagnostics
    );
}

#[test]
fn script_setup_reads_an_unknown_name_off_the_public_instance_once_per_use() {
    let script = "const label = 'x'\n";
    let template = r#"<div :title="isSP ? 'a' : 'b'">{{ label }} {{ isSP }}</div>"#;
    let (code, spans) = setup_output(script, template, &VirtualTsOptions::default());
    parses(&code);

    for expected in [PRELUDE, MEMBER_ALIAS, INSTANCE] {
        assert_eq!(count_lines(&code, expected), 1, "{expected:?} in:\n{code}");
    }
    // one hoisted declaration per name: the authored expressions resolve it
    // with the member's type, `any` when the instance does not carry it
    assert_eq!(
        count_lines(
            &code,
            "  var isSP!: __VizeTemplateInstanceMember<\"isSP\">; void (isSP);"
        ),
        1,
        "{code}"
    );
    // one instance read per authored occurrence, each mapped to its own use
    assert_eq!(
        count_lines(&code, "  void (__vize_public_instance.isSP);"),
        2,
        "{code}"
    );
    let first = template.find("isSP").expect("the first use");
    let second = template.rfind("isSP").expect("the second use");
    for use_start in [first, second] {
        let reads = spans
            .iter()
            .filter(|mapping| {
                mapping.src_range == (use_start..use_start + "isSP".len())
                    && code
                        .get(..mapping.gen_range.start)
                        .is_some_and(|head| head.ends_with("__vize_public_instance."))
            })
            .count();
        assert_eq!(
            reads, 1,
            "each use owns one instance read: {spans:?}\n{code}"
        );
    }
    // a bound name is never read off the instance
    assert_eq!(
        count_lines(&code, "  void (__vize_public_instance.label);"),
        0,
        "{code}"
    );
    assert_eq!(count_lines(&code, "  void (isSP);"), 0, "{code}");
}

#[test]
fn a_declared_template_global_keeps_its_context_binding() {
    let options = VirtualTsOptions {
        template_globals: vec![TemplateGlobal {
            name: "appName".into(),
            type_annotation: "string".into(),
            default_value: "undefined as any".into(),
        }],
        ..Default::default()
    };
    let (code, _) = setup_output("", "<p>{{ appName }} {{ other }}</p>", &options);
    parses(&code);
    // the context declares `appName`; only `other` reads the instance
    assert_eq!(
        count_lines(&code, "  void (__vize_public_instance.appName);"),
        0,
        "{code}"
    );
    assert_eq!(
        count_lines(
            &code,
            "  var appName!: __VizeTemplateInstanceMember<\"appName\">; void (appName);"
        ),
        0,
        "{code}"
    );
    assert_eq!(
        count_lines(&code, "  void (__vize_public_instance.other);"),
        1,
        "{code}"
    );
}

/// A TypeScript-only declaration the script holds (a `namespace`) is no
/// template binding for the analyzer, but it resolves from the enclosing
/// scope. It is never read off the instance: a hoisted `var` would shadow it.
#[test]
fn a_script_declared_type_only_name_is_not_read_off_the_instance() {
    let script = "namespace Bare {\n  export const label = 'bare'\n}\nconst label = 'x'\n";
    let (code, _) = setup_output(
        script,
        "<p :data-mode=\"Bare.label\">{{ label }}</p>",
        &VirtualTsOptions::default(),
    );
    parses(&code);
    assert_eq!(
        count_lines(&code, "  void (__vize_public_instance.Bare);"),
        0,
        "{code}"
    );
    assert_eq!(
        count_lines(
            &code,
            "  var Bare!: __VizeTemplateInstanceMember<\"Bare\">; void (Bare);"
        ),
        0,
        "{code}"
    );
}

/// The Options API form of #3888 is untouched: a plain default export keeps
/// resolving unknown names on `typeof __default__`.
#[test]
fn an_options_api_default_export_keeps_its_own_instance_form() {
    let script = "export default { data: () => ({ count: 1 }) }\n";
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<p>{{ count }} {{ missing }}</p>");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let code = super::generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    )
    .code;
    parses(&code);
    assert_eq!(count_lines(&code, PRELUDE), 0, "{code}");
    assert_eq!(
        count_lines(&code, "  void (__vize_template_instance.missing);"),
        1,
        "{code}"
    );
    assert_eq!(
        count_lines(&code, "  var missing: any = undefined; void (missing);"),
        1,
        "{code}"
    );
}

#[test]
fn script_setup_with_no_unknown_name_emits_no_instance_prelude() {
    let (code, _) = setup_output(
        "const label = 'x'\n",
        "<p>{{ label }}</p>",
        &VirtualTsOptions::default(),
    );
    parses(&code);
    assert_eq!(count_lines(&code, PRELUDE), 0, "{code}");
    assert_eq!(count_lines(&code, INSTANCE), 0, "{code}");
}

/// Loop aliases become visible only after evaluating their iterable. Repeated
/// names, including names also used as aliases, keep separate authored owners.
#[test]
fn loop_sources_keep_each_read_and_exclude_the_current_alias() {
    for template in [
        r#"<i v-for="missing in (missing || missing)" :key="missing">{{ missing }}</i>"#,
        r#"<i v-for="entry in ('日本語 missing' && /* missing */ missing || missing)" :key="entry">{{ entry }}</i>"#,
    ] {
        let (code, spans) = setup_output("", template, &VirtualTsOptions::default());
        parses(&code);
        assert_eq!(
            count_lines(&code, "  void (__vize_public_instance.missing);"),
            2,
            "{code}"
        );
        let iterable_start = template.find(" in ").expect("loop source") + " in ".len();
        let iterable_end =
            template[iterable_start..].find('"').expect("source end") + iterable_start;
        let source = &template[iterable_start..iterable_end];
        // The last two spellings are the authored reads; earlier spellings
        // belong to a literal/comment and must never own their mappings.
        let last = source.rfind("missing").expect("second authored read");
        let first = source[..last]
            .rfind("missing")
            .expect("first authored read");
        let expected = vec![iterable_start + first, iterable_start + last];
        let actual: Vec<_> = spans
            .iter()
            .filter(|mapping| {
                code.get(..mapping.gen_range.start)
                    .is_some_and(|head| head.ends_with("__vize_public_instance."))
            })
            .map(|mapping| mapping.src_range.start)
            .collect();
        assert_eq!(actual, expected, "{template}: {spans:?}\n{code}");
    }
}

#[test]
fn nested_loop_sources_keep_the_outer_alias_visible() {
    let template = r#"<section v-for="item in rows"><i v-for="item in item" :key="item">{{ item }}</i></section>"#;
    let (code, _) = setup_output("", template, &VirtualTsOptions::default());
    parses(&code);
    assert_eq!(
        count_lines(&code, "  void (__vize_public_instance.rows);"),
        1,
        "{code}"
    );
    assert_eq!(
        count_lines(&code, "  void (__vize_public_instance.item);"),
        0,
        "{code}"
    );
}
