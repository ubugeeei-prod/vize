//! Emission of `$`-prefixed template instance globals in every form.
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput,
    generate_virtual_ts_with_offsets, generate_virtual_ts_with_offsets_and_checks,
    generate_virtual_ts_with_offsets_options_api,
};
use vize_carton::config::VueVersion;
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = "const label = 'x'\n";
const TEMPLATE: &str =
    r#"<p :title="$missing('c')">{{ $missing('a') }}{{ $missing('b') }}{{ label }}</p>"#;

fn generate(strict: bool) -> VirtualTsOutput {
    generate_for(strict, VueVersion::V3)
}

fn generate_for(strict: bool, dialect: VueVersion) -> VirtualTsOutput {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(SCRIPT);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let options = VirtualTsOptions {
        strict_instance_globals: strict,
        ..Default::default()
    };
    if dialect == VueVersion::V3 {
        return generate_virtual_ts_with_offsets(
            &summary,
            Some(SCRIPT),
            Some(&root),
            0,
            0,
            &options,
        );
    }
    generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(SCRIPT),
        Some(&root),
        0,
        0,
        &options,
        VirtualTsGenerationOptions {
            dialect,
            ..Default::default()
        },
    )
}

/// Authored offsets of the three `$missing` occurrences in `TEMPLATE`.
fn authored_offsets() -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut from = 0;
    while let Some(found) = TEMPLATE[from..].find("$missing") {
        offsets.push(from + found);
        from += found + "$missing".len();
    }
    offsets
}

/// `vue-tsc` reports `TS2339` (`TS2551` with a spelling suggestion) at every
/// read of a `$` name the public instance does not declare (#10617).
/// Ordinary Vue 3 reads are rewritten at each authored occurrence (#10617).
/// The prelude must not also declare `const $missing`.
#[test]
fn the_default_vue3_form_checks_every_occurrence_on_the_public_instance() {
    let output = generate(false);
    assert_eq!(
        [
            "__ctx.$missing('c')",
            "__ctx.$missing('a')",
            "__ctx.$missing('b')"
        ]
        .map(|access| output.code.matches(access).count()),
        [1, 1, 1],
        "{}",
        output.code
    );
    assert_eq!(output.code.matches("const $missing =").count(), 0);
    let context_helpers = output
        .code
        .lines()
        .filter(|line| {
            line.contains("__VizeInstanceGlobal")
                || line.contains("$InstanceGlobal")
                || line.contains("__vize_strict_template_context")
        })
        .collect::<Vec<_>>();
    assert_eq!(context_helpers, Vec::<&str>::new());
}

#[test]
fn vue3_reads_every_occurrence_off_the_public_instance() {
    the_default_vue3_form_checks_every_occurrence_on_the_public_instance();
}

#[test]
fn vue2_declares_the_name_once_with_an_any_fallback() {
    let output = generate_for(false, VueVersion::V2_7);

    assert_eq!(
        output
            .code
            .matches(
                "type $InstanceGlobal<K extends string> = K extends keyof __Ctx ? __Ctx[K] : any;"
            )
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const $missing: $InstanceGlobal<'$missing'> = undefined as any;")
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output.code.matches("__ctx.$missing").count(),
        0,
        "{}",
        output.code
    );
}

#[test]
fn the_strict_form_reads_every_occurrence_off_the_template_context() {
    let output = generate(true);

    assert_eq!(
        output.code.matches("$InstanceGlobal").count(),
        0,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const $missing = __vize_strict_template_context.$missing;")
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("void (__vize_strict_template_context.$missing);")
            .count(),
        2,
        "{}",
        output.code
    );
}

#[test]
fn typed_receivers_map_each_access_back_to_its_authored_occurrence() {
    for (strict, receiver) in [(true, "__vize_strict_template_context."), (false, "__ctx.")] {
        let output = generate(strict);

        let mut mapped = output
            .mapping
            .spans()
            .iter()
            .filter(|mapping| {
                output.code.get(mapping.gen_range.clone()) == Some("$missing")
                    && output.code[..mapping.gen_range.start].ends_with(receiver)
            })
            .map(|mapping| mapping.src_range.clone())
            .collect::<Vec<_>>();
        mapped.sort_by_key(|range| range.start);

        assert_eq!(
            mapped,
            authored_offsets()
                .into_iter()
                .map(|start| start..start + "$missing".len())
                .collect::<Vec<_>>(),
            "{}",
            output.code
        );
    }
}

#[test]
fn vue2_maps_the_single_declaration_to_the_first_occurrence() {
    let output = generate_for(false, VueVersion::V2_7);

    let mapped = output
        .mapping
        .spans()
        .iter()
        .filter(|mapping| output.code.get(mapping.gen_range.clone()) == Some("$missing"))
        .map(|mapping| mapping.src_range.clone())
        .collect::<Vec<_>>();

    let first = authored_offsets()[0];
    assert_eq!(mapped, vec![first..first + "$missing".len()]);
}

/// An Options API template resolves on its authored instance, where mixins
/// and `$`-named options members exist, as they do in `vue-tsc`'s context.
#[test]
fn options_api_reads_the_authored_instance() {
    let script = "import { defineComponent } from 'vue'\nexport default defineComponent({})\n";
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let output = generate_virtual_ts_with_offsets_options_api(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    );

    assert_eq!(
        output
            .code
            .matches("  const __vize_instance_globals = undefined as unknown as (typeof __default__ extends abstract new (...args: any) => infer __I ? __I : __Ctx);\n")
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const $missing = __vize_instance_globals.$missing;")
            .count(),
        1,
        "{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("void (__vize_instance_globals.$missing);")
            .count(),
        2,
        "{}",
        output.code
    );
}

/// Normal Options API members already have generated lexical declarations.
/// They must not also acquire public-instance reads, which would report one
/// false TS2339 per authored interpolation when the instance helper is precise.
#[test]
fn options_api_lexical_members_do_not_gain_public_instance_reads() {
    let script = r#"export default {
  props: { label: { type: String, required: true } },
  data() { return { count: 1 }; },
  computed: { doubled() { return this.count * 2; } },
  methods: { increment() { return ++this.count; } },
};
"#;
    let template = "<p>{{ label }}:{{ count }}:{{ doubled }}:{{ increment() }}:{{ missing }}</p>";
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let output = generate_virtual_ts_with_offsets_options_api(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    );

    for name in ["label", "count", "doubled", "increment"] {
        assert_eq!(
            output
                .code
                .matches(&format!("$OptionsBinding<typeof __default__, \"{name}\""))
                .count(),
            1,
            "missing lexical Options API binding for {name}:\n{}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches(&format!("__vize_public_instance.{name}"))
                .count(),
            0,
            "Options API member {name} gained a public-instance read:\n{}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches(&format!("$TemplateInstanceMember<\"{name}\">"))
                .count(),
            0,
            "Options API member {name} gained a public-instance declaration:\n{}",
            output.code
        );
    }
    assert_eq!(
        output
            .code
            .matches("void (__vize_template_instance.missing);")
            .count(),
        1,
        "the undeclared negative control must retain its instance diagnostic:\n{}",
        output.code
    );
}

/// A member the analyzer cannot list — contributed by an imported `mixins:`
/// entry or an `extends:` base — is declared from the authored instance's own
/// type, so the authored expression is checked against it as `vue-tsc` checks
/// `__VLS_ctx.<name>`. A name the instance lacks keeps the `any` fallback and
/// its single mapped instance read.
#[test]
fn options_api_inherited_members_are_declared_from_the_authored_instance() {
    let script = r#"import { defineComponent } from 'vue'
import { base } from './base'
import { shared } from './shared'
export default defineComponent({
  extends: base,
  mixins: [shared],
  data() { return { count: 1 }; },
});
"#;
    let template = "<p>{{ count }}:{{ fromBase.toFixed() }}:{{ fromMixin.toFixed() }}:{{ fromMixin }}:{{ missing }}</p>";
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full()).with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let output = generate_virtual_ts_with_offsets_options_api(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    );

    assert_eq!(
        output
            .code
            .matches("  type $AuthoredInstanceMember<__K extends string> = __K extends keyof typeof __vize_template_instance ? (typeof __vize_template_instance)[__K] : any;\n")
            .count(),
        1,
        "the authored-instance member helper is declared once:\n{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("$OptionsBinding<typeof __default__, \"count\"")
            .count(),
        1,
        "the component's own data member keeps its lexical binding:\n{}",
        output.code
    );
    for name in ["fromBase", "fromMixin", "missing"] {
        assert_eq!(
            output
                .code
                .matches(&format!(
                    "  var {name}!: $AuthoredInstanceMember<\"{name}\">; void ({name});\n"
                ))
                .count(),
            1,
            "inherited or unknown member {name} is declared once from the authored instance:\n{}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches(&format!("void (__vize_template_instance.{name});"))
                .count(),
            1,
            "inherited or unknown member {name} keeps one mapped instance read:\n{}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches(&format!("var {name}: any = undefined;"))
                .count(),
            0,
            "inherited or unknown member {name} must not be declared as a bare any:\n{}",
            output.code
        );
    }
    assert_eq!(
        output
            .code
            .matches("  void (fromMixin.toFixed()); // Interpolation\n")
            .count(),
        1,
        "the authored expression is checked against the declared member:\n{}",
        output.code
    );
}

/// Disabling Options API removes its generated declarations, so authored
/// member reads must remain free names (TS2304). They must not be converted to
/// public-instance reads (TS2339), while a truly unknown name still uses the
/// public instance added for ComponentCustomProperties support.
#[test]
fn disabled_options_api_keeps_known_members_lexical_and_unknowns_public() {
    let script = r#"const spread = { fromSpread: 1 };
const dynamicName = "fromComputed";
export default {
  props: { label: { type: String, required: true } },
  data() { return { count: 1, ...spread, [dynamicName]: 2 }; },
  computed: { doubled() { return this.count * 2; } },
  methods: { increment() { return ++this.count; } },
};
"#;
    let template = "<p>{{ label }}:{{ count }}:{{ doubled }}:{{ increment() }}:{{ fromSpread }}:{{ fromComputed }}:{{ missing }}</p>";
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let output = generate_virtual_ts_with_offsets(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    );

    assert_eq!(output.code.matches("$OptionsBinding").count(), 0);
    for name in ["label", "count", "doubled", "increment"] {
        assert_eq!(
            output
                .code
                .matches(&format!("__vize_public_instance.{name}"))
                .count(),
            0,
            "disabled Options API member {name} gained a public-instance read:\n{}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches(&format!("$TemplateInstanceMember<\"{name}\">"))
                .count(),
            0,
            "disabled Options API member {name} gained a public-instance declaration:\n{}",
            output.code
        );
    }
    for name in ["fromSpread", "fromComputed", "missing"] {
        assert_eq!(
            output
                .code
                .matches(&format!("$TemplateInstanceMember<\"{name}\">"))
                .count(),
            1,
            "dynamic and unknown controls must retain public-instance resolution:\n{}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches(&format!("void (__vize_public_instance.{name});"))
                .count(),
            1,
            "dynamic and unknown controls need one mapped public-instance read:\n{}",
            output.code
        );
    }
}

mod lexical_scope;
