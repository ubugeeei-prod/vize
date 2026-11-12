//! Rooted module specifiers inside importer-scoped package shadows.
//!
//! A package shadow keeps its TS and declaration modules byte-for-byte, so the
//! native checker resolves them exactly as it would in the installed package.
//! A rooted specifier is the exception: it names one file on disk, and the
//! copy would keep pointing at the original tree, outside the mirror. A stub
//! build does this (`nuxt-module-build build --stub` writes
//! `export * from "/abs/src/module.js"` into `dist/module.d.mts`), and from
//! the original tree every `.vue` re-export of the barrel is unreadable.
//!
//! The specifier is rebased onto the mirror copy of the file it names, which
//! is the identity the rest of the program already checks; the copy keeps the
//! file's own resolution context. A target that is not part of the program
//! stays authored, so the rewrite never invents a dangling edge.

use std::path::Path;

use oxc_span::SourceType;
use vize_carton::{String as CompactString, ToCompactString};

use super::VirtualProject;
use super::build::mirrored_virtual_path;
use super::dependency_scan::resolve_dependency;

impl VirtualProject {
    /// `content` of the shadowed module authored at `original_path`, with each
    /// rooted specifier that names a program file rebased onto its mirror copy.
    pub(super) fn rebase_rooted_shadow_specifiers(
        &self,
        content: &str,
        original_path: &Path,
    ) -> CompactString {
        if !may_contain_rooted_specifier(content) {
            return content.to_compact_string();
        }
        let Some(importer_dir) = original_path.parent() else {
            return content.to_compact_string();
        };
        let source_type = SourceType::from_path(original_path).unwrap_or_else(|_| SourceType::ts());
        self.rewriter
            .rewrite_with(content, source_type, |specifier, _| {
                if !is_rooted_specifier(specifier) {
                    return None;
                }
                let target = resolve_dependency(specifier, importer_dir, &self.project_root, &[])?;
                self.find_by_original(&target)?;
                let mirrored = mirrored_virtual_path(
                    &self.project_root,
                    &self.virtual_root,
                    Path::new(specifier),
                )
                .ok()?;
                mirrored.to_str().map(ToCompactString::to_compact_string)
            })
            .code
    }
}

/// A quoted literal that starts at a filesystem root. Most package modules
/// have none, and they skip the parse.
fn may_contain_rooted_specifier(content: &str) -> bool {
    content.contains("\"/") || content.contains("'/")
}

/// An absolute POSIX path. `//host` is a protocol-relative URL, not a file.
fn is_rooted_specifier(specifier: &str) -> bool {
    specifier.starts_with('/') && !specifier.starts_with("//") && Path::new(specifier).is_absolute()
}

#[cfg(test)]
mod tests {
    use super::{is_rooted_specifier, may_contain_rooted_specifier};

    #[test]
    fn only_rooted_paths_are_rebased() {
        assert!(is_rooted_specifier("/repo/src/module.js"));
        assert!(!is_rooted_specifier("//cdn.example/module.js"));
        assert!(!is_rooted_specifier("./module.js"));
        assert!(!is_rooted_specifier("@nuxt/ui"));
    }

    #[test]
    fn modules_without_a_rooted_literal_skip_the_parse() {
        assert!(may_contain_rooted_specifier(
            "export * from \"/repo/src/module.js\";"
        ));
        assert!(!may_contain_rooted_specifier(
            "export * from './runtime/types';"
        ));
    }
}
