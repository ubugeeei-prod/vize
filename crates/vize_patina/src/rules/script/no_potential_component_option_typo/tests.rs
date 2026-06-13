use super::NoPotentialComponentOptionTypo;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoPotentialComponentOptionTypo));
    linter
}

#[test]
fn test_valid_all_known_options() {
    let source = "export default { name: 'Foo', components: {}, props: ['foo'], \
             data() { return {} }, computed: {}, methods: {}, watch: {}, mounted() {} }";
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_no_options_object() {
    let source = r#"
import { ref } from 'vue'
const count = ref(0)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_far_custom_option_ignored() {
    // `myCustomPluginOption` is far from every known option -> not flagged.
    let source = r#"
export default {
  methods: {},
  myCustomPluginOption: true
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_methods_typo() {
    let source = r#"
export default {
  method: {
    foo() {}
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_props_typo() {
    let source = r#"
export default {
  prop: ['foo']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_distance_one_edits_flagged() {
    // Substitution (`computes`->`computed`), insertion (`mountd`->`mounted`)
    // and deletion (`watchh`->`watch`) are each edit-distance 1.
    for src in [
        "export default { computes: { foo() { return 1 } } }",
        "export default { mountd() {} }",
        "export default { watchh: {} }",
    ] {
        let result = create_linter().lint(src, 0);
        assert_eq!(result.error_count, 1, "expected typo in: {src}");
    }
}

#[test]
fn test_define_component_typo() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  computd: {
    foo() { return 1 }
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_identifier_bound_export_typo() {
    let source = r#"
const component = {
  methos: {}
}

export default component
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_multiple_typos_reported() {
    let source = r#"
export default {
  method: {},
  prop: ['foo'],
  compute: {}
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 3);
}

#[test]
fn test_exact_known_options_never_flag() {
    // `extends` and `expose` are 1 apart from each other but both known,
    // so neither should be reported.
    let source = r#"
export default {
  extends: {},
  expose: []
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_spread_key_ignored() {
    // Spread elements have no static key name and must not be flagged.
    let source = r#"
export default {
  ...mixinOptions,
  methods: {}
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_distance_two_not_flagged() {
    // `methoo` is distance 2 from `methods` (and far from everything else).
    let source = r#"
export default {
  methoo: {}
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}
