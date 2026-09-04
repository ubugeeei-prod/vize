use super::{
    NoOptionsApi, ScriptLintResult, ScriptRule, has_component_option_key,
    imports_vue_runtime_module, is_component_option_name,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn parse_program<'a>(allocator: &'a Allocator, source: &'a str) -> oxc_ast::ast::Program<'a> {
    Parser::new(allocator, source, SourceType::ts())
        .parse()
        .program
}

#[test]
fn test_component_option_names_cover_vue_options() {
    for name in ["data", "props", "methods", "mounted", "render", "filters"] {
        assert!(is_component_option_name(name));
    }
    for name in ["signInConfirm", "breakpoint", "alert"] {
        assert!(!is_component_option_name(name));
    }
}

#[test]
fn test_vue_runtime_import_detection() {
    let allocator = Allocator::default();
    let program = parse_program(&allocator, "import { h } from 'vue'\n");
    assert!(program.body.iter().any(imports_vue_runtime_module));

    let allocator = Allocator::default();
    let program = parse_program(&allocator, "import { createApp } from 'petite-vue'\n");
    assert!(!program.body.iter().any(imports_vue_runtime_module));
}

#[test]
fn test_valid_composition_api() {
    let source = r#"
import { ref, computed } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
"#;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_plain_default_exported_i18n_catalog_is_not_options_api() {
    let source = r#"
export default {
  signInConfirm: {
    teacherName: "Sign in as {name}."
  },
  alert: {
    default: "Something went wrong."
  }
}
"#;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 0, "got: {:?}", result.diagnostics);
}

#[test]
fn test_plain_default_exported_design_tokens_are_not_options_api() {
    let source = r#"
export default {
  breakpoint: {
    s: { value: 600 },
    m: { value: 1200 }
  }
}
"#;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 0, "got: {:?}", result.diagnostics);
}

#[test]
fn test_default_export_with_vue_import_is_component_options() {
    let source = r#"
import { h } from 'vue'

export default {
  customOption: true
}
"#;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "script/no-options-api");
}

#[test]
fn test_invalid_data_option() {
    let source = r#"
export default {
  data() {
    return { count: 0 }
  }
}
"#;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_plain_default_export_with_component_option_key_is_options_api() {
    let source = r#"
export default {
  props: {
    value: String
  }
}
"#;
    let allocator = Allocator::default();
    let program = parse_program(&allocator, source);
    let export = program
        .body
        .iter()
        .find_map(|statement| match statement {
            oxc_ast::ast::Statement::ExportDefaultDeclaration(export) => {
                match &export.declaration {
                    oxc_ast::ast::ExportDefaultDeclarationKind::ObjectExpression(object) => {
                        Some(object)
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .expect("default object export");
    assert!(has_component_option_key(export));

    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "script/no-options-api");
}

#[test]
fn test_invalid_define_component_props_option() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    count: Number
  }
})
"#;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_identifier_export() {
    let source = r#"
const component = {
  methods: {
    increment() { this.count++ }
  }
}

export default component
"#;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_component_metadata_only_still_errors() {
    let source = r#"
export default {
  name: 'CounterButton',
  inheritAttrs: false
}
"#;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_cdn_create_app_options() {
    let source = r##"
Vue.createApp({
  data() {
    return { count: 0 }
  }
}).mount("#app")
"##;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_destructured_create_app_options() {
    let source = r##"
const { createApp } = Vue
const options = {
  methods: {
    increment() {}
  }
}

createApp(options).mount("#app")
"##;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_petite_vue_global_create_app_is_not_options_api() {
    let source = r##"
PetiteVue.createApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_petite_vue_imported_create_app_is_not_options_api() {
    let source = r##"
import { createApp } from 'petite-vue'

createApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_petite_vue_cdn_imported_create_app_is_not_options_api() {
    let source = r##"
import { createApp as createPetiteApp } from 'https://unpkg.com/petite-vue?module'

createPetiteApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_petite_vue_destructured_create_app_is_not_options_api() {
    let source = r##"
const { createApp } = PetiteVue

createApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_petite_vue_namespace_named_vue_is_not_options_api() {
    let source = r##"
import * as Vue from 'petite-vue'

Vue.createApp({
  count: 0,
  increment() {
    this.count++
  }
}).mount()
"##;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_no_export_default_skip() {
    let source = r#"
const computed = { foo: 'bar' }
"#;
    let rule = NoOptionsApi;
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_eq!(result.error_count, 0);
}
