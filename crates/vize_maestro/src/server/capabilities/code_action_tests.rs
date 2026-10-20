use super::*;

#[test]
fn advertises_only_the_kinds_the_handler_emits() {
    let kinds = |typecheck| {
        let capabilities = server_capabilities(LspFeatureConfig {
            lint: true,
            code_actions: true,
            typecheck,
            ..LspFeatureConfig::default()
        });
        let Some(CodeActionProviderCapability::Options(options)) =
            capabilities.code_action_provider
        else {
            panic!("code action options should be advertised");
        };
        options.code_action_kinds
    };
    assert_eq!(kinds(false), Some(vec![CodeActionKind::QUICKFIX]));
    let expected = if cfg!(feature = "native") {
        vec![
            CodeActionKind::QUICKFIX,
            CodeActionKind::from("source.organizeImports".to_owned()),
            CodeActionKind::from("source.sortImports".to_owned()),
            CodeActionKind::from("source.removeUnusedImports".to_owned()),
            CodeActionKind::from("source.fixAll".to_owned()),
        ]
    } else {
        vec![CodeActionKind::QUICKFIX]
    };
    assert_eq!(kinds(true), Some(expected));
}
