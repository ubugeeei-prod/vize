//! Restoring authored file paths in a diagnostic message body.
//!
//! [`module_specifier`](super::module_specifier) undoes the import rewriter's
//! `./Panel.vue` -> `./Panel.vue.ts` spelling change. This module undoes the
//! other half of the mirroring: the *root*. Every registered source is
//! materialized under `.vize/canon/projects/<hash>/`, and a checker
//! message that interpolates an absolute path quotes it from there.
//!
//! `vue-tsc` names the authored path, so the difference is a divergence the
//! parity comparator scores; worse, the path names a directory the author never
//! wrote and cannot act on. Measured over the pinned real-project corpus, six
//! projects leak it across seven diagnostic codes — `TS6307` (202 in
//! `element-plus`), `TS2345`/`TS2739` (66 in `vue-router`), `TS7053`, `TS6142`,
//! `TS7016`, `TS1149` — including inside multi-line explanation chains and
//! inside `import("...")` type identities, neither of which the quoted-specifier
//! scan reaches (#3227).
//!
//! Rewriting is a plain prefix substitution, and deliberately unconditional: the
//! virtual root is a Vize-generated directory keyed by a content hash, so a path
//! under it can only have come from the mirroring. That is the opposite of the
//! specifier rewrite next door, which must first prove the author did not write
//! the mirror spelling themselves — there, `./Panel.vue.ts` is a name a project
//! is allowed to have.

use std::path::Path;

use super::{Diagnostic, VirtualProject};
use vize_carton::{String, ToCompactString};

/// Restore authored paths across every message one checker output produced.
///
/// Doing it here rather than per producer is what makes the coverage total: a
/// CLI message is assembled from three sources — the diagnostic line itself, a
/// project-level line with no file position, and the raw continuation lines a
/// multi-line explanation appends below it (`TS6307`'s "The file is in the
/// program because:" chain). Only the first ever passes through
/// [`DiagnosticMapper`](super::DiagnosticMapper), so without this the other two
/// print `.vize/canon/projects/<hash>/…` paths the author never
/// wrote (#3227).
pub(in crate::batch::executor) fn restore_authored_paths_in_messages(
    mut diagnostics: Vec<Diagnostic>,
    project: &VirtualProject,
) -> Vec<Diagnostic> {
    for diagnostic in &mut diagnostics {
        diagnostic.message = restore_authored_paths(
            &diagnostic.message,
            project.virtual_root(),
            project.project_root(),
        );
    }
    diagnostics
}

/// Rewrite every virtual-root-prefixed path in `message` to its authored path.
///
/// Both separators are handled because a checker message may print either
/// spelling, and on Windows the two mix within one message.
pub(in crate::batch::executor) fn restore_authored_paths(
    message: &str,
    virtual_root: &Path,
    project_root: &Path,
) -> String {
    let Some(virtual_root) = virtual_root.to_str() else {
        return message.to_compact_string();
    };
    let Some(project_root) = project_root.to_str() else {
        return message.to_compact_string();
    };
    if virtual_root.is_empty() || !message.contains(root_stem(virtual_root)) {
        return message.to_compact_string();
    }

    let mut rewritten = replace_root_prefix(message, virtual_root, project_root);
    let virtual_alternate = flip_separators(virtual_root);
    if virtual_alternate.as_str() != virtual_root {
        rewritten = replace_root_prefix(
            rewritten.as_str(),
            virtual_alternate.as_str(),
            flip_separators(project_root).as_str(),
        );
    }
    rewritten
}

/// Replace `root` with `replacement` only where it spans whole path
/// components on both sides, so neither a sibling whose name merely starts
/// with the root's — the virtual root is `…/projects/8c5cb99f`, the sibling
/// `…/projects/8c5cb99f-backup` — nor an unrelated directory the root's
/// spelling is embedded in (`/backup<root>/a.ts`) is rewritten.
///
/// `preceding` carries the last character already emitted, because a match at
/// the very start of `rest` is only at the start of the *message* on the first
/// iteration; afterwards the character before it is the tail of the previous
/// match.
fn replace_root_prefix(message: &str, root: &str, replacement: &str) -> String {
    let mut rewritten = String::with_capacity(message.len());
    let mut rest = message;
    let mut preceding = None;
    while let Some(at) = rest.find(root) {
        let (before, matched) = rest.split_at(at);
        let after = &matched[root.len()..];
        rewritten.push_str(before);
        let emitted = if starts_a_path_component(before.chars().next_back().or(preceding))
            && ends_a_path_component(after)
        {
            replacement
        } else {
            root
        };
        rewritten.push_str(emitted);
        preceding = emitted.chars().next_back();
        rest = after;
    }
    rewritten.push_str(rest);
    rewritten
}

/// Whether the character before a match opens the matched path: the start of
/// the message, a separator, or the punctuation and whitespace a checker
/// message opens a path with (`project '<path>'.`).
fn starts_a_path_component(before: Option<char>) -> bool {
    before.is_none_or(delimits_a_path)
}

/// Whether the text following a match ends the matched directory name: a
/// separator, the end of the message, or the punctuation and whitespace a
/// checker message closes a path with (`project '<path>'.`).
fn ends_a_path_component(after: &str) -> bool {
    after.is_empty() || after.starts_with(delimits_a_path)
}

fn delimits_a_path(character: char) -> bool {
    matches!(character, '/' | '\\')
        || character.is_whitespace()
        || matches!(
            character,
            '\'' | '"' | '`' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | ',' | ';'
        )
}

/// The trailing path component of the virtual root, used as a cheap pre-filter
/// so the common case — a message with no virtual path at all — allocates
/// nothing.
fn root_stem(virtual_root: &str) -> &str {
    virtual_root
        .rsplit(['/', '\\'])
        .next()
        .filter(|stem| !stem.is_empty())
        .unwrap_or(virtual_root)
}

fn flip_separators(path: &str) -> String {
    if path.contains('\\') {
        String::from(path.replace('\\', "/").as_str())
    } else {
        String::from(path.replace('/', "\\").as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::restore_authored_paths;
    use std::path::Path;

    const VIRTUAL: &str = "/repo/.vize/canon/projects/8c5cb99f";
    const PROJECT: &str = "/repo";

    fn restore(message: &str) -> String {
        restore_authored_paths(message, Path::new(VIRTUAL), Path::new(PROJECT)).into()
    }

    /// The `element-plus` shape: 202 diagnostics named the generated shard
    /// tsconfig, so the reader was pointed at a file Vize wrote.
    #[test]
    fn restores_the_project_reference_explanation_chain() {
        let message = format!(
            "File '{PROJECT}/packages/utils/index.ts' is not listed within the file list of \
             project '{VIRTUAL}/tsconfig.shard0.json'.\nThe file is in the program because:\n\
             Imported via '@element-plus/utils' from file '{VIRTUAL}/packages/components/affix/index.ts'"
        );

        let restored = restore(&message);

        assert!(!restored.contains(".vize/canon"), "{restored}");
        assert!(
            restored.contains("project '/repo/tsconfig.shard0.json'"),
            "{restored}"
        );
        assert!(
            restored.contains("from file '/repo/packages/components/affix/index.ts'"),
            "{restored}",
        );
    }

    /// The `vue-router` shape: one side of an assignability message carried the
    /// authored identity and the other the mirrored one, so the diagnostic read
    /// as a type mismatch against itself.
    #[test]
    fn restores_both_module_identities_in_one_message() {
        let message = format!(
            "Type 'import(\"{PROJECT}/packages/router/src/typed-routes/navigation-guards\").NavigationGuard' \
             is not assignable to type 'import(\"{VIRTUAL}/router/src/typed-routes/navigation-guards\").NavigationGuard'."
        );

        let restored = restore(&message);

        assert!(!restored.contains(".vize/canon"), "{restored}");
        assert_eq!(restored.matches("import(\"/repo/").count(), 2, "{restored}");
    }

    #[test]
    fn leaves_a_message_without_a_virtual_path_untouched() {
        let message = "Type 'string' is not assignable to type 'number'.";
        assert_eq!(restore(message).as_str(), message);
    }

    /// An authored path that merely *contains* the hash stem is not under the
    /// virtual root, so the pre-filter must not turn into a rewrite.
    #[test]
    fn leaves_an_authored_path_sharing_the_root_stem_untouched() {
        let message = format!("File '{PROJECT}/src/8c5cb99f/index.ts' is not a module.");
        assert_eq!(restore(&message).as_str(), message.as_str());
    }

    /// A sibling directory whose name starts with the virtual root's full name
    /// is a different directory, so the prefix must match a whole component.
    #[test]
    fn leaves_a_sibling_directory_extending_the_root_name_untouched() {
        let message = format!("File '{VIRTUAL}-backup/a.ts' is not a module.");
        assert_eq!(restore(&message).as_str(), message.as_str());
    }

    /// The same message can carry both, and only the real virtual path moves.
    #[test]
    fn restores_the_virtual_path_beside_a_sibling_that_extends_the_root_name() {
        let message = format!("File '{VIRTUAL}/a.ts' shadows '{VIRTUAL}-backup/a.ts'.");

        let restored = restore(&message);

        assert_eq!(
            restored.as_str(),
            format!("File '{PROJECT}/a.ts' shadows '{VIRTUAL}-backup/a.ts'."),
        );
    }

    /// A directory the root's spelling is merely embedded in is a different
    /// directory too, so the match needs a component boundary on its left as
    /// well — otherwise `/backup<root>/a.ts` collapses to `/backup/repo/a.ts`.
    #[test]
    fn leaves_a_directory_whose_name_ends_with_the_root_untouched() {
        let message = format!("File '/backup{VIRTUAL}/a.ts' is not a module.");
        assert_eq!(restore(&message).as_str(), message.as_str());
    }

    /// Rejecting one match must not desensitize the ones after it: the second
    /// path here opens on a quote, so it still restores.
    #[test]
    fn restores_a_virtual_path_following_an_embedded_root_spelling() {
        let message = format!("File '/backup{VIRTUAL}/a.ts' shadows '{VIRTUAL}/a.ts'.");

        let restored = restore(&message);

        assert_eq!(
            restored.as_str(),
            format!("File '/backup{VIRTUAL}/a.ts' shadows '{PROJECT}/a.ts'."),
        );
    }

    #[test]
    fn restores_windows_separator_spellings() {
        let restored = restore_authored_paths(
            "File 'C:\\repo\\node_modules\\.vize\\canon\\projects\\8c5cb99f\\a.ts' is stale.",
            Path::new("C:\\repo\\node_modules\\.vize\\canon\\projects\\8c5cb99f"),
            Path::new("C:\\repo"),
        );

        assert_eq!(restored.as_str(), "File 'C:\\repo\\a.ts' is stale.");
    }
}
