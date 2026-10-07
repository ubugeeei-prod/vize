//! Complete original root and transitions survive storage reservation.

use super::super::{BindingType, CompactString, ScopeBinding, ScopeChain, ScopeKind};

#[test]
fn reserved_existing_root_keeps_original_whole_scope_graph() {
    for capacity in [0, 1, 16, 64] {
        let mut original = ScopeChain::with_capacity(capacity);
        let mut reserved = ScopeChain::default();
        reserved.reserve_scopes(capacity);
        assert_eq!(format!("{reserved:?}"), format!("{original:?}"));

        for chain in [&mut original, &mut reserved] {
            chain.enter_scope(ScopeKind::Block);
            chain.add_binding(
                CompactString::new("Math"),
                ScopeBinding::new(BindingType::SetupRef, 17),
            );
            chain.enter_scope(ScopeKind::Function);
            chain.add_binding(
                CompactString::new("local"),
                ScopeBinding::new(BindingType::SetupLet, 29),
            );
            chain.mark_used("Math");
            chain.mark_used("Array");
            chain.mark_mutated("local");
        }
        assert_eq!(format!("{reserved:?}"), format!("{original:?}"));
        assert_eq!(reserved.current_id(), original.current_id());
        reserved.reserve_scopes(128);
        assert_eq!(format!("{reserved:?}"), format!("{original:?}"));
    }
}
