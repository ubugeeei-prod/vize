//! Script-language classification for `.vue` files, and the `checkJs` gate
//! that decides whether a JavaScript SFC is type-checked at all.
//!
//! Vize always generates a `.vue.ts` virtual module, even when the SFC's script
//! block is plain JavaScript. TypeScript itself never checks a `.js` file
//! unless `checkJs` is on (`@vue/language-core` gives a `lang="js"` SFC a
//! `.js` virtual extension for exactly that reason), so generating `.ts`
//! unconditionally made `vize check` report `noImplicitAny` and `never[]`
//! inference errors on code `tsc`/`vue-tsc` deliberately leave alone (#3322).
//!
//! The classification here restores the TypeScript contract: an SFC whose
//! script block is JavaScript is *unchecked* unless the project enables
//! `checkJs`, or the block opts in with a leading `// @ts-check` pragma.
//! Nothing else changes — the virtual module is still emitted and still
//! contributes its exported types to importers, and Vize's own SFC/template
//! parse diagnostics are unaffected because they never pass through the
//! TypeScript diagnostic mapper.

use vize_atelier_sfc::SfcDescriptor;

/// TypeScript-flavoured `<script lang>` values. Everything else (`js`, `jsx`,
/// or an absent `lang`) is JavaScript.
fn is_typescript_lang(lang: &str) -> bool {
    matches!(lang, "ts" | "tsx" | "mts" | "cts")
}

fn is_jsx_like_lang(lang: &str) -> bool {
    matches!(lang, "jsx" | "tsx")
}

pub(super) fn descriptor_uses_jsx_script(descriptor: &SfcDescriptor) -> bool {
    descriptor
        .script
        .as_ref()
        .and_then(|script| script.lang.as_deref())
        .is_some_and(is_jsx_like_lang)
        || descriptor
            .script_setup
            .as_ref()
            .and_then(|script| script.lang.as_deref())
            .is_some_and(is_jsx_like_lang)
}

/// Whether this SFC's TypeScript diagnostics must be suppressed when the
/// project does not enable `checkJs`.
///
/// True only when the SFC actually has a script block and every present block
/// is JavaScript. A script-less SFC is *not* JavaScript: `vue-tsc` type-checks
/// a template-only `.vue` under a plain strict tsconfig, so its template
/// diagnostics must keep flowing.
pub(super) fn descriptor_is_unchecked_javascript(descriptor: &SfcDescriptor) -> bool {
    let blocks = [descriptor.script.as_ref(), descriptor.script_setup.as_ref()];
    let mut has_script_block = false;
    for block in blocks.into_iter().flatten() {
        has_script_block = true;
        if block.lang.as_deref().is_some_and(is_typescript_lang) {
            return false;
        }
        if opts_into_type_checking(&block.content) {
            return false;
        }
    }
    has_script_block
}

/// TypeScript's per-file opt-in for an otherwise unchecked JavaScript file: a
/// `// @ts-check` pragma in the leading comments. `ts-check` is registered as a
/// single-line pragma in `tsc`'s own scan, so `/* @ts-check */` never opts a
/// file in; a leading block comment is plain trivia. Only comments and blank
/// lines may precede the pragma.
fn opts_into_type_checking(content: &str) -> bool {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("//") {
            if rest.trim() == "@ts-check" {
                return true;
            }
            continue;
        }
        // A closed leading block comment is trivia: it cannot opt in, but it
        // also must not stop the scan for a `// @ts-check` on a later line.
        if line.starts_with("/*") && line.ends_with("*/") {
            continue;
        }
        return false;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{descriptor_is_unchecked_javascript, opts_into_type_checking};
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

    fn descriptor_for(source: &str) -> vize_atelier_sfc::SfcDescriptor<'_> {
        parse_sfc(source, SfcParseOptions::default()).expect("SFC must parse")
    }

    #[test]
    fn plain_script_setup_is_unchecked_javascript() {
        let descriptor = descriptor_for("<script setup>\nconst a = 1\n</script>\n");
        assert!(descriptor_is_unchecked_javascript(&descriptor));
    }

    #[test]
    fn typescript_script_setup_is_checked() {
        let descriptor = descriptor_for("<script setup lang=\"ts\">\nconst a = 1\n</script>\n");
        assert!(!descriptor_is_unchecked_javascript(&descriptor));
    }

    #[test]
    fn typescript_options_block_keeps_a_javascript_setup_block_checked() {
        let descriptor = descriptor_for(
            "<script lang=\"ts\">\nexport default {}\n</script>\n<script setup>\nconst a = 1\n</script>\n",
        );
        assert!(!descriptor_is_unchecked_javascript(&descriptor));
    }

    #[test]
    fn script_less_sfc_is_not_javascript() {
        let descriptor = descriptor_for("<template>\n  <div />\n</template>\n");
        assert!(!descriptor_is_unchecked_javascript(&descriptor));
    }

    #[test]
    fn ts_check_pragma_opts_a_javascript_block_back_in() {
        let descriptor = descriptor_for("<script setup>\n// @ts-check\nconst a = 1\n</script>\n");
        assert!(!descriptor_is_unchecked_javascript(&descriptor));
    }

    #[test]
    fn ts_check_pragma_is_only_honored_in_leading_comments() {
        assert!(opts_into_type_checking("\n// @ts-check\nconst a = 1\n"));
        assert!(opts_into_type_checking(
            "/* header */\n// @ts-check\nconst a = 1\n"
        ));
        assert!(!opts_into_type_checking("/* @ts-check */\nconst a = 1\n"));
        assert!(!opts_into_type_checking("const a = 1\n// @ts-check\n"));
        assert!(!opts_into_type_checking("// not a pragma\nconst a = 1\n"));
    }
}
