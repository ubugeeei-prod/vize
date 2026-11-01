//! Authored callees keep receiver ownership without changing other references.
use super::*;
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = "import { shared } from './shared'; export default { mixins: [shared] };";
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
fn callee(receiver: &str, name: &str) -> std::string::String {
    format!(
        "({receiver} as unknown as (\"{name}\" extends keyof typeof {receiver} ? typeof {receiver} : {{ \"{name}\": any }})).{name}"
    )
}
#[test]
fn direct_optional_generic_and_grouped_calls_retain_affine_property_tokens() {
    let template = "<p :title=\"greet('x')\">{{ greet?.('x') }} {{ identity<number>(1) }} {{ (greet)('x') }} {{ ({ greet, member: holder.greet, text: 'greet' }) }}</p>";
    let output = generate(template, SCRIPT);
    let receiver = "__vize_template_instance";
    assert!(
        output
            .code
            .contains(&format!("{}('x')", callee(receiver, "greet"))),
        "{}",
        output.code
    );
    assert!(
        output
            .code
            .contains(&format!("{}?.('x')", callee(receiver, "greet"))),
        "{}",
        output.code
    );
    assert!(
        output
            .code
            .contains(&format!("{}<number>(1)", callee(receiver, "identity"))),
        "{}",
        output.code
    );
    assert!(
        output
            .code
            .contains(&format!("({})('x')", callee(receiver, "greet"))),
        "{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("({ greet, member: holder.greet, text: 'greet' })"),
        "{}",
        output.code
    );
    let early = output
        .code
        .find("const __vize_template_instance = undefined as unknown as")
        .unwrap();
    let first_call = output.code.find(&callee(receiver, "greet")).unwrap();
    assert!(early < first_call);
    let prefixes = output
        .mapping
        .spans()
        .iter()
        .filter(|m| {
            output
                .code
                .get(m.gen_range.clone())
                .is_some_and(|s| s.ends_with(")).") && s.contains("as unknown as (\""))
        })
        .collect::<Vec<_>>();
    assert!(!prefixes.is_empty(), "{}", output.code);
    for prefix in prefixes {
        let authored = template.get(prefix.src_range.clone()).unwrap();
        assert!(matches!(authored, "greet" | "identity"), "{authored}");
    }
}
#[test]
fn lexical_loop_and_slot_callees_keep_their_own_receiver() {
    let template = "<div>{{ ((greet: (x: string) => string) => greet('x'))((x) => x) }}<p v-for=\"greet in [(x: string) => x]\">{{ greet('x') }}</p><Child v-slot=\"{ greet }\">{{ greet('x') }}</Child></div>";
    let output = generate(template, SCRIPT);
    assert!(
        !output
            .code
            .contains("as unknown as (\"greet\" extends keyof"),
        "{}",
        output.code
    );
}
#[test]
fn receiver_hygiene_and_missing_member_guard_keep_distinct_owners() {
    let template =
        "<p>{{ (__vize_template_instance => greet('x'))(0) }} {{ missingInherited() }}</p>";
    let output = generate(template, SCRIPT);
    assert!(
        output
            .code
            .contains(&callee("__vize_template_instance_", "greet")),
        "{}",
        output.code
    );
    assert!(
        output
            .code
            .contains(&callee("__vize_template_instance_", "missingInherited")),
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
    assert!(!output.code.contains("\"greet\" | \"missingInherited\""));
}
#[test]
fn script_names_and_parse_failure_keep_the_old_unprojected_route() {
    for script in [
        "declare namespace Named { const item: string }; export default { mixins: [shared] };",
        "export default { mixins: [shared], broken: };",
    ] {
        let output = generate("<p>{{ Named('x') }} {{ greet('x') }}</p>", script);
        assert!(
            !output
                .code
                .contains("as unknown as (\"Named\" extends keyof"),
            "{}",
            output.code
        );
        if script.contains("broken:") {
            assert!(
                !output
                    .code
                    .contains("as unknown as (\"greet\" extends keyof"),
                "{}",
                output.code
            );
        }
    }
}
