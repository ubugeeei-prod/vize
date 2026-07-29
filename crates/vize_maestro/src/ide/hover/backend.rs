//! Provenance gate for hover *type* text.
//!
//! Vize carries a script-binding inference used to answer hovers before the
//! type backend existed. It is a heuristic: it shows the script-side
//! `Ref<string>` at template positions (where Vue unwraps the ref) and types a
//! plain `string` const as `MaybeRef<unknown>`. Rendered as a `typescript`
//! code block it is indistinguishable from a real answer, so a session whose
//! backend never came up still looks like it is type checking (#3321).
//!
//! When the live backend owns the answer -- a `native` build with typecheck
//! enabled -- the heuristic must not supply type text at all. Either the
//! backend answers or hover declines the type; it never guesses.
//!
//! Only *type* text is gated. Documentation hovers are untouched: directives,
//! the Vue API and macro reference, and the binding-provenance hovers that
//! carry no signature all keep answering exactly as before.

use tower_lsp::lsp_types::Hover;

use super::HoverService;
use crate::ide::IdeContext;

/// True when the live type backend owns hover type text for this document.
///
/// `native` is the build that can reach a backend at all; `typecheck` is the
/// per-session switch a client uses to ask for real types. With both set, a
/// heuristic type would be a claim vize cannot stand behind.
fn backend_owns_type_hovers(ctx: &IdeContext<'_>) -> bool {
    cfg!(feature = "native") && ctx.state.lsp_features().typecheck
}

/// Template binding type hover from script analysis, gated on provenance.
pub(super) fn binding_type_hover(ctx: &IdeContext<'_>, word: &str) -> Option<Hover> {
    if backend_owns_type_hovers(ctx) {
        return None;
    }

    HoverService::hover_ts_binding(ctx, word)
}

/// Script binding type hover from script analysis, gated on provenance.
pub(super) fn script_binding_type_hover(ctx: &IdeContext<'_>, word: &str) -> Option<Hover> {
    if backend_owns_type_hovers(ctx) {
        return None;
    }

    HoverService::hover_ts_binding_in_script(ctx, word)
}

/// Template-expression type from vize's own inference, gated on provenance.
pub(super) fn heuristic_type_at(ctx: &IdeContext<'_>) -> Option<vize_canon::TypeInfo> {
    if backend_owns_type_hovers(ctx) {
        return None;
    }

    crate::ide::TypeService::get_type_at(ctx)
}

#[cfg(test)]
mod tests {
    use super::backend_owns_type_hovers;
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::Url;

    // The gate keys off the session's typecheck switch, not off whether a
    // backend happened to answer: a session that asked for real types must
    // never be handed a heuristic type, including when the backend is missing.
    #[test]
    fn typecheck_sessions_hand_type_hovers_to_the_backend() {
        let source = "<template><p>{{ message }}</p></template>";
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/App.vue").unwrap();
        let ctx = IdeContext::with_content(&state, &uri, 0, source.into());

        assert!(state.lsp_features().typecheck);
        assert_eq!(backend_owns_type_hovers(&ctx), cfg!(feature = "native"));
    }
}
