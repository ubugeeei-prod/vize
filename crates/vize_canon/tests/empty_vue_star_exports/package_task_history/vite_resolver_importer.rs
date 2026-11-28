//! One direct importer edge; every byte of the qualified lock predecessor stays bound.
use crate::support;

const CURRENT_BYTES: usize = 1_123_285;
const CURRENT_SHA: &str = "0fe46333b6cb4321229547f803b1920d1223cfdb19d1a3f55ba63d4e59154380";
const PREVIOUS_BYTES: usize = 1_123_206;
const PREVIOUS_SHA: &str = "f70a28ee91cdde0551e473c30eceedcda7e40c0d58e155c283f60005d063a8d4";
const BEFORE: &str = "  npm/builder/vite:\n    dependencies:\n      '@vizejs/native':\n";
const ADDED: &str =
    "      '@rolldown/pluginutils':\n        specifier: 1.0.1\n        version: 1.0.1\n";
const AFTER: &str = "  npm/builder/vite:\n    dependencies:\n      '@rolldown/pluginutils':\n        specifier: 1.0.1\n        version: 1.0.1\n      '@vizejs/native':\n";

pub(crate) fn before_vite_resolver_importer(path: &str, body: &[u8]) -> Option<Vec<u8>> {
    if path != "pnpm-lock.yaml" || body.len() != CURRENT_BYTES || support::hash(body) != CURRENT_SHA
    {
        return None;
    }
    let current = std::str::from_utf8(body).ok()?;
    if current.matches(AFTER).count() != 1 {
        return None;
    }
    let previous = current.replacen(AFTER, BEFORE, 1).into_bytes();
    (previous.len() == PREVIOUS_BYTES && support::hash(&previous) == PREVIOUS_SHA)
        .then_some(previous)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_importer_inverse_preserves_the_whole_lock_and_history_composition() {
        let path = "pnpm-lock.yaml";
        let current = super::super::current_lock_before_parser_free_builders();
        let previous = before_vite_resolver_importer(path, current.as_bytes()).unwrap();
        assert_eq!(ADDED.len(), 79);
        assert_eq!(current.len() - previous.len(), ADDED.len());
        assert_eq!(support::hash(&previous), PREVIOUS_SHA);
        let previous = String::from_utf8(previous).unwrap();
        assert_eq!(previous.matches(BEFORE).count(), 1);
        assert_eq!(previous.replacen(BEFORE, AFTER, 1), current);
        assert!(before_vite_resolver_importer(path, previous.as_bytes()).is_none());
        assert_eq!(
            super::super::restore(path, current.clone()),
            super::super::restore(path, previous.clone()),
        );
        assert_eq!(
            super::super::before_translation_worker(path, current.as_bytes()),
            super::super::before_translation_worker(path, previous.as_bytes()),
        );
        assert!(super::super::before_translation_worker(path, current.as_bytes()).is_some());
    }

    #[test]
    fn missing_duplicate_foreign_and_unrelated_lock_bytes_cannot_use_the_importer_edge() {
        let path = "pnpm-lock.yaml";
        let current = super::super::current_lock_before_parser_free_builders();
        assert!(before_vite_resolver_importer("foreign-lock.yaml", current.as_bytes()).is_none());
        for changed in [
            current.replacen(AFTER, BEFORE, 1),
            current.replacen(AFTER, &format!("{AFTER}{ADDED}"), 1),
            current.replacen(AFTER, &AFTER.replace("1.0.1", "1.0.2"), 1),
            current.replacen("  npm/builder/vite:\n", "  npm/builder/other:\n", 1),
            current.replacen("pythonia@1.2.6", "pythonia@0.0.0", 1),
            format!("{current}\n"),
        ] {
            assert_ne!(changed, current);
            assert!(before_vite_resolver_importer(path, changed.as_bytes()).is_none());
        }
    }
}
