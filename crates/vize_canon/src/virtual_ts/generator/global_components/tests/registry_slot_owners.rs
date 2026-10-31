use crate::virtual_ts::VirtualTsOptions;
use vize_carton::{FxHashSet, String};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn emit(
    script: &str,
    template: &str,
    legacy_vue2: bool,
    options: VirtualTsOptions,
) -> (Vec<(u32, [String; 3])>, String) {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse_with_options(
        &allocator,
        template,
        vize_relief::options::ParserOptions {
            is_native_tag: Some(vize_carton::is_native_tag),
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let mut plan =
        super::super::GlobalComponentPlan::new((&summary, Some(&root)), legacy_vue2, true, None);
    let mut code = String::default();
    plan.emit(
        &mut code,
        &summary,
        &options,
        &FxHashSet::default(),
        &FxHashSet::default(),
        None,
    );
    let mut owners = plan.registry_slots.into_iter().collect::<Vec<_>>();
    owners.sort_by_key(|(start, _)| *start);
    (owners, code)
}

fn owners(
    script: &str,
    template: &str,
    legacy_vue2: bool,
    options: VirtualTsOptions,
) -> Vec<(u32, [String; 3])> {
    emit(script, template, legacy_vue2, options).0
}

#[test]
fn repeated_registry_usages_keep_exact_primary_and_fallback_before_declaration_dedup() {
    let template = concat!(
        "<RegistryChild v-slot=\"{ item }\">{{ item }}</RegistryChild>",
        "<RegistryChild v-slot=\"{ item }\">{{ item }}</RegistryChild>",
        "<registry-child v-slot=\"{ item }\">{{ item }}</registry-child>",
        "<RegistryChild />",
    );
    let starts = template
        .match_indices("<RegistryChild v-slot")
        .map(|(start, _)| {
            (
                start as u32,
                [
                    "RegistryChild".into(),
                    "RegistryChild".into(),
                    "RegistryChild".into(),
                ],
            )
        })
        .chain(
            template
                .match_indices("<registry-child v-slot")
                .map(|(start, _)| {
                    (
                        start as u32,
                        [
                            "RegistryChild".into(),
                            "registryChild".into(),
                            "registry-child".into(),
                        ],
                    )
                }),
        )
        .collect::<Vec<_>>();
    assert_eq!(
        owners("", template, false, VirtualTsOptions::default()),
        starts
    );
}

#[test]
fn a_named_dynamic_usage_cannot_claim_a_later_static_registry_usage_with_the_same_name() {
    let template = concat!(
        "<component :is=\"RegistryChild\" v-slot=\"{ item }\">{{ item }}</component>",
        "<RegistryChild v-slot=\"{ item }\">{{ item }}</RegistryChild>",
    );
    assert_eq!(
        owners("", template, false, VirtualTsOptions::default()),
        [(
            template.find("<RegistryChild").expect("one static usage") as u32,
            [
                "RegistryChild".into(),
                "RegistryChild".into(),
                "RegistryChild".into()
            ]
        )],
    );
}

#[test]
fn real_values_self_art_autoimports_and_vue2_keep_their_existing_owners() {
    let template = "<RegistryChild v-slot=\"{ item }\">{{ item }}</RegistryChild>";
    for (script, legacy, options) in [
        (
            "const RegistryChild = null;",
            false,
            VirtualTsOptions::default(),
        ),
        (
            "import RegistryChild from './component';",
            false,
            VirtualTsOptions::default(),
        ),
        (
            "defineOptions({ name: 'RegistryChild' });",
            false,
            VirtualTsOptions::default(),
        ),
        (
            "defineArt('./RegistryChild.vue', {});",
            false,
            VirtualTsOptions::default(),
        ),
        ("", true, VirtualTsOptions::default()),
        (
            "",
            false,
            VirtualTsOptions {
                external_template_bindings: vec!["RegistryChild".into()],
                ..Default::default()
            },
        ),
        (
            "",
            false,
            VirtualTsOptions {
                auto_import_stubs: vec!["declare const RegistryChild: any;".into()],
                ..Default::default()
            },
        ),
    ] {
        assert_eq!(
            owners(script, template, legacy, options),
            [],
            "{script}, legacy={legacy}"
        );
    }
    assert_eq!(
        owners(
            "",
            "<component :is=\"flag ? A : B\" v-slot=\"{ item }\">{{ item }}</component>",
            false,
            VirtualTsOptions::default()
        ),
        [],
    );
}

#[test]
fn vue3_stub_and_slot_provenance_share_pascal_camel_authored_order_without_changing_vue2() {
    let template = "<registry-child v-slot=\"{ item }\">{{ item }}</registry-child>";
    let (owners, vue3) = emit("", template, false, VirtualTsOptions::default());
    assert_eq!(
        owners,
        [(
            0,
            [
                "RegistryChild".into(),
                "registryChild".into(),
                "registry-child".into()
            ]
        )]
    );
    assert_eq!(
        vue3.lines()
            .filter_map(|line| line.strip_prefix("declare const "))
            .collect::<Vec<_>>(),
        [
            "registry_child: $GlobalComponent<\"RegistryChild\", \"registryChild\", \"registry-child\"> & $GlobalComponentFallback;"
        ]
    );
    let (owners, vue2) = emit("", template, true, VirtualTsOptions::default());
    assert!(owners.is_empty());
    assert_eq!(
        vue2.lines()
            .filter_map(|line| line.strip_prefix("declare const "))
            .collect::<Vec<_>>(),
        [
            "registry_child: $GlobalComponent<\"registry-child\", \"RegistryChild\"> & $GlobalComponentFallback;"
        ]
    );
}
