use super::support::{RULE, SOURCE, configured, history_case, observe};
use vize_l0::config::VueVersion;
use vize_patina::{
    HelpLevel, Locale,
    native::{sfc::NativeSfcLintRefusal as Refusal, template::NativeTemplateLintRefusal},
};

#[test]
fn the_original_vue2_control_remains_clean_in_legacy_and_refused_in_native() {
    let case = history_case("slot-compat-vue2");
    assert_eq!(case.source, SOURCE);
    let linter = configured(Locale::En, HelpLevel::Full).with_vue_version(Some(VueVersion::V2));
    let original = linter.lint_sfc(&case.source, &case.filename);
    assert!(original.diagnostics.is_empty());
    assert_eq!(original.error_count, 0);
    assert_eq!(original.warning_count, 0);
    let observation = observe(&linter, &case, false);
    let frozen = include_str!("../snapshots/integration__lint_history__slot-compat-vue2.snap")
        .split_once("\n---\n")
        .unwrap()
        .1;
    assert_eq!(format!("{case:#?}\n{observation:#?}\n"), frozen);
    assert_eq!(
        linter
            .lint_native_sfc(&case.source, &case.filename)
            .unwrap_err(),
        Refusal::UnsupportedVueVersion {
            requested: VueVersion::V2
        },
    );
}

#[test]
fn unsupported_host_requests_refuse_before_producing_findings() {
    assert_eq!(
        configured(Locale::En, HelpLevel::Full)
            .with_vapor_mode(Some(true))
            .lint_native_sfc(SOURCE, "History.vue")
            .unwrap_err(),
        Refusal::UnsupportedVaporMode { requested: true },
    );
    assert_eq!(
        configured(Locale::En, HelpLevel::Full)
            .with_type_aware_lint(true)
            .lint_native_sfc(SOURCE, "History.vue")
            .unwrap_err(),
        Refusal::UnsupportedTypeAwareMode,
    );
    for unprovided in [
        "test/not-registered",
        "vue/no-v-html",
        "script/prefer-use-template-ref",
    ] {
        assert_eq!(
            configured(Locale::En, HelpLevel::Full)
                .with_enabled_rules(Some(vec![RULE.into(), unprovided.into()]))
                .lint_native_sfc(SOURCE, "History.vue")
                .unwrap_err(),
            Refusal::UnprovidedRule {
                rule: unprovided.into()
            },
        );
    }
}

#[test]
fn this_template_only_owner_does_not_certify_other_sfc_envelopes() {
    for source in [
        "<script setup lang='ts'>const value = 1</script><template><div slot='header'>Title</div></template>",
        "<script>export default {}</script><template><div slot='header'>Title</div></template>",
        "<template><div slot='header'>Title</div></template><style>.x{}</style>",
        "<template><div slot='header'>Title</div></template><docs>opaque</docs>",
        "<template src='./external.html'></template>",
        "<template lang='html'><div slot='header'>Title</div></template>",
        "<template><div slot='header'>Title</div></template><template><div/></template>",
    ] {
        assert!(
            matches!(
                configured(Locale::En, HelpLevel::Full).lint_native_sfc(source, "Envelope.vue"),
                Err(Refusal::UnsupportedEnvelope { .. } | Refusal::Descriptor { .. }),
            ),
            "{source}"
        );
    }
}

#[test]
fn static_only_capability_does_not_silently_omit_bound_slot_keys() {
    for key in [":slot", "v-bind:slot", "v-bind:slot.camel", ".slot"] {
        let source = format!("<template><div {key}=\"'header'\">Title</div></template>");
        assert!(
            matches!(
                configured(Locale::En, HelpLevel::Full).lint_native_sfc(&source, "Bindings.vue"),
                Err(Refusal::Template(
                    NativeTemplateLintRefusal::UnsupportedAttribute { .. }
                        | NativeTemplateLintRefusal::Header(_)
                )),
            ),
            "{source}"
        );
    }
}
