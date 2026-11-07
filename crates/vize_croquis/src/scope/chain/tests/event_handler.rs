use super::{CompactString, EventHandlerScopeData, ScopeChain};

#[test]
fn test_event_handler_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_event_handler_scope(
        EventHandlerScopeData {
            event_name: CompactString::new("click"),
            event_name_is_dynamic: false,
            has_implicit_event: true,
            param_names: vize_carton::smallvec![],
            handler_expression: None,
            target_component: None,
        },
        0,
        50,
    );

    // $event should be available
    assert!(chain.is_defined("$event"));
}
