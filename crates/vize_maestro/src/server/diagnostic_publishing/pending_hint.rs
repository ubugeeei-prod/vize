//! Partial native feedback keeps the lint diagnostics and one hint.

use tower_lsp::lsp_types::Diagnostic;

/// Append the pending typecheck hint without a second buffer copy.
///
/// `to_vec` sizes the allocation to `sync.len()`. Pushing the hint then
/// allocates again and copies every diagnostic.
pub(super) fn with_typecheck_pending_hint(sync: &[Diagnostic]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::with_capacity(sync.len().saturating_add(1));
    diagnostics.extend_from_slice(sync);
    diagnostics.push(crate::ide::diagnostics::typecheck_pending_hint());
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::with_typecheck_pending_hint;
    use crate::ide::diagnostics::typecheck_pending_hint;

    #[test]
    fn the_pending_hint_matches_an_appended_copy_in_one_buffer() {
        let sync = vec![typecheck_pending_hint(), typecheck_pending_hint()];
        let mut expected = sync.clone();
        expected.push(typecheck_pending_hint());
        let actual = with_typecheck_pending_hint(&sync);
        assert_eq!(actual, expected);
        assert_eq!(actual.capacity(), sync.len() + 1);

        let empty = with_typecheck_pending_hint(&[]);
        assert_eq!(empty, vec![typecheck_pending_hint()]);
        assert_eq!(empty.capacity(), 1);
    }
}
