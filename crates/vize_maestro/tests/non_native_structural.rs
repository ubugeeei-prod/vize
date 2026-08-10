use tower_lsp::lsp_types::{Position, Range, SemanticTokensResult, Url};
use vize_maestro::ide::{
    JsxCodeActionService, JsxDocumentSymbolsService, JsxScopedStyleService,
    JsxSemanticTokensService,
};

fn whole_document() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: u32::MAX,
            character: 0,
        },
    }
}

#[test]
fn jsx_structural_services_work_without_the_native_checker() {
    let uri = Url::parse("file:///tmp/non-native/Counter.tsx").unwrap();
    let source = r#"const Counter = (props: { count: number }) => (
  <section    class={props.count > 0 ? "active" : "idle"}>
    <span>{props.count}</span>
    <style scoped>{`.active { color: green; }`}</style>
  </section>
);
"#;

    let symbols = JsxDocumentSymbolsService::symbols(source, &uri).expect("component symbol");
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "Counter");

    let tokens = JsxSemanticTokensService::tokens(source, &uri).expect("semantic tokens");
    let SemanticTokensResult::Tokens(tokens) = tokens else {
        panic!("expected full semantic tokens");
    };
    assert!(!tokens.data.is_empty());

    let actions = JsxCodeActionService::code_actions(source, &uri, whole_document());
    assert!(!actions.is_empty(), "multi-space JSX should offer a fix");

    let styles = JsxScopedStyleService::virtual_css_documents(source, &uri);
    assert_eq!(styles.len(), 1);
    assert_eq!(styles[0].content, ".active { color: green; }");
}
