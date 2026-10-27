//! Existing incomplete-source recovery contracts.

use super::*;
use crate::script_parser::parse_script_setup;

#[test]
fn incomplete_members_keep_imports_macros_and_authored_binding_offsets() {
    for expression in [
        "slots.",
        "slots?.",
        "slots. /* docs */",
        "slots?. // docs\r\n",
        "(() => slots.)()",
        "[slots., slots?.]",
        "slots.({ total: 1 })",
    ] {
        let source = vize_carton::cstr!(
            "import {{ useSlots }} from 'vue';\nconst 雪 = 1;\ndefineSlots<{{ summary(props: {{ total: number }}): any }}>();\nconst slots = useSlots();\n{expression}"
        );
        let allocator = Allocator::default();
        let parsed = parse_program_for_analysis(&allocator, &source, SourceType::ts());
        assert!(!parsed.panicked, "{expression}: {:?}", parsed.diagnostics);
        assert!(
            !parsed.diagnostics.is_empty(),
            "authored error must survive"
        );
        let result = parse_script_setup(&source);
        assert_eq!(result.import_statements.len(), 1, "{expression}");
        assert!(result.macros.define_slots().is_some(), "{expression}");
        let mut bindings: Vec<_> = result
            .bindings
            .bindings
            .keys()
            .map(|name| name.as_str())
            .collect();
        bindings.sort_unstable();
        assert_eq!(bindings, ["slots", "useSlots", "雪"], "{expression}");
        let import = &result.import_statements[0];
        assert_eq!(
            source
                .get(import.start as usize..import.end as usize)
                .unwrap_or_default(),
            "import { useSlots } from 'vue';"
        );
    }
}

#[test]
fn incomplete_members_before_the_next_statement_keep_the_whole_script() {
    for next in ["const next = 1;", "function next() {}"] {
        let source = vize_carton::cstr!(
            "import {{ ref }} from 'vue';\nconst items = ref([]);\nitems.value.\n{next}"
        );
        let allocator = Allocator::default();
        let parsed = parse_program_for_analysis(&allocator, &source, SourceType::ts());
        assert!(!parsed.panicked, "{next}: {:?}", parsed.diagnostics);
        assert_eq!(parsed.program.body.len(), 4, "{next}");
        assert_eq!(parsed.program.source_text, source.as_str(), "{next}");
        assert!(
            !parsed.diagnostics.is_empty(),
            "authored error must survive"
        );
        let result = parse_script_setup(&source);
        assert_eq!(result.import_statements.len(), 1, "{next}");
        assert!(result.bindings.bindings.contains_key("items"), "{next}");
        assert!(result.bindings.bindings.contains_key("next"), "{next}");
    }
}

#[test]
fn incomplete_calls_before_the_next_declaration_keep_the_whole_script() {
    for next in ["const selected = null;", "function pick() {}"] {
        let source = vize_carton::cstr!(
            "import {{ ref }} from 'vue';\nconst items = ref([]);\npick(\n{next}"
        );
        let allocator = Allocator::default();
        let parsed = parse_program_for_analysis(&allocator, &source, SourceType::ts());
        assert!(!parsed.panicked, "{next}: {:?}", parsed.diagnostics);
        assert_eq!(parsed.program.body.len(), 4, "{next}");
        assert_eq!(parsed.program.source_text, source.as_str(), "{next}");
        assert!(
            !parsed.diagnostics.is_empty(),
            "authored error must survive"
        );
        let result = parse_script_setup(&source);
        assert_eq!(result.import_statements.len(), 1, "{next}");
        assert!(result.bindings.bindings.contains_key("items"), "{next}");
        let binding = if next.starts_with("const") {
            "selected"
        } else {
            "pick"
        };
        assert!(result.bindings.bindings.contains_key(binding), "{next}");
    }
}

#[test]
fn fatal_incomplete_call_keeps_rich_setup_bindings() {
    let source = r#"import { ref } from "vue";
import MyButton from "./MyButton.vue";
import type { Item } from "./types";

const items = ref<Item[]>([]);
const selected = ref<Item | null>(null);
pick(
function pick(item: Item): void {
  selected.value = item;
}
"#;
    let allocator = Allocator::default();
    let raw = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(raw.panicked);

    let parsed = parse_program_for_analysis(&allocator, source, SourceType::ts());
    assert!(!parsed.panicked, "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.body.len(), 7);
    assert_eq!(parsed.program.source_text, source);
    let result = parse_script_setup(source);
    assert_eq!(result.import_statements.len(), 3);
    for binding in ["ref", "MyButton", "items", "selected", "pick"] {
        assert!(result.bindings.bindings.contains_key(binding), "{binding}");
    }
}

#[test]
fn valid_literals_and_comments_are_not_rewritten() {
    for source in [
        "const a = 1.;",
        "const a = 'slots.';",
        "// slots.\nconst a = 1;",
        "slots?.summary();",
    ] {
        let allocator = Allocator::default();
        let parsed = parse_program_for_analysis(&allocator, source, SourceType::ts());
        assert!(
            !parsed.panicked && parsed.diagnostics.is_empty(),
            "{source}"
        );
        assert_eq!(parsed.program.source_text, source);
    }
}

#[test]
fn unrelated_parse_errors_do_not_invent_semantic_facts() {
    let allocator = Allocator::default();
    let parsed = parse_program_for_analysis(&allocator, "const a = ;", SourceType::ts());
    assert!(parsed.panicked);
}
