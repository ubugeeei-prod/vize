//! Destructuring names own identifier byte ranges, independently of source kind.

#[test]
fn destructured_bindings_keep_authored_identifier_spans() {
    let source = "// 日本語\r\nconst { source: renamed = fallback, nested: { deep }, ...rest } = objectFactory();\r\nconst [first, , ...tail] = arrayFactory();";
    let result = super::super::parse_script_setup(source);
    for name in ["renamed", "deep", "rest", "first", "tail"] {
        let start = source.find(name).expect("unique authored declaration");
        let expected = (start as u32, (start + name.len()) as u32);
        assert_eq!(result.binding_spans.get(name), Some(&expected), "{name}");
        assert_eq!(
            source.get(expected.0 as usize..expected.1 as usize),
            Some(name),
        );
    }
    for name in [
        "source",
        "nested",
        "fallback",
        "objectFactory",
        "arrayFactory",
    ] {
        assert!(
            !result.binding_spans.contains_key(name),
            "{name} is not a declared local"
        );
    }
}
