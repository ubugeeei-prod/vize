//! Template-scope ref unwrapping for framework auto-import bindings (#4146).
//!
//! An auto-imported composable is a `<script setup>` import once the framework
//! transform runs, so Vue unwraps its refs in the template exactly like an
//! authored import. These tests pin the generated shadow pair
//! (`type __R_x = typeof x;` + `var x: __U<__R_x>`) and every case that must
//! *not* get one.

use vize_canon::virtual_ts::{
    TemplateGlobal, VirtualTsOptions, generate_virtual_ts_with_offsets,
    generate_virtual_ts_with_offsets_legacy_vue2, generate_virtual_ts_with_offsets_options_api,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

const REF_STUB: &str = "declare const currentUser: typeof import('./composables')['currentUser'];";

fn generate(script: &str, template: &str, options: &VirtualTsOptions) -> String {
    let allocator = vize_s0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets(&summary, Some(script), Some(&root), 0, 0, options)
        .code
        .into()
}

fn generate_legacy(script: &str, template: &str, options: &VirtualTsOptions) -> String {
    let allocator = vize_s0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets_legacy_vue2(&summary, Some(script), Some(&root), 0, 0, options)
        .code
        .into()
}

fn options_with_stubs(stubs: &[&str]) -> VirtualTsOptions {
    VirtualTsOptions {
        auto_import_stubs: stubs.iter().map(|stub| (*stub).into()).collect(),
        ..Default::default()
    }
}

#[test]
fn template_referenced_auto_import_gets_the_unwrap_shadow() {
    let code = generate(
        "",
        r#"<div>{{ currentUser.account }}</div>"#,
        &options_with_stubs(&[REF_STUB]),
    );
    assert!(
        code.contains("  type __R_currentUser = typeof currentUser;\n"),
        "auto-imported refs need a pre-template type capture:\n{code}"
    );
    assert!(
        code.contains("    var currentUser: __U<__R_currentUser> = undefined as any;\n"),
        "auto-imported refs need an unwrapped template shadow:\n{code}"
    );
    // The capture must be emitted *outside* `__template()`, otherwise `typeof`
    // would read the shadow it is defining.
    let capture = code.find("type __R_currentUser").expect("capture");
    let template_fn = code.find(";(function __template()").expect("template fn");
    assert!(
        capture < template_fn,
        "the type capture must precede the template closure:\n{code}"
    );
}

#[test]
fn auto_import_names_the_template_never_uses_are_not_shadowed() {
    let code = generate("", r#"<div>hello</div>"#, &options_with_stubs(&[REF_STUB]));
    assert!(
        !code.contains("__R_currentUser"),
        "an unreferenced auto-import must not reach template scope:\n{code}"
    );
}

#[test]
fn batch_path_binding_names_produce_the_same_shadow() {
    // The batch virtual project materializes the stubs into one ambient file
    // and carries only the names, so both spellings must agree.
    let inline = generate(
        "",
        r#"<div>{{ currentUser.account }}</div>"#,
        &options_with_stubs(&[REF_STUB]),
    );
    let names_only = generate(
        "",
        r#"<div>{{ currentUser.account }}</div>"#,
        &VirtualTsOptions {
            auto_import_bindings: vec!["currentUser".into()],
            ..Default::default()
        },
    );
    assert!(
        names_only.contains("    var currentUser: __U<__R_currentUser> = undefined as any;\n"),
        "names-only options must still shadow the binding:\n{names_only}"
    );
    assert!(
        !names_only.contains("declare const currentUser:"),
        "the batch path declares the stub once in its ambient file, not per SFC:\n{names_only}"
    );
    assert_eq!(
        template_scope(&inline),
        template_scope(&names_only),
        "the two spellings must generate the same template scope"
    );
}

/// The generated `__template()` body, which is the surface this issue owns.
/// Byte offsets recorded in `@vize-map` comments differ between the two option
/// spellings (one emits an extra module-scope stub), so they are excluded.
fn template_scope(code: &str) -> String {
    let start = code
        .find(";(function __template()")
        .expect("template closure");
    let end = code.find("  })();").expect("template closure end");
    code[start..end]
        .lines()
        .filter(|line| !line.trim_start().starts_with("// @vize-map:"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn script_declared_and_imported_names_are_never_double_shadowed() {
    let script = "import { currentUser } from './composables'\n";
    let code = generate(
        script,
        r#"<div>{{ currentUser.account }}</div>"#,
        &options_with_stubs(&[REF_STUB]),
    );
    assert_eq!(
        code.matches("type __R_currentUser = typeof currentUser;")
            .count(),
        1,
        "an authored import already owns the shadow:\n{code}"
    );
    assert_eq!(
        code.matches("var currentUser: __U<__R_currentUser> = undefined as any;")
            .count(),
        1,
        "an authored import must not gain a second shadow:\n{code}"
    );
}

#[test]
fn options_api_setup_spread_names_are_never_double_shadowed() {
    // A `setup()` return spread infers its bindings from the template, so it
    // claims names `summary.bindings` never holds: exactly the set the
    // auto-import candidates are drawn from. Only one of the two may shadow
    // the name; a second `__R_currentUser` would be a `TS2451`.
    let script = r#"import { defineComponent, toRefs } from 'vue'

export default defineComponent({
    setup() {
        const state = useAccountState()
        return {
            ...toRefs(state),
        }
    },
})
"#;
    let allocator = vize_s0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, r#"<div>{{ currentUser.account }}</div>"#);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &options_with_stubs(&[REF_STUB]),
    );
    let code = output.code.as_str();
    assert_eq!(
        code.matches("type __R_currentUser =").count(),
        1,
        "the setup spread already owns the capture:\n{code}"
    );
    assert_eq!(
        code.matches("var currentUser: __U<__R_currentUser> = undefined as any;")
            .count(),
        1,
        "the setup spread already owns the shadow:\n{code}"
    );
}

#[test]
fn declined_stub_shapes_never_reach_template_scope() {
    let stubs = [
        // A function is never a ref.
        "declare function useThing<T = any>(...args: any[]): any;",
        // The degraded fallback stub carries no type information.
        "declare const useAsyncData: any;",
        // `$`-prefixed names are declared by the template context itself.
        "declare const $fetch: typeof import('ofetch')['$fetch'];",
        // A type alias declares no value.
        "type NuxtApp = any;",
    ];
    let template = r#"<div>{{ useThing() }}{{ useAsyncData }}{{ $fetch }}</div>"#;
    let code = generate("", template, &options_with_stubs(&stubs));
    for name in ["useThing", "useAsyncData", "$fetch", "NuxtApp"] {
        assert!(
            !code.contains(&format!("__R_{name}")),
            "`{name}` must not be shadowed:\n{code}"
        );
    }
}

#[test]
fn component_and_template_global_names_keep_their_own_declarations() {
    let options = VirtualTsOptions {
        auto_import_stubs: vec![
            "declare const AutoCard: typeof import('./AutoCard.vue')['default'];".into(),
            "declare const t: typeof import('vue-i18n')['t'];".into(),
        ],
        template_globals: vec![TemplateGlobal {
            name: "t".into(),
            type_annotation: "(key: string) => string".into(),
            default_value: "((key: string) => key) as any".into(),
        }],
        ..Default::default()
    };
    let code = generate("", r#"<AutoCard :label="t('a')" />"#, &options);
    assert!(
        !code.contains("__R_AutoCard"),
        "a component binding must keep its own declaration:\n{code}"
    );
    assert!(
        !code.contains("__R_t"),
        "a configured template global must keep its own declaration:\n{code}"
    );
}

#[test]
fn vue2_structural_unwrap_never_takes_auto_imports() {
    // Vue 2.7's `__U` is `T extends { value: infer V } ? V : T`, which cannot
    // tell a ref from a plain `{ text, value }` constant (#3767). Auto-imports
    // have no declaration site to classify, so they stay out of it.
    let code = generate_legacy(
        "const localRef = ref(0)\n",
        r#"<div>{{ localRef }}{{ OPTION.value }}</div>"#,
        &options_with_stubs(&["declare const OPTION: typeof import('./options')['OPTION'];"]),
    );
    assert!(
        code.contains("type __U<T> = T extends { value: infer __V } ? __V : T;"),
        "the legacy dialect keeps its structural helper:\n{code}"
    );
    assert!(
        code.contains("var localRef: __U<__R_localRef> = undefined as any;"),
        "the legacy dialect still unwraps its own setup refs:\n{code}"
    );
    assert!(
        !code.contains("__R_OPTION"),
        "Vue 2.7 must not structurally unwrap an auto-import:\n{code}"
    );
}

#[test]
fn vue3_dialect_uses_the_nominal_ref_helper_for_auto_imports() {
    let mut options = options_with_stubs(&[
        "declare const OPTION: typeof import('./options')['OPTION'];",
        REF_STUB,
    ]);
    options.template_globals.clear();
    let code = generate(
        "",
        r#"<div>{{ OPTION.value }}{{ currentUser.account }}</div>"#,
        &options,
    );
    assert!(
        code.contains(
            "type __U<T> = T extends import('vue').Ref ? __VizeWidenTemplateRef<T['value']> : T;"
        ),
        "the Vue 3 dialect must keep the nominal `Ref` test:\n{code}"
    );
    assert!(
        code.contains("    var OPTION: __U<__R_OPTION> = undefined as any;\n"),
        "the nominal helper is safe for every auto-import:\n{code}"
    );
}
