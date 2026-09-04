//! The Options API public-instance form for unknown template names (#3888):
//! opted in only for a Vue 3 plain default export, and never for names the
//! script itself declares.

use crate::virtual_ts::{
    TemplateGlobal, VirtualTsOptions, generate_virtual_ts_with_offsets_options_api,
};

mod data_assignment;

#[test]
fn test_options_api_template_bindings_use_default_instance_type() {
    let script = r#"export default {
    props: {
        initial: Number,
    },
    data() {
        return { count: 0 }
    },
    computed: {
        doubled() {
            return this.count * 2
        },
    },
    methods: {
        bump() {
            return this.count + 1
        },
    },
}
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ count }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert!(
        output.code.contains("type __VizeOptionsInstance<T>"),
        "expected Options API instance helper:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("var count: __VizeOptionsBinding<typeof __default__, \"count\">"),
        "expected data binding to reference the default component instance:\n{}",
        output.code
    );
    assert!(
        !output.code.contains("const count: any = undefined as any;"),
        "template data binding must not be emitted as a fixed any:\n{}",
        output.code
    );
    assert!(
        output.code.contains(", __default__ }"),
        "the typed authored component must escape setup for declaration emit:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "type __VizeAuthoredComponent = Awaited<ReturnType<typeof __setup>>[\"__default__\"]"
        ),
        "expected a module-scope authored component alias:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("type __VizeComponentInstance = Omit<__VizeAuthoredInstance, '$props' | '$emit' | '$slots'> & {"),
        "the public instance must retain Options API members without broadening normalized Vue members:\n{}",
        output.code
    );
}

/// A `namespace` the plain script declares stays resolvable from template scope
/// (the generated module keeps the declaration in a scope that encloses the
/// template closure), so it is not an unknown template name: a public-instance
/// property access for it would invent a `TS2339` vue-tsc never reports. A name
/// the script does not declare still resolves on the instance.
#[test]
fn test_options_api_script_declared_namespace_is_not_checked_on_the_instance() {
    let script = r#"namespace Bare {
  export const label = 'bare'
}

export default {
    name: 'Namespaces',
}
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) =
        vize_armature::parse(&allocator, "<div>{{ Bare.label }} {{ missingThing }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert!(
        !output.code.contains("__vize_template_instance.Bare"),
        "a script-declared namespace must not be checked against the public instance:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("__vize_template_instance.missingThing"),
        "a name the script never declares still resolves on the public instance:\n{}",
        output.code
    );
}

/// `<script lang="ts">` may hold JSX, which only the TSX dialect parses: the
/// script's top-level names must still be found so they are not checked on the
/// public instance.
#[test]
fn test_options_api_tsx_script_names_are_not_checked_on_the_instance() {
    let script = r#"const icon = () => <span>ok</span>

export default {
    name: 'Tsx',
}
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ icon }} {{ missingThing }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert!(
        !output.code.contains("__vize_template_instance.icon"),
        "a name a TSX script declares must not be checked against the public instance:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("__vize_template_instance.missingThing"),
        "a name the script never declares still resolves on the public instance:\n{}",
        output.code
    );
}

/// A script the parser cannot read leaves its top-level names unknown, so no
/// template name may be classified against the public instance: a name the
/// broken script does declare would otherwise gain an invented `TS2339`.
#[test]
fn test_options_api_unparseable_script_disables_the_instance_form() {
    let script = r#"const broken = (

export default {
    name: 'Broken',
}
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ missingThing }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert!(
        !output.code.contains("__vize_template_instance"),
        "an unparseable script must keep the public-instance form off:\n{}",
        output.code
    );
}

/// A `<script setup>` block next to the plain script keeps the form off: there
/// the plain default export carries only the options the setup form cannot
/// express (`inheritAttrs`, `name`, ...), so its instance type describes none of
/// the template's real bindings, and every setup binding or auto-imported name
/// would gain an invented `TS2339` (measured on the Elk and Nuxt UI surfaces).
#[test]
fn test_options_api_script_setup_next_to_plain_script_disables_the_instance_form() {
    let setup = r#"const props = defineProps<{ toaster: boolean }>()
"#;
    let script = r#"export default {
    inheritAttrs: false,
}
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ toaster }} {{ missingThing }}</div>");
    // The split-script shape the SFC pipeline builds: the setup summary is the
    // base, the plain script is merged into it, then the template is drawn.
    let options = vize_croquis::AnalyzerOptions::full();
    let mut setup_analyzer = vize_croquis::Analyzer::with_options(options).with_options_api();
    setup_analyzer.analyze_script_setup(setup);
    let mut summary = setup_analyzer.finish();
    let mut plain_analyzer = vize_croquis::Analyzer::with_options(options).with_options_api();
    plain_analyzer.analyze_script_plain(script);
    summary.merge_plain_script(plain_analyzer.finish());
    let mut analyzer =
        vize_croquis::Analyzer::with_summary(options, summary, true).with_options_api();
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert!(
        !output.code.contains("__vize_template_instance"),
        "a `<script setup>` block must keep the public-instance form off:\n{}",
        output.code
    );
}

/// The same split-script shape keeps the authored component out of the emitted
/// export. Next to a `<script setup>` block the plain default export is an
/// options fragment (`inheritAttrs`, `name`, shared helper objects), not the
/// component, so intersecting it into `__vize_component__` breaks the export's
/// construct-signature inference: Misskey's `popup(MkAutocomplete, props, {
/// done: res => ... })` lost every listener's contextual type (`TS7006`).
#[test]
fn test_script_setup_next_to_plain_script_drops_the_authored_component() {
    let setup = r#"const props = defineProps<{ toaster: boolean }>()
"#;
    let script = r#"export default {
    inheritAttrs: false,
}
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ toaster }}</div>");
    let options = vize_croquis::AnalyzerOptions::full();
    let mut setup_analyzer = vize_croquis::Analyzer::with_options(options).with_options_api();
    setup_analyzer.analyze_script_setup(setup);
    let mut summary = setup_analyzer.finish();
    let mut plain_analyzer = vize_croquis::Analyzer::with_options(options).with_options_api();
    plain_analyzer.analyze_script_plain(script);
    summary.merge_plain_script(plain_analyzer.finish());
    let mut analyzer =
        vize_croquis::Analyzer::with_summary(options, summary, true).with_options_api();
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );

    assert!(
        !output.code.contains("__VizeAuthoredComponent"),
        "a `<script setup>` block must keep the authored component aliases off:\n{}",
        output.code
    );
    assert!(
        output.code.contains("type __VizeComponentInstance = {\n"),
        "the public instance must stay the normalized shape:\n{}",
        output.code
    );
}

/// A configured `globalTypes` entry already declares the name in the template
/// closure, so the `var` the instance form hoists would redeclare it
/// (`TS2451 Cannot redeclare block-scoped variable`).
#[test]
fn test_options_api_configured_globals_are_not_redeclared_by_the_instance_form() {
    let script = r#"export default {
    name: 'Globals',
}
"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ toThousandFilter(1) }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let options = VirtualTsOptions {
        template_globals: vec![TemplateGlobal {
            name: "toThousandFilter".into(),
            type_annotation: "any".into(),
            default_value: "undefined as any".into(),
        }],
        ..Default::default()
    };
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &options,
    );

    assert!(
        !output
            .code
            .contains("__vize_template_instance.toThousandFilter"),
        "a configured template global must not be checked against the public instance:\n{}",
        output.code
    );
    assert!(
        !output.code.contains("var toThousandFilter"),
        "a configured template global must not be redeclared by the instance form:\n{}",
        output.code
    );
}
