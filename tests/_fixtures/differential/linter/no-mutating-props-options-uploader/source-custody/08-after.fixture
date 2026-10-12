use super::super::{NoMutatingProps, NoMutatingPropsOptions};
use crate::diagnostic::Severity;
use crate::linter::LintResult;
use crate::{Linter, RuleRegistry};
use serde_json::{Value, json};

const RULE: &str = "vue/no-mutating-props";
const HELP: &str = "Use a local ref or emit an event instead of mutating props directly";
const UPLOADER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-mutating-props-options-uploader/input.vue.txt"
));
const CARD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-mutating-props-options-uploader/class-decorator.vue.txt"
));

fn configured(shallow_only: bool) -> Linter {
    Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_no_mutating_props_options(NoMutatingPropsOptions { shallow_only })
}

fn packet(result: &LintResult) -> Value {
    let diagnostics = result.diagnostics.iter().map(|diagnostic| json!({
        "rule": diagnostic.rule_name, "severity": diagnostic.severity,
        "message": diagnostic.message.as_str(), "start": diagnostic.start, "end": diagnostic.end,
        "help": diagnostic.help.as_deref(),
        "labels": diagnostic.labels.iter().map(|label| json!({"message":label.message.as_str(),"start":label.start,"end":label.end})).collect::<Vec<_>>(),
        "fix": diagnostic.fix.as_ref(),
    })).collect::<Vec<_>>();
    json!({"filename":result.filename.as_str(),"errorCount":result.error_count,"warningCount":result.warning_count,"diagnostics":diagnostics})
}

fn diagnostic(source: &str, expression: &str, key: &str, context: &str) -> Value {
    let start = source.find(expression).unwrap();
    json!({"rule":RULE,"severity":"error","start":start,"end":start+expression.len(),
        "message":format!("Unexpected mutation of prop '{key}' {context}"),
        "help":HELP,"labels":[],"fix":null})
}

fn expected(filename: &str, diagnostics: Vec<Value>) -> Value {
    json!({"filename":filename,"errorCount":diagnostics.len(),"warningCount":0,"diagnostics":diagnostics})
}

#[test]
fn whole_pinned_uploader_has_one_exact_api_and_cli_packet() {
    assert_eq!(UPLOADER.len(), 4000);
    let filename = "src/components/uploader.vue";
    let result = configured(false).lint_sfc(UPLOADER, filename);
    let finding = json!({"rule":RULE,"severity":"error","start":2975,"end":3019,
        "message":"Unexpected mutation of prop 'options' in <script>","help":HELP,"labels":[],"fix":null});
    assert_eq!(packet(&result), expected(filename, vec![finding]));
    let output = crate::format_results(
        &[result],
        &[(filename.into(), UPLOADER.into())],
        crate::OutputFormat::Json,
    );
    assert_eq!(
        serde_json::from_str::<Value>(&output).unwrap(),
        json!([{
            "file":filename,"errorCount":1,"warningCount":0,"messages":[{
                "ruleId":RULE,"ruleDocsPath":"docs/content/rules/vue.md","severity":2,
                "message":"[vize:vue/no-mutating-props] Unexpected mutation of prop 'options' in <script>",
                "line":116,"column":7,"endLine":116,"endColumn":51,"help":HELP
            }]
        }])
    );
    assert_eq!(
        packet(&configured(true).lint_sfc(UPLOADER, filename)),
        expected(filename, vec![])
    );
}

#[test]
fn genuine_instance_bodies_keep_order_ranges_static_names_and_shallow_policy() {
    let source = r#"<script>
export default {
  props: { items: Array, count: Number, settings: Object },
  mounted() { this.items.push(1); this['settings'][key] = 2; },
  created() { this.count++; delete this.items[0]; Object.assign(this.settings, { enabled: true }); },
  methods: { change() { this.items = []; this[dynamic] = 4; } },
  computed: { current: { get() { this.count = 5; return 0; }, set(v) { this.settings.enabled = v; } } },
  watch: { count: [{ handler() { delete this.count; } }, function() { this.items.splice(0, 1); }] }
}
</script>"#;
    let operations = [
        ("this.items.push(1)", "items", true),
        ("this['settings'][key] = 2", "settings", true),
        ("this.count++", "count", false),
        ("delete this.items[0]", "items", true),
        (
            "Object.assign(this.settings, { enabled: true })",
            "settings",
            true,
        ),
        ("this.items = []", "items", false),
        ("this.count = 5", "count", false),
        ("this.settings.enabled = v", "settings", true),
        ("delete this.count", "count", false),
        ("this.items.splice(0, 1)", "items", true),
    ];
    for shallow in [false, true] {
        let diagnostics = operations
            .iter()
            .filter(|(_, _, deep)| !shallow || !deep)
            .map(|(expression, key, _)| diagnostic(source, expression, key, "in <script>"))
            .collect();
        assert_eq!(
            packet(&configured(shallow).lint_sfc(source, "Probe.vue")),
            expected("Probe.vue", diagnostics)
        );
    }
}

#[test]
fn arrows_inherit_instance_this_but_foreign_functions_factories_and_classes_do_not() {
    let source = r#"<script>
export default {
  props: ['value'],
  setup() { this.value = 1; },
  mounted: () => { this.value = 2; },
  created() {
    (() => { this.value = 3; })();
    function foreign() { this.value = 4; }
    const object = { change() { this.value = 5; }, field: () => { this.value = 6; } };
    class Foreign { field = (this.value = 7); method() { this.value = 8; } }
    const alias = this; alias.value = 9;
  },
  methods: { save: () => { this.value = 10; } },
  computed: { read: () => { this.value = 11; }, get accessor() { this.value = 12; } }
}
</script>"#;
    let diagnostics = ["this.value = 3", "this.value = 6"]
        .into_iter()
        .map(|expression| diagnostic(source, expression, "value", "in <script>"))
        .collect();
    assert_eq!(
        packet(&configured(false).lint_sfc(source, "Probe.vue")),
        expected("Probe.vue", diagnostics)
    );
    let factory =
        "<script>export default { props: { value: { default() { this.value = 13; } } } }</script>";
    assert_eq!(
        packet(&configured(false).lint_sfc(factory, "Probe.vue")),
        expected("Probe.vue", vec![])
    );
}

#[test]
fn only_known_literal_component_options_grant_prop_receiver_provenance() {
    for body in [
        "const other = { props: ['value'], created() { this.value = 1; } }; export default {}",
        "function defineComponent(value) { return value; } export default defineComponent({ props: ['value'], created() { this.value = 1; } })",
        "const options = { props: ['value'], created() { this.value = 1; } }; export default options",
        "export default { props: ['value'], created() { this.value = 1; }, ...foreign }",
        "export default { props: ['value'], created() { this.value = 1; }, [unknown]: null }",
        "export default { props: ['value'], props: [], created() { this.value = 1; } }",
        "export default { props: ['value'], created() { other.value = 1; this.other = 2; }, text: 'this.value = 3' }",
    ] {
        let source = format!("<script>{body}</script>");
        assert_eq!(
            packet(&configured(false).lint_sfc(&source, "Probe.vue")),
            expected("Probe.vue", vec![])
        );
    }
    let source = r#"<script>export default { ...foreign, ['props']: { ['value']: Number }, created() { this[`value`] = 4; } }</script>"#;
    assert_eq!(
        packet(&configured(false).lint_sfc(source, "Probe.vue")),
        expected(
            "Probe.vue",
            vec![diagnostic(
                source,
                "this[`value`] = 4",
                "value",
                "in <script>"
            )]
        )
    );
}

#[test]
fn shadowed_object_assign_never_inherits_the_global_mutator() {
    for prefix in [
        "const Object = foreign;",
        "import Object from 'foreign';",
        "function unrelated(Object) {}",
        "const { Object } = foreign;",
    ] {
        let source = format!(
            "<script>{prefix} export default {{ props: ['value'], created() {{ Object.assign(this.value, {{}}); }} }}</script>"
        );
        assert_eq!(
            packet(&configured(false).lint_sfc(&source, "Probe.vue")),
            expected("Probe.vue", vec![])
        );
    }
    let source = "<script>export default { props: ['value'], created(Object) { Object.assign(this.value, {}); this.value = 1; } }</script>";
    assert_eq!(
        packet(&configured(false).lint_sfc(source, "Probe.vue")),
        expected(
            "Probe.vue",
            vec![diagnostic(source, "this.value = 1", "value", "in <script>")]
        )
    );
}

#[test]
fn setup_template_and_class_decorator_packets_remain_independent() {
    let setup = "<script setup>const props = defineProps(['value']); props.value = 1;</script>";
    assert_eq!(
        packet(&configured(false).lint_sfc(setup, "Probe.vue")),
        expected(
            "Probe.vue",
            vec![diagnostic(
                setup,
                "props.value = 1",
                "props.value",
                "in <script setup>"
            )]
        )
    );
    let template = "<script setup>defineProps(['value']);</script><template><input v-model=\"value\" /></template>";
    let start = template.find("v-model=\"value\"").unwrap() + "v-model=\"".len();
    let finding = json!({"rule":RULE,"severity":"error","start":start,"end":start+5,
        "message":"Unexpected mutation of prop 'value' via v-model","help":HELP,"labels":[],"fix":null});
    assert_eq!(
        packet(&configured(false).lint_sfc(template, "Probe.vue")),
        expected("Probe.vue", vec![finding])
    );
    let filename = "src/views/home/widgets/card.vue";
    let result = configured(false).lint_sfc(CARD, filename);
    let start = CARD.find("v-model=\"notebook.notes\"").unwrap() + "v-model=\"".len();
    assert_eq!(
        packet(&result),
        expected(
            filename,
            vec![json!({"rule":RULE,"severity":"error",
        "message":"Unexpected mutation of prop 'notebook.notes' via v-model","start":start,"end":start+14,
        "help":HELP,"labels":[],"fix":null})]
        )
    );
}

#[test]
fn one_existing_rule_owns_activation_severity_and_all_configured_options() {
    let source =
        "<script>export default { props: ['value'], created() { this.value = 1; } }</script>";
    let linter = configured(false);
    assert_eq!(
        linter
            .registry
            .rule_names()
            .iter()
            .filter(|name| **name == RULE)
            .count(),
        1
    );
    assert_eq!(
        crate::linter::builtin_script_rules()
            .iter()
            .filter(|entry| entry.name == RULE)
            .count(),
        0
    );
    assert_eq!(
        packet(
            &linter
                .with_disabled_rules(vec![RULE.into()])
                .lint_sfc(source, "Probe.vue")
        ),
        expected("Probe.vue", vec![])
    );
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoMutatingProps::new(NoMutatingPropsOptions {
        shallow_only: false,
    })));
    assert_eq!(
        packet(&Linter::with_registry(registry).lint_sfc(source, "Probe.vue")),
        expected(
            "Probe.vue",
            vec![diagnostic(source, "this.value = 1", "value", "in <script>")]
        )
    );
    let result = configured(false)
        .with_rule_severity_overrides(vec![(RULE.into(), Severity::Warning)])
        .lint_sfc(source, "Probe.vue");
    let mut value = diagnostic(source, "this.value = 1", "value", "in <script>");
    value["severity"] = json!("warning");
    assert_eq!(
        packet(&result),
        json!({"filename":"Probe.vue","errorCount":0,"warningCount":1,"diagnostics":[value]})
    );
}

#[test]
fn configured_namesake_cannot_inherit_the_private_instance_callback() {
    struct Namesake;
    impl crate::Rule for Namesake {
        fn meta(&self) -> &'static crate::RuleMeta {
            &super::super::META
        }
    }
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(Namesake));
    let source =
        "<script>export default { props: ['value'], created() { this.value = 1; } }</script>";
    assert_eq!(
        packet(&Linter::with_registry(registry).lint_sfc(source, "Probe.vue")),
        expected("Probe.vue", vec![])
    );
}

#[test]
fn mixed_blocks_preserve_complete_existing_packets_and_utf8_byte_offsets() {
    let existing = "<!-- 日本語 --><script setup>const props = defineProps(['setupValue']); props.setupValue = 1;</script><template><input v-model=\"setupValue\" /></template>";
    let source = format!(
        "{existing}<script>export default {{ props: ['normalValue'], created() {{ this.normalValue = 2; }} }}</script>"
    );
    let mut wanted = packet(&configured(false).lint_sfc(existing, "Mixed.vue"));
    assert_eq!(wanted["errorCount"], json!(2));
    wanted["errorCount"] = json!(3);
    wanted["diagnostics"]
        .as_array_mut()
        .unwrap()
        .push(diagnostic(
            &source,
            "this.normalValue = 2",
            "normalValue",
            "in <script>",
        ));
    assert_eq!(
        packet(&configured(false).lint_sfc(&source, "Mixed.vue")),
        wanted
    );
}
