//! Vue 3 accepts intrinsic attribute names that `NativeElements` does not list.
//! An omitted `vueCompilerOptions.target` excess-checks the name instead of
//! assigning it to `never`.

#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]

use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput,
    generate_virtual_ts_with_offsets_and_checks,
};
use vize_carton::config::VueVersion;
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = "import Child from './Child.vue'\nconst borderStyle = { color: \"red\" };\nconst label = \"math\";\n";

const TEMPLATE: &str = r#"<div class="subject-label" :style="borderStyle" x-small>
  {{ label }}
</div>
<table width="100%">
  <tr height="26px">
    <th width="56px">edit</th>
  </tr>
</table>
<div no-gutters class="divider"></div>
<div cols="12" class="cards"></div>
<Child not-a-real-prop="x" />"#;

fn generate(vue3: bool) -> VirtualTsOutput {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(SCRIPT);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets_and_checks(
        &analyzer.finish(),
        Some(SCRIPT),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            check_options: VirtualTsCheckOptions {
                check_unknown_props: true,
                vue_compiler_target: vue3.then_some(VueVersion::V3),
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

fn has_fragment(code: &str, needle: &str) -> bool {
    code.split(needle).nth(1).is_some()
}

fn assert_has(code: &str, needle: &str) {
    let found = has_fragment(code, needle);
    assert!(found, "missing `{needle}`:\n{code}");
}

fn assert_lacks(code: &str, needle: &str) {
    let found = has_fragment(code, needle);
    assert!(!found, "unexpected `{needle}`:\n{code}");
}

fn assert_component_prop_still_checked(code: &str) {
    assert_has(code, "const __vize_prop_check_0_not_a_real_prop:");
    assert_has(
        code,
        "__VizePublicComponentAttrs & __VizeComponentGlobalHtmlAttrs & __VizeAllowedFallthroughAttrs<C>",
    );
    assert_has(
        code,
        r#"__VizeNativeElementProp<__VizeNativeElement<"div">, "style">"#,
    );
}

#[test]
fn vue3_check_unknown_props_accepts_intrinsic_attributes() {
    let output = generate(true);
    assert_lacks(&output.code, "__vize_native_key_");
    for name in ["x-small", "height", "no-gutters", "cols"] {
        assert_lacks(&output.code, &format!("\"{name}\""));
    }
    assert_lacks(
        &output.code,
        r#"Partial<__VizeNativeElement<"th">> = { "width""#,
    );
    assert_component_prop_still_checked(&output.code);
}

#[test]
fn omitted_target_excess_checks_intrinsic_names_without_never_assignment() {
    let output = generate(false);
    let native_keys: Vec<_> = output
        .code
        .lines()
        .filter(|line| has_fragment(line, "__vize_native_key_"))
        .collect();
    assert!(!native_keys.is_empty(), "{}", output.code);
    for line in native_keys {
        let assigns_never = has_fragment(line, "never =");
        assert!(
            !assigns_never,
            "omitted target must not assign the name to never: {line}"
        );
        let typed = has_fragment(line, "Partial<__VizeNativeElement<")
            || has_fragment(line, "void __vize_native_key_");
        assert!(typed, "{line}");
    }
    for fragment in [
        r#"Partial<__VizeNativeElement<"div">> = { "x-small": undefined as never }"#,
        r#"Partial<__VizeNativeElement<"tr">> = { "height": undefined as never }"#,
        r#"Partial<__VizeNativeElement<"th">> = { "width": undefined as never }"#,
        r#"Partial<__VizeNativeElement<"div">> = { "no-gutters": undefined as never }"#,
        r#"Partial<__VizeNativeElement<"div">> = { "cols": undefined as never }"#,
    ] {
        assert_has(&output.code, fragment);
    }
    let name_start = TEMPLATE.find("x-small").expect("x-small");
    let src_range = name_start..name_start + "x-small".len();
    let mapping = output
        .mapping
        .spans()
        .iter()
        .find(|mapping| mapping.src_range == src_range)
        .expect("x-small must map its own attribute name");
    assert_eq!(
        &output.code[mapping.gen_range.clone()],
        "\"x-small\"",
        "the diagnostic key must be x-small, not a neighboring binding"
    );
    assert_component_prop_still_checked(&output.code);
}
