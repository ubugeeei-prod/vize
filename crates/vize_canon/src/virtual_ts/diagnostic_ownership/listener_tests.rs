use super::{DiagnosticOwnership, InputOwner};

#[test]
fn listener_render_read_requires_its_actual_typed_primary() {
    let owner = InputOwner::for_listener_reference(7, 10, 17);
    let mut missing = DiagnosticOwnership::default();
    missing.record_bare_navigation(owner, 100..107, 40..47);
    assert!(missing.into_echoes().is_empty());
    let mut paired = DiagnosticOwnership::default();
    paired.record_bare_navigation(owner, 100..107, 40..47);
    paired.record(owner, 200..207, 40..47, false);
    let echoes = paired.into_echoes();
    assert_eq!(echoes.len(), 1);
    assert_eq!(echoes[0].generated, 100..107);
    assert_eq!(echoes[0].authored, 40..47);
    assert!(echoes[0].suppress);
}

#[test]
fn another_listener_scope_or_span_cannot_hide_a_read_error() {
    let read = InputOwner::for_listener_reference(7, 10, 17);
    for primary in [
        InputOwner::for_listener_reference(8, 10, 17),
        InputOwner::for_listener_reference(7, 20, 27),
    ] {
        let mut ownership = DiagnosticOwnership::default();
        ownership.record_bare_navigation(read, 100..107, 40..47);
        ownership.record(primary, 200..207, 40..47, false);
        assert!(ownership.into_echoes().is_empty());
    }
}
