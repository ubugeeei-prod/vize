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

fn assert_component_prop_still_checked(code: &str) {
    assert!(
        code.contains("const __vize_prop_check_0_not_a_real_prop:"),
        "component checkUnknownProps must still check the unknown prop:\n{code}"
    );
    assert!(
        code.contains("__VizePublicComponentAttrs & __VizeComponentGlobalHtmlAttrs & __VizeAllowedFallthroughAttrs<C>"),
        "strict component tail must stay closed:\n{code}"
    );
    assert!(
        code.contains(r#"__VizeNativeElementProp<__VizeNativeElement<"div">, "style">"#),
        "known DOM prop values stay checked:\n{code}"
    );
}

#[test]
fn vue3_check_unknown_props_accepts_intrinsic_attributes() {
    let output = generate(true);
    assert!(
        !output.code.contains("__vize_native_key_"),
        "Vue 3 must not assign intrinsic names to never:\n{}",
        output.code
    );
    for name in ["x-small", "height", "no-gutters", "cols"] {
        assert!(
            !output.code.contains(&format!("\"{name}\"")),
            "Vue 3 must not excess-check intrinsic name {name}:\n{}",
            output.code
        );
    }
    assert!(
        !output
            .code
            .contains(r#"Partial<__VizeNativeElement<"th">> = { "width""#),
        "Vue 3 must not excess-check width on th:\n{}",
        output.code
    );
    assert_component_prop_still_checked(&output.code);
}

#[test]
fn omitted_target_excess_checks_intrinsic_names_without_never_assignment() {
    let output = generate(false);
    let native_keys: Vec<_> = output
        .code
        .lines()
        .filter(|line| line.contains("__vize_native_key_"))
        .collect();
    assert!(!native_keys.is_empty(), "{}", output.code);
    for line in native_keys {
        assert!(
            !line.contains("never ="),
            "omitted target must not assign the name to never: {line}"
        );
        assert!(
            line.contains("Partial<__VizeNativeElement<")
                || line.contains("void __vize_native_key_"),
            "{line}"
        );
    }
    for fragment in [
        r#"Partial<__VizeNativeElement<"div">> = { "x-small": undefined as never }"#,
        r#"Partial<__VizeNativeElement<"tr">> = { "height": undefined as never }"#,
        r#"Partial<__VizeNativeElement<"th">> = { "width": undefined as never }"#,
        r#"Partial<__VizeNativeElement<"div">> = { "no-gutters": undefined as never }"#,
        r#"Partial<__VizeNativeElement<"div">> = { "cols": undefined as never }"#,
    ] {
        assert!(
            output.code.contains(fragment),
            "missing excess-property check `{fragment}`:\n{}",
            output.code
        );
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
