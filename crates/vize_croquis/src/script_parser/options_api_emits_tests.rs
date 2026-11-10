use super::{ScriptParserOptions, parse_script_with_options};

#[test]
fn parses_options_api_emits_runtime_object_payloads() {
    let source = r#"
type ChoiceOption = { value: string }

const sharedEmits = {
  close: null,
}

export default {
  emits: {
    ...sharedEmits,
    input(value: ChoiceOption) {
      return value.value.length > 0
    },
    cancel() {
      return true
    },
  },
}
"#;
    let result = parse_script_with_options(
        source,
        ScriptParserOptions {
            options_api: true,
            legacy_vue2: true,
        },
    );

    let emits = result.macros.emits();
    let input = emits
        .iter()
        .find(|emit| emit.name == "input")
        .expect("input emit should be extracted");
    assert_eq!(input.payload_type.as_deref(), Some("[value: ChoiceOption]"));
    let cancel = emits
        .iter()
        .find(|emit| emit.name == "cancel")
        .expect("cancel emit should be extracted");
    assert_eq!(cancel.payload_type.as_deref(), Some("[]"));
    assert!(
        emits.iter().any(|emit| emit.name == "close"),
        "spread emits should be extracted, got {emits:?}"
    );
    for name in ["close", "input", "cancel"] {
        let (start, end) = result
            .macros
            .emit_declaration(name)
            .expect("Options API event declaration");
        let authored = source
            .get(start as usize..end as usize)
            .expect("authored event key");
        assert_eq!(authored.trim_matches(['\'', '"']), name);
    }
}
