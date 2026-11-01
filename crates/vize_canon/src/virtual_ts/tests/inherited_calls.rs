//! Authored callees keep receiver ownership without changing other references.
use super::*;
use crate::virtual_ts::VirtualTsOutput;
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = "import { shared } from './shared'; import Child from './Child.vue'; export default { mixins: [shared] };";
fn generate(template: &str, script: &str) -> VirtualTsOutput {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets_options_api(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    )
}
fn callee(receiver: &str, name: &str) -> vize_carton::String {
    vize_carton::cstr!(
        "({receiver} as unknown as (\"{name}\" extends keyof typeof {receiver} ? typeof {receiver} : {{ \"{name}\": any }})).{name}"
    )
}
#[test]
fn direct_optional_generic_and_grouped_calls_retain_affine_property_tokens() {
    let template = "<p :title=\"greet('x')\">{{ greet?.('x') }} {{ identity<number>(1) }} {{ (greet)('x') }} {{ ({ greet, member: holder.greet, text: 'greet' }) }}</p><Child :title=\"/* comment */greet(/* inner */'x')\"/>";
    let output = generate(template, SCRIPT);
    let receiver = "__vize_template_instance";
    let early = output
        .code
        .find("const __vize_template_instance = undefined as unknown as")
        .unwrap();
    let first_call = output
        .code
        .find(callee(receiver, "greet").as_str())
        .unwrap();
    assert!(early < first_call);
    assert_eq!(
        output
            .code
            .matches("({ greet, member: holder.greet, text: 'greet' })")
            .count(),
        1
    );
    let expected = [
        (
            ":title=\"greet('x')",
            ":title=\"".len(),
            "greet",
            "greet('x')",
        ),
        ("greet?.('x')", 0, "greet", "greet?.('x')"),
        ("identity<number>(1)", 0, "identity", "identity<number>(1)"),
        ("(greet)('x')", 1, "greet", "greet)('x')"),
        (
            "/* comment */greet(/* inner */'x')",
            "/* comment */".len(),
            "greet",
            "greet( 'x')",
        ),
    ]
    .map(|(marker, offset, name, tail)| {
        let start = template.find(marker).unwrap() + offset;
        let whole = callee(receiver, name);
        (
            start,
            start + name.len(),
            whole.get(..whole.len() - name.len()).unwrap().to_owned(),
            tail,
        )
    });
    let mut actual = std::collections::BTreeSet::new();
    let owner = format!("({receiver} as ");
    for row in output.mapping.rows().filter(|row| {
        row.meta.features == crate::virtual_ts::mapping::ProjectionFeatures::DIAGNOSTICS
    }) {
        let prefix = output.code.get(row.span.gen_range.clone()).unwrap();
        if !prefix.starts_with(&owner) {
            continue;
        }
        let source = &row.span.src_range;
        let expected_row = expected
            .iter()
            .find(|(start, end, _, _)| *start == source.start && *end == source.end)
            .expect("every synthetic receiver prefix must own an actual authored callee");
        assert_eq!(
            prefix, expected_row.2,
            "complete receiver view at {source:?}"
        );
        assert_eq!(
            output
                .mapping
                .diagnostic_range_to_authored(row.span.gen_range.start, row.span.gen_range.end),
            Some((source.start, source.end)),
            "actual diagnostic consumer honors this authored callee"
        );
        let property_start = row.span.gen_range.end;
        assert_eq!(
            output
                .code
                .get(property_start..property_start + expected_row.3.len()),
            Some(expected_row.3),
            "all call syntax and type arguments remain authored"
        );
        if source.start == template.find("/* comment */greet").unwrap() + "/* comment */".len() {
            let argument = template.find("/* inner */'x'").unwrap() + "/* inner */".len();
            let generated_argument = property_start + "greet( ".len();
            assert_eq!(
                output
                    .mapping
                    .diagnostic_range_to_authored(generated_argument, generated_argument + 3),
                Some((argument, argument + 3)),
                "internal comment preserves the actual argument range"
            );
        }
        actual.insert((source.start, source.end, prefix.to_owned()));
    }
    // Multiple emitted copies are all checked above; each authored call has one
    // distinct owner/prefix, independent of prop-inference duplication.
    let expected = expected
        .into_iter()
        .map(|(start, end, prefix, _)| (start, end, prefix))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "complete diagnostic-only receiver projection"
    );
}
#[test]
fn lexical_loop_and_slot_callees_keep_their_own_receiver() {
    let template = "<div>{{ ((greet: (x: string) => string) => greet('x'))((x) => x) }}<p v-for=\"greet in [(x: string) => x]\">{{ greet('x') }}</p><Child v-slot=\"{ greet }\">{{ greet('x') }}</Child></div>";
    let output = generate(template, SCRIPT);
    assert_eq!(
        output
            .code
            .matches("as unknown as (\"greet\" extends keyof")
            .count(),
        0,
        "{}",
        output.code
    );
}
#[test]
fn receiver_hygiene_and_missing_member_guard_keep_distinct_owners() {
    let template =
        "<p>{{ (__vize_template_instance => greet('x'))(0) }} {{ missingInherited() }}</p>";
    let output = generate(template, SCRIPT);
    assert_eq!(
        output
            .code
            .matches(callee("__vize_template_instance_", "greet").as_str())
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches(callee("__vize_template_instance_", "missingInherited").as_str())
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("void (__vize_template_instance.missingInherited);")
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("\"greet\" | \"missingInherited\"")
            .count(),
        0
    );
}
#[test]
fn script_names_and_parse_failure_keep_the_old_unprojected_route() {
    for script in [
        "declare namespace Named { const item: string }; export default { mixins: [shared] };",
        "export default { mixins: [shared], broken: };",
    ] {
        let output = generate("<p>{{ Named('x') }} {{ greet('x') }}</p>", script);
        assert_eq!(
            output
                .code
                .matches("as unknown as (\"Named\" extends keyof")
                .count(),
            0,
            "{}",
            output.code
        );
        if script.contains("broken:") {
            assert_eq!(
                output
                    .code
                    .matches("as unknown as (\"greet\" extends keyof")
                    .count(),
                0,
                "{}",
                output.code
            );
        }
    }
}
