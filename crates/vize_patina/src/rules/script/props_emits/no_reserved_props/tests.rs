use crate::rules::script::NoReservedProps;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoReservedProps));
    linter
}

#[test]
fn test_valid_normal_props() {
    let source = r#"
export default {
  props: {
    name: String,
    refValue: Number
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_ref() {
    let source = r#"
export default {
  props: {
    ref: String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_key() {
    let source = r#"
export default {
  props: {
    key: String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_is() {
    let source = r#"
export default {
  props: {
    is: String
  }
}
const runtime = defineProps({ is: String, label: String })
const listed = defineProps(['is', 'label'])
defineProps<{ is: string; label: string }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_invalid_ref_for_and_ref_key() {
    let source = r#"
export default {
  props: {
    ref_for: Boolean,
    ref_key: String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_invalid_dollar_prefixed() {
    let source = r#"
export default {
  props: {
    $foo: Number
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_array_form_reserved() {
    let source = r#"
export default {
  props: ['key', 'name']
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_props_object_reserved() {
    let source = r#"
const props = defineProps({
  ref: String
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_props_array_reserved() {
    let source = r#"
const props = defineProps(['ref', 'label'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_component_reserved() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    key: String
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_no_props_option() {
    let source = r#"
export default {
  data() {
    return { count: 0 }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_string_literal_key_reserved() {
    let source = r#"
export default {
  props: {
    "ref": String
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_type_literal_reports_key_and_ref() {
    let source = "defineProps<{ ref: string; key: string; label: string }>()\n";
    let result = create_linter().lint(source, 0);
    assert_eq!(reserved_names(&result), ["key", "ref"]);
    assert_eq!(
        result.diagnostics[0].start,
        source.find("ref").unwrap() as u32
    );
    assert_eq!(
        result.diagnostics[1].start,
        source.find("key").unwrap() as u32
    );
}

#[test]
fn test_type_forms_report_reserved_members() {
    let interface = "interface Props { ref: string; label: string }\ndefineProps<Props>()\n";
    assert_eq!(reserved_names(&create_linter().lint(interface, 0)), ["ref"]);

    let alias = "type Props = { key: string; label: string }\ndefineProps<Props>()\n";
    assert_eq!(reserved_names(&create_linter().lint(alias, 0)), ["key"]);

    let defaults =
        "withDefaults(defineProps<{ ref?: string; label?: string }>(), { ref: 'a', label: 'b' })\n";
    assert_eq!(reserved_names(&create_linter().lint(defaults, 0)), ["ref"]);

    let destructure =
        "const { key = 'a', label = 'b' } = defineProps<{ key?: string; label?: string }>()\n";
    assert_eq!(
        reserved_names(&create_linter().lint(destructure, 0)),
        ["key"]
    );

    let quoted = "defineProps<{ \"ref\": string; label: string }>()\n";
    assert_eq!(reserved_names(&create_linter().lint(quoted, 0)), ["ref"]);

    let method = "defineProps<{ ref(): void; label: string }>()\n";
    assert_eq!(reserved_names(&create_linter().lint(method, 0)), ["ref"]);

    let intersection =
        "interface Base { ref: string }\ndefineProps<Base & { key: string; label: string }>()\n";
    assert_eq!(
        reserved_names(&create_linter().lint(intersection, 0)),
        ["key", "ref"]
    );

    let generic = "defineProps<{ ref: T; label: string }>()\n";
    assert_eq!(reserved_names(&create_linter().lint(generic, 0)), ["ref"]);
}

#[test]
fn test_template_literal_array_element() {
    let source = "defineProps([`ref`, 'label'])\n";
    let result = create_linter().lint(source, 0);
    assert_eq!(reserved_names(&result), ["ref"]);
    let diagnostic = &result.diagnostics[0];
    let text = &source[diagnostic.start as usize..diagnostic.end as usize];
    assert!(text.contains("ref"), "{text}");
}

#[test]
fn test_imported_type_stays_unresolved() {
    let source = "import type { Props } from './types'\ndefineProps<Props>()\n";
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

fn reserved_names(result: &crate::rules::script::ScriptLintResult) -> Vec<&str> {
    let mut names: Vec<&str> = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| {
            diagnostic
                .message
                .strip_prefix("Prop name '")
                .and_then(|rest| rest.strip_suffix("' is reserved."))
        })
        .collect();
    names.sort_unstable();
    names
}
