//! Factory sharing preserves the local ref registry and its demand boundary.
use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
    helpers::SHARED_PREAMBLE_DTS,
    scope::{REF_INSTANCE_HELPERS, REF_INSTANCE_HELPERS_HOISTED},
};
use vize_carton::{Allocator, String, cstr};
use vize_croquis::{Analyzer, AnalyzerOptions};

const FACTORY: &str = include_str!("../../helpers/declarations/ref-instance-factory.d.ts.txt");
const FORWARD: &str =
    "  type $RefInstanceFactory<_C> = import(\"vize:virtual-ts\").$RefInstanceFactory<_C>;\n";
const FACTORY_PREFIX: &str = "  type $RefInstanceFactory<_C> = ";

fn generate(script: &str, template: &str, hoisted: bool) -> String {
    let allocator = Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets_and_checks(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            hoist_shared_preamble: hoisted,
            ..Default::default()
        },
    )
    .code
}

fn factory_lines(code: &str) -> Vec<&str> {
    code.lines()
        .filter(|line| line.starts_with(FACTORY_PREFIX))
        .collect()
}

#[test]
fn sharing_changes_only_the_factory_body_in_the_local_helper_block() {
    let embedded_factory = cstr!("  {FACTORY}");
    assert_eq!(
        REF_INSTANCE_HELPERS
            .matches(embedded_factory.as_str())
            .count(),
        1
    );
    assert_eq!(REF_INSTANCE_HELPERS_HOISTED.matches(FORWARD).count(), 1);
    assert_eq!(
        REF_INSTANCE_HELPERS_HOISTED.replacen(FORWARD, &embedded_factory, 1),
        REF_INSTANCE_HELPERS,
    );
    assert_eq!(
        SHARED_PREAMBLE_DTS
            .lines()
            .filter(|line| line.starts_with("export type $RefInstanceFactory<"))
            .collect::<Vec<_>>(),
        vec![cstr!("export {}", FACTORY.trim_end())],
    );
}

#[test]
fn instantiated_component_refs_select_the_forwarder_only_in_hoisted_programs() {
    let script = "import Child from './Child.vue'\nimport { useTemplateRef } from 'vue'\nconst child = useTemplateRef('child')\n";
    let template = r#"<Child ref="child" :value="1" />"#;
    for hoisted in [false, true] {
        let code = generate(script, template, hoisted);
        let expected = if hoisted {
            FORWARD.trim_end().into()
        } else {
            cstr!("  {}", FACTORY.trim_end())
        };
        assert_eq!(factory_lines(&code), vec![expected.as_str()]);
        assert_eq!(
            code.lines()
                .filter(|line| line.starts_with("  type $TemplateRefInstance<"))
                .collect::<Vec<_>>(),
            REF_INSTANCE_HELPERS
                .lines()
                .filter(|line| line.starts_with("  type $TemplateRefInstance<"))
                .collect::<Vec<_>>(),
        );
    }
}

#[test]
fn unused_native_declared_foreign_and_typed_refs_do_not_gain_a_factory() {
    for (script, template) in [
        (
            "import Child from './Child.vue'",
            r#"<Child ref="child" :value="1" />"#,
        ),
        (
            "import { useTemplateRef } from 'vue'\nconst child = useTemplateRef('child')",
            r#"<div ref="child" />"#,
        ),
        (
            "import Child from './Child.vue'\nimport { useTemplateRef } from 'vue'\nconst child = useTemplateRef('child')",
            r#"<Child ref="child" />"#,
        ),
        (
            "import Child from './Child.vue'\nimport { useTemplateRef } from './foreign'\nconst child = useTemplateRef('child')",
            r#"<Child ref="child" :value="1" />"#,
        ),
        (
            "import Child from './Child.vue'\nimport { useTemplateRef } from 'vue'\nconst child = useTemplateRef<never>('child')",
            r#"<Child ref="child" :value="1" />"#,
        ),
    ] {
        for hoisted in [false, true] {
            assert_eq!(
                factory_lines(&generate(script, template, hoisted)),
                Vec::<&str>::new()
            );
        }
    }
}

#[test]
fn authored_module_factory_types_do_not_capture_the_local_forwarder() {
    let script = "import Child from './Child.vue'\nimport { useTemplateRef } from 'vue'\ntype $RefInstanceFactory = 'authored'\nconst child = useTemplateRef('child')\n";
    for hoisted in [false, true] {
        let code = generate(script, r#"<Child ref="child" :value="1" />"#, hoisted);
        assert_eq!(
            code.lines()
                .filter(|line| line.trim_start().starts_with("type $RefInstanceFactory"))
                .collect::<Vec<_>>(),
            vec![
                "type $RefInstanceFactory = 'authored'".into(),
                if hoisted {
                    FORWARD.trim_end().into()
                } else {
                    cstr!("  {}", FACTORY.trim_end())
                },
            ],
        );
    }
}

#[cfg(feature = "native")]
#[test]
fn batch_ref_factory_uses_pinned_generic_and_dom_inputs() {
    use crate::batch::VirtualProject;
    use std::{fs, path::Path};

    let inputs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/shared-ref-factory");
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    for (source, target) in [
        ("Child.vue.txt", "src/Child.vue"),
        ("Probe.vue.txt", "src/Probe.vue"),
        ("tsconfig.json.txt", "tsconfig.json"),
    ] {
        fs::copy(inputs.join(source), root.join(target)).unwrap();
    }
    let child = root.join("src/Child.vue");
    let probe = root.join("src/Probe.vue");
    let mut project = VirtualProject::new(root).unwrap();
    project
        .register_paths(&[child.clone(), probe.clone()])
        .unwrap();
    assert_eq!(
        factory_lines(&project.find_by_original(&child).unwrap().content),
        Vec::<&str>::new()
    );
    assert_eq!(
        factory_lines(&project.find_by_original(&probe).unwrap().content),
        vec![FORWARD.trim_end()]
    );
}
