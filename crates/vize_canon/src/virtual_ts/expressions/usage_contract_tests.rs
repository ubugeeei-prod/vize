//! A plain component binding reads its props and check props from the shared
//! `$UsageProps` / `$UsageCheckExtras` helpers; every other target keeps its
//! per-usage aliases.
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

fn virtual_ts(script: &str, template: &str) -> std::string::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    generate_virtual_ts(&summary, Some(script), Some(&root), 0)
        .code
        .as_str()
        .to_string()
}

/// The declared alias name of a `type __X = ...;` line.
fn alias_name(line: &str) -> Option<&str> {
    line.strip_prefix("type __")?.split(" = ").next()
}

fn props_aliases(code: &str) -> Vec<&str> {
    code.lines()
        .map(str::trim)
        .filter(|line| {
            alias_name(line).is_some_and(|name| {
                name.contains("_Props_")
                    && !name.contains("_ValueProps_")
                    && !name.contains("_CheckProps_")
            })
        })
        .collect()
}

fn check_props_aliases(code: &str) -> Vec<&str> {
    code.lines()
        .map(str::trim)
        .filter(|line| alias_name(line).is_some_and(|name| name.contains("_CheckProps_")))
        .collect()
}

const PER_USAGE_PROPS: &str = " extends { __vizeCheck: infer __F } ? ($IsAny<__F> extends true ? ";
const PER_USAGE_GATE: &str = "$FallthroughProps<";

#[test]
fn a_plain_binding_reads_the_shared_usage_contract() {
    let code = virtual_ts(
        "import Child from './Child.vue'\nconst n = 1",
        r#"<Child :a="n" /><Child :b="n" />"#,
    );
    assert!(
        code.contains("  type __Child_Props_0 = $UsageProps<typeof Child>;\n"),
        "{code}"
    );
    assert!(
        code.contains(
            "  type __Child_CheckProps_0 = __Child_Props_0 & $UsageCheckExtras<__Child_Component_0>;\n"
        ),
        "{code}"
    );
    // The local helpers declare both, right after `$PropValueOr`.
    assert!(code.contains("  type $UsageProps<C> = 0 extends (1 & C) ? $ComponentRawProps<C> : [C] extends [{ __vizeCheck: infer __F }]"), "{code}");
    assert!(code.contains("  type $UsageCheckExtras<C> = $EmitListeners<C> & ($FallthroughProps<C> extends infer __F"), "{code}");
    for alias in props_aliases(&code) {
        assert!(!alias.contains(PER_USAGE_PROPS), "{alias}");
    }
    for alias in check_props_aliases(&code) {
        assert!(!alias.contains(PER_USAGE_GATE), "{alias}");
    }
}

#[test]
fn a_dynamic_is_target_keeps_its_per_usage_aliases() {
    let code = virtual_ts(
        "import A from './A.vue'\nimport B from './B.vue'\nconst flag = true",
        r#"<component :is="flag ? A : B" :a="1" />"#,
    );
    let props = props_aliases(&code);
    assert!(!props.is_empty(), "{code}");
    for alias in props {
        assert!(alias.contains("__vize_dynamic_is_"), "{alias}");
        assert!(alias.contains(PER_USAGE_PROPS), "{alias}");
        assert!(!alias.contains("$UsageProps<"), "{alias}");
    }
    for alias in check_props_aliases(&code) {
        assert!(alias.contains(PER_USAGE_GATE), "{alias}");
        assert!(!alias.contains("$UsageCheckExtras<"), "{alias}");
    }
}

#[test]
fn a_static_namespace_member_reads_the_usage_contract() {
    let code = virtual_ts(
        "import * as icons from './icons'\nconst n = 1",
        r#"<icons.Check :size="n" />"#,
    );
    assert!(
        code.contains("  type __icons_Check_Props_0 = $UsageProps<typeof icons.Check>;\n"),
        "{code}"
    );
}

#[test]
fn a_member_is_target_keeps_its_per_usage_aliases() {
    let code = virtual_ts(
        "import Child from './Child.vue'\nconst def = { view: Child }",
        r#"<component :is="def.view" :size="1" />"#,
    );
    let props = props_aliases(&code);
    assert!(!props.is_empty(), "{code}");
    for alias in props {
        assert!(alias.contains(PER_USAGE_PROPS), "{alias}");
        assert!(!alias.contains("$UsageProps<"), "{alias}");
    }
    for alias in check_props_aliases(&code) {
        assert!(alias.contains(PER_USAGE_GATE), "{alias}");
        assert!(!alias.contains("$UsageCheckExtras<"), "{alias}");
    }
}

#[test]
fn slot_host_instance_props_keep_their_per_usage_aliases() {
    let code = virtual_ts(
        "import Child from './Child.vue'\nconst n = 1",
        r#"<Child :a="n" v-slot="{ item }">{{ item }}</Child>"#,
    );
    let props = props_aliases(&code);
    assert!(!props.is_empty(), "{code}");
    for alias in props {
        assert!(alias.contains("__vize_slot_host_"), "{alias}");
        assert!(!alias.contains("$UsageProps<"), "{alias}");
    }
    for alias in check_props_aliases(&code) {
        assert!(alias.contains(PER_USAGE_GATE), "{alias}");
        assert!(!alias.contains("$UsageCheckExtras<"), "{alias}");
    }
}
