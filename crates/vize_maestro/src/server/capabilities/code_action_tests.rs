use super::*;

#[test]
fn advertises_only_the_kind_the_handler_emits() {
    let capabilities = server_capabilities(LspFeatureConfig {
        lint: true,
        code_actions: true,
        ..LspFeatureConfig::default()
    });
    let Some(CodeActionProviderCapability::Options(options)) = capabilities.code_action_provider
    else {
        panic!("code action options should be advertised");
    };
    assert_eq!(
        options.code_action_kinds,
        Some(vec![CodeActionKind::QUICKFIX])
    );
}
