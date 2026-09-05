use super::WorkspaceSymbolsService;
use tower_lsp::lsp_types::{Position, Range, SymbolKind};

#[test]
fn test_to_pascal_case() {
    assert_eq!(
        WorkspaceSymbolsService::to_pascal_case("hello-world"),
        "HelloWorld"
    );
    assert_eq!(
        WorkspaceSymbolsService::to_pascal_case("my_component"),
        "MyComponent"
    );
    assert_eq!(WorkspaceSymbolsService::to_pascal_case("Button"), "Button");
}

#[test]
fn test_extract_identifier() {
    assert_eq!(
        WorkspaceSymbolsService::extract_identifier("count = 0"),
        Some("count".to_string())
    );
    assert_eq!(
        WorkspaceSymbolsService::extract_identifier("MyClass extends Base"),
        Some("MyClass".to_string())
    );
    assert_eq!(
        WorkspaceSymbolsService::extract_identifier("{ a, b } = obj"),
        None
    );
}

#[test]
fn test_extract_css_classes() {
    let classes = WorkspaceSymbolsService::extract_css_classes(".container .item-active { }");
    assert_eq!(classes, vec!["container", "item-active"]);
}

#[test]
fn test_extract_css_ids() {
    let ids = WorkspaceSymbolsService::extract_css_ids("#app #main-content { }");
    assert_eq!(ids, vec!["app", "main-content"]);
}

#[test]
fn test_parse_declaration() {
    let (name, kind) = WorkspaceSymbolsService::parse_declaration("count = ref(0)").unwrap();
    assert_eq!(name, "count");
    assert_eq!(kind, SymbolKind::VARIABLE);

    let (name, kind) =
        WorkspaceSymbolsService::parse_declaration("handleClick = () => {}").unwrap();
    assert_eq!(name, "handleClick");
    assert_eq!(kind, SymbolKind::FUNCTION);
}

#[test]
fn macro_declaration_range_targets_quoted_name_content() {
    let source = "<script setup>\nconst emit = defineEmits(['save-item'])\n</script>\n";
    let script_start = source.find("const emit").unwrap();
    let script_end = source.find("\n</script>").unwrap();
    let script = &source[script_start..script_end];
    let quoted_start = script.find("'save-item'").unwrap();
    let quoted_end = quoted_start + "'save-item'".len();
    let name_start = source.find("save-item").unwrap();
    let name_end = name_start + "save-item".len();
    let fallback = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    };

    let range = WorkspaceSymbolsService::macro_declaration_range(
        source,
        script,
        script_start,
        Some((quoted_start as u32, quoted_end as u32)),
        fallback,
    );

    assert_eq!(
        range.start,
        WorkspaceSymbolsService::offset_to_position(source, name_start)
    );
    assert_eq!(
        range.end,
        WorkspaceSymbolsService::offset_to_position(source, name_end)
    );
}
