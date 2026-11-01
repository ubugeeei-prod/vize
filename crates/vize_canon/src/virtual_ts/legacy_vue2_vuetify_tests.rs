#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use vize_carton::config::VueVersion;
use vize_croquis::{Analyzer, AnalyzerOptions};

use super::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions,
    generate_virtual_ts_with_offsets, generate_virtual_ts_with_offsets_and_checks,
};

fn standard_virtual_ts(script: &str, template: &str, options: &VirtualTsOptions) -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets(&analyzer.finish(), Some(script), Some(&root), 0, 0, options)
        .code
        .to_string()
}

fn legacy_virtual_ts(script: &str, template: &str, options: &VirtualTsOptions) -> String {
    legacy_virtual_ts_with_checks(script, template, options, VirtualTsCheckOptions::default())
}

fn legacy_virtual_ts_with_checks(
    script: &str,
    template: &str,
    options: &VirtualTsOptions,
    check_options: VirtualTsCheckOptions,
) -> String {
    let allocator = vize_carton::Allocator::new();
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
        options,
        VirtualTsGenerationOptions {
            legacy_vue2: true,
            check_options,
            ..Default::default()
        },
    )
    .code
    .to_string()
}

#[test]
fn legacy_vue2_fallthrough_listener_payloads_remain_explicit_only() {
    let script = r#"import MyItem from './MyItem.vue'
function onClick(event: string) {
  void event
}
"#;
    let template = r#"<MyItem @click="onClick" />"#;
    let generated = |enabled, specified| {
        legacy_virtual_ts_with_checks(
            script,
            template,
            &VirtualTsOptions::default(),
            VirtualTsCheckOptions {
                fallthrough_attributes: enabled,
                fallthrough_attributes_specified: specified,
                ..Default::default()
            },
        )
    };
    let omitted = generated(false, false);
    let explicit_false = generated(false, true);
    let explicit_true = generated(true, true);
    insta::assert_snapshot!(
        "legacy_fallthrough_listener_modes",
        format!(
            "=== omitted ===\n{omitted}\n=== explicit false ===\n{explicit_false}\n=== explicit true ===\n{explicit_true}"
        )
    );
}

fn dialect_virtual_ts(
    script: &str,
    template: &str,
    options: &VirtualTsOptions,
    dialect: VueVersion,
) -> String {
    let allocator = vize_carton::Allocator::new();
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
        options,
        VirtualTsGenerationOptions {
            dialect,
            ..Default::default()
        },
    )
    .code
    .to_string()
}

#[test]
fn legacy_vue2_component_event_fallback_stays_permissive() {
    let script = r#"import VDatePicker from './VDatePicker.vue'
function updateDate(newDate: string) {
  void newDate
}
"#;
    let template = r#"<VDatePicker @input="updateDate" />"#;

    let standard = standard_virtual_ts(script, template, &VirtualTsOptions::default());

    let legacy = legacy_virtual_ts(script, template, &VirtualTsOptions::default());
    insta::assert_snapshot!(
        "legacy_vue2_component_event_fallback_stays_permissive",
        format!("=== standard ===\n{standard}\n=== legacy ===\n{legacy}")
    );
}

#[test]
fn legacy_vue2_component_event_dollar_event_alias_stays_scoped() {
    let script = r#"import VDatePicker from './VDatePicker.vue'
function updateDate(newDate: string) {
  void newDate
}
"#;
    let template = r#"<VDatePicker @input="updateDate($event)" />"#;

    let legacy = legacy_virtual_ts(script, template, &VirtualTsOptions::default());
    insta::assert_snapshot!(
        "legacy_vue2_component_event_dollar_event_alias_stays_scoped",
        legacy
    );
}

#[test]
fn legacy_vue2_component_event_object_payloads_stay_permissive() {
    let script = r#"type Manager = { id: string }
declare const DataTable: { new (): { $props: { "onClick:Row"?: (item: Object) => unknown } } }
function moveToDetail(item: Manager) {
  void item
}
"#;
    let template = r#"<DataTable @click:row="moveToDetail" />"#;

    let legacy = legacy_virtual_ts(script, template, &VirtualTsOptions::default());
    insta::assert_snapshot!(
        "legacy_vue2_component_event_object_payloads_stay_permissive",
        legacy
    );
}

#[test]
fn legacy_vue2_unresolved_component_event_payloads_stay_variadic() {
    let script = r#"import CalendarRange from './CalendarRange.vue'
function updateRange(
  calendarDate: { start: string; end: string },
  highlightSelectedDate: () => void,
) {
  void calendarDate
  highlightSelectedDate()
}
function submitHooks(hooks: { before?: () => void; after?: () => void }) {
  hooks.before?.()
}
"#;
    let template = r#"<CalendarRange @range-change="updateRange" @submit="submitHooks" />"#;

    let legacy = legacy_virtual_ts(script, template, &VirtualTsOptions::default());
    insta::assert_snapshot!(
        "legacy_vue2_unresolved_component_event_payloads_stay_variadic",
        legacy
    );
}

#[test]
fn legacy_vue2_checks_external_values_with_loose_extra_attributes() {
    let script = "const width = 320\n";
    let template = r#"<VDatePicker :width="width" />"#;
    let options = VirtualTsOptions {
        auto_import_stubs: vec![
            "declare const VDatePicker: { new (): { $props: { mini?: boolean } } };".into(),
        ],
        external_template_bindings: vec!["VDatePicker".into()],
        ..Default::default()
    };

    let standard = standard_virtual_ts(script, template, &options);

    let legacy = legacy_virtual_ts(script, template, &options);
    insta::assert_snapshot!(
        "legacy_vue2_checks_external_values_with_loose_extra_attributes",
        format!("=== standard ===\n{standard}\n=== legacy ===\n{legacy}")
    );
}

#[test]
fn legacy_vue2_unresolved_keyed_props_are_unchecked() {
    let script = r#"
type Props = { mini?: boolean } & { dense?: boolean }
defineProps<Props>()
"#;
    let template = r#"<v-select :width="width" :hide-details="hideDetails" />"#;

    let standard = standard_virtual_ts(script, template, &VirtualTsOptions::default());

    let legacy = legacy_virtual_ts(script, template, &VirtualTsOptions::default());
    insta::assert_snapshot!(
        "legacy_vue2_unresolved_keyed_props_are_unchecked",
        format!("=== standard ===\n{standard}\n=== legacy ===\n{legacy}")
    );
}

#[test]
fn vue2_dialect_unresolved_keyed_props_are_unchecked() {
    let script = r#"
type Props = { mini?: boolean } & { dense?: boolean }
defineProps<Props>()
"#;
    let template = r#"<div>{{ isMini }} {{ hideDetails }} {{ chips }} {{ width }}</div>"#;

    let standard = standard_virtual_ts(script, template, &VirtualTsOptions::default());

    let vue2 = dialect_virtual_ts(
        script,
        template,
        &VirtualTsOptions::default(),
        VueVersion::V2_7,
    );
    insta::assert_snapshot!(
        "vue2_dialect_unresolved_keyed_props_are_unchecked",
        format!("=== standard ===\n{standard}\n=== vue2 ===\n{vue2}")
    );
}

/// A Vue 2 dialect declares `$listeners` and friends in the template context,
/// so the instance-global emitter must not declare them a second time in the
/// same closure (`TS2451`). Vue 3 has no such declaration, so there the
/// public instance read stays the only one, and reports the `TS2339` `vue-tsc`
/// reports for a member Vue 3 removed.
#[test]
fn vue2_instance_members_are_declared_once() {
    let script = "";
    let template = r#"<div>{{ $listeners }} {{ $scopedSlots }}</div>"#;

    let legacy = legacy_virtual_ts(script, template, &VirtualTsOptions::default());
    let vue2 = dialect_virtual_ts(
        script,
        template,
        &VirtualTsOptions::default(),
        VueVersion::V2_7,
    );
    let standard = standard_virtual_ts(script, template, &VirtualTsOptions::default());
    insta::assert_snapshot!(
        "vue2_instance_member_declarations",
        format!(
            "=== legacy ===\n{legacy}\n=== Vue2 dialect ===\n{vue2}\n=== standard ===\n{standard}"
        )
    );
}

#[test]
fn legacy_external_inline_callbacks_keep_the_existing_loose_route() {
    let options = VirtualTsOptions {
        auto_import_stubs: vec!["declare const ExternalInput: any;".into()],
        external_template_bindings: vec!["ExternalInput".into()],
        ..Default::default()
    };
    let legacy = legacy_virtual_ts(
        "",
        r#"<ExternalInput :rules="[(value) => !!value]" />"#,
        &options,
    );
    insta::assert_snapshot!(
        "legacy_external_inline_callbacks_keep_the_existing_loose_route",
        legacy
    );
}

#[test]
fn legacy_local_import_shadow_keeps_the_original_strict_check_tail() {
    let options = VirtualTsOptions {
        auto_import_stubs: vec!["declare const ExternalInput: any;".into()],
        external_template_bindings: vec!["ExternalInput".into()],
        ..Default::default()
    };
    let legacy = legacy_virtual_ts(
        "import ExternalInput from './Input.vue'",
        r#"<ExternalInput :count="'wrong'" />"#,
        &options,
    );
    insta::assert_snapshot!(
        "legacy_local_import_shadow_keeps_the_original_strict_check_tail",
        legacy
    );
}
