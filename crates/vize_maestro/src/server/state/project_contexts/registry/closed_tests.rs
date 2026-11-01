//! A late initializer and normal full retirement retain distinct ownership laws.
use super::*;

#[test]
fn closed_detach_post_initialization_never_explicitly_clears_a_held_bridge() {
    let state = Arc::new(ServerState::new());
    let bridge = Arc::new(vize_canon::CorsaBridge::new());
    *state.corsa_bridge.write() = Some(Arc::clone(&bridge));
    let context = ProjectContext::new(std::path::PathBuf::from("/closed/child"));
    assert!(context.state.set(Arc::clone(&state)).is_ok());
    context.detach_closed();
    context.retire_initialized_state(&state);
    assert!(state.project_context_retired());
    assert!(state.owns_corsa_bridge(&bridge));
    // Existing explicit configuration/shutdown retirement keeps its old authority.
    context.retire();
    assert!(!state.owns_corsa_bridge(&bridge));
}

#[test]
fn ordinary_retirement_after_initialization_still_clears_the_old_bridge() {
    let state = Arc::new(ServerState::new());
    let bridge = Arc::new(vize_canon::CorsaBridge::new());
    *state.corsa_bridge.write() = Some(Arc::clone(&bridge));
    let context = ProjectContext::new(std::path::PathBuf::from("/retired/child"));
    context.mark_retired();
    context.retire_initialized_state(&state);
    assert!(!state.owns_corsa_bridge(&bridge));
}
