use super::{ScriptParserOptions, parse_script_with_options};
use vize_relief::BindingType;

fn parse_legacy(source: &str) -> super::ScriptParseResult {
    parse_script_with_options(
        source,
        ScriptParserOptions {
            options_api: false,
            legacy_vue2: true,
        },
    )
}

fn parse_vue3_options(source: &str) -> super::ScriptParseResult {
    parse_script_with_options(
        source,
        ScriptParserOptions {
            options_api: true,
            legacy_vue2: false,
        },
    )
}

#[test]
fn options_filters_are_exposed_only_in_legacy_mode() {
    let source = r#"
export default {
  filters: {
    formatStatus(value) { return String(value) }
  }
}
"#;
    assert_eq!(
        parse_legacy(source).bindings.get("formatStatus"),
        Some(BindingType::Options)
    );
    assert!(!parse_vue3_options(source).bindings.contains("formatStatus"));
}

#[test]
fn filters_follow_same_file_mixins_and_extends() {
    let source = r#"
const BaseComponent = {
  filters: {
    fromBase(value) { return value }
  }
}
const LocalMixin = {
  filters: {
    fromMixin(value) { return value }
  }
}
export default {
  extends: BaseComponent,
  mixins: [LocalMixin],
  filters: {
    fromComponent(value) { return value }
  }
}
"#;
    let legacy = parse_legacy(source);
    let vue3 = parse_vue3_options(source);
    for name in ["fromBase", "fromMixin", "fromComponent"] {
        assert!(
            legacy.bindings.contains(name),
            "missing legacy filter {name}"
        );
        assert!(
            !vue3.bindings.contains(name),
            "Vue 3 must not expose legacy filter {name}"
        );
    }
}

fn assert_typed_mixin_array(source: &str) {
    let legacy = parse_legacy(source);
    let vue3 = parse_vue3_options(source);

    assert_eq!(
        legacy.bindings.get("inheritedMethod"),
        Some(BindingType::Options)
    );
    assert_eq!(
        vue3.bindings.get("inheritedMethod"),
        Some(BindingType::Options)
    );
    assert_eq!(
        legacy.bindings.get("inheritedFilter"),
        Some(BindingType::Options)
    );
    assert!(!vue3.bindings.contains("inheritedFilter"));
}

#[test]
fn mixins_unwrap_as_const_arrays() {
    assert_typed_mixin_array(
        r#"
const LocalMixin = {
  methods: { inheritedMethod() {} },
  filters: { inheritedFilter(value) { return value } }
}
export default { mixins: ([LocalMixin] as const) }
"#,
    );
}

#[test]
fn mixins_unwrap_satisfies_arrays() {
    assert_typed_mixin_array(
        r#"
const LocalMixin = {
  methods: { inheritedMethod() {} },
  filters: { inheritedFilter(value) { return value } }
}
export default { mixins: ([LocalMixin] satisfies readonly object[]) }
"#,
    );
}

#[test]
fn mixins_unwrap_angle_bracket_asserted_arrays() {
    assert_typed_mixin_array(
        r#"
const LocalMixin = {
  methods: { inheritedMethod() {} },
  filters: { inheritedFilter(value) { return value } }
}
export default { mixins: (<const>[LocalMixin]) }
"#,
    );
}

#[test]
fn class_component_decorator_filters_are_legacy_only() {
    let source = r#"
import Vue from 'vue'
import Component from 'vue-class-component'

@Component({
  filters: {
    formatStatus(value) { return String(value) }
  }
})
export default class App extends Vue {}
"#;
    assert_eq!(
        parse_legacy(source).bindings.get("formatStatus"),
        Some(BindingType::Options)
    );
    assert!(!parse_vue3_options(source).bindings.contains("formatStatus"));
}
