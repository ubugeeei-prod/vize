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

use crate::script_parse::is_typescript_lang;
use vize_atelier_sfc::SfcDescriptor;
use vize_carton::{String, cstr};

fn is_jsx_like_lang(lang: &str) -> bool {
    matches!(lang, "jsx" | "tsx")
}

/// The script languages TypeScript can read. Anything else (`lang="gleam"`,
/// `lang="coffee"`) is a block for some other toolchain: `vue-tsc` generates
/// no code for it, so it contributes neither bindings nor syntax errors, and
/// the template is checked as if the SFC had no script at all.
fn is_typescript_readable_lang(lang: &str) -> bool {
    matches!(
        lang,
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts"
    )
}

/// Shape the script blocks the way the checker reads them: drop the ones in a
/// language TypeScript cannot read, and load an external `<script src>`.
pub(super) fn prepare_script_blocks(descriptor: SfcDescriptor<'_>) -> SfcDescriptor<'_> {
    with_external_script_source(without_foreign_script_blocks(descriptor))
}

/// `<script src="./component.ts">` keeps its code in another module. The
/// block becomes that module's default export, which is what `vue-tsc`
/// generates too: the external file is checked as its own TypeScript source,
/// and the template resolves its names on the component it exports. A `.ts`
/// or `.tsx` specifier is imported with its emitted extension so the import
/// also resolves under `moduleResolution: node16`/`nodenext`.
fn with_external_script_source(mut descriptor: SfcDescriptor<'_>) -> SfcDescriptor<'_> {
    let Some(script) = descriptor.script.as_mut() else {
        return descriptor;
    };
    let Some(src) = script
        .src
        .as_deref()
        .map(str::trim)
        .filter(|src| !src.is_empty())
    else {
        return descriptor;
    };
    if !script.content.trim().is_empty() {
        return descriptor;
    }
    let specifier = src
        .strip_suffix(".tsx")
        .map(|stem| cstr!("{stem}.jsx"))
        .or_else(|| {
            src.strip_suffix(".ts")
                .filter(|stem| !stem.ends_with(".d"))
                .map(|stem| cstr!("{stem}.js"))
        })
        .unwrap_or_else(|| String::from(src));
    let specifier = serde_json::to_string(specifier.as_str()).expect("string serialization");
    let content = cstr!("import __vize_src from {specifier};\nexport default __vize_src;\n");
    script.content = std::borrow::Cow::Owned(content.as_str().into());
    descriptor
}

/// Drop the script blocks whose language TypeScript cannot read.
fn without_foreign_script_blocks(mut descriptor: SfcDescriptor<'_>) -> SfcDescriptor<'_> {
    let foreign = |block: &vize_atelier_sfc::SfcScriptBlock<'_>| {
        block
            .lang
            .as_deref()
            .is_some_and(|lang| !is_typescript_readable_lang(lang.trim()))
    };
    if descriptor.script.as_ref().is_some_and(foreign) {
        descriptor.script = None;
    }
    if descriptor.script_setup.as_ref().is_some_and(foreign) {
        descriptor.script_setup = None;
    }
    descriptor
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

#[cfg(test)]
mod foreign_script_tests {
    use super::{prepare_script_blocks, without_foreign_script_blocks};
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
    use vize_carton::cstr;

    #[test]
    fn an_external_script_source_becomes_its_module_default_export() {
        let descriptor = parse_sfc(
            "<script lang=\"ts\" src=\"./typescript.ts\"></script>\n<template><div /></template>\n",
            SfcParseOptions::default(),
        )
        .expect("SFC must parse");
        let descriptor = prepare_script_blocks(descriptor);
        assert_eq!(
            descriptor
                .script
                .as_ref()
                .map(|block| block.content.as_ref()),
            Some("import __vize_src from \"./typescript.js\";\nexport default __vize_src;\n")
        );

        for (source, specifier) in [
            ("./typescript.js", "\"./typescript.js\""),
            ("./component.tsx", "\"./component.jsx\""),
            ("./types.d.ts", "\"./types.d.ts\""),
            ("./javascript.jsx", "\"./javascript.jsx\""),
        ] {
            let sfc = cstr!("<script lang=\"ts\" src=\"{source}\"></script>\n");
            let descriptor =
                parse_sfc(sfc.as_str(), SfcParseOptions::default()).expect("SFC must parse");
            let descriptor = prepare_script_blocks(descriptor);
            let expected =
                cstr!("import __vize_src from {specifier};\nexport default __vize_src;\n");
            assert_eq!(
                descriptor
                    .script
                    .as_ref()
                    .map(|block| block.content.as_ref()),
                Some(expected.as_str()),
                "{source}"
            );
        }

        // Authored content wins over `src`; the compiler reports that conflict.
        let descriptor = parse_sfc(
            "<script lang=\"ts\" src=\"./typescript.ts\">export default {}</script>\n",
            SfcParseOptions::default(),
        )
        .expect("SFC must parse");
        let descriptor = prepare_script_blocks(descriptor);
        assert_eq!(
            descriptor
                .script
                .as_ref()
                .map(|block| block.content.as_ref()),
            Some("export default {}")
        );
    }

    #[test]
    fn a_foreign_language_script_block_is_ignored_and_a_readable_one_is_kept() {
        let descriptor = parse_sfc(
            "<script lang=\"gleam\">import gleam/io</script>\n<script setup lang=\"ts\">const a = 1;</script>\n",
            SfcParseOptions::default(),
        )
        .expect("SFC must parse");
        let descriptor = without_foreign_script_blocks(descriptor);
        assert!(descriptor.script.is_none());
        assert_eq!(
            descriptor
                .script_setup
                .as_ref()
                .map(|block| block.content.as_ref()),
            Some("const a = 1;")
        );

        let descriptor = parse_sfc(
            "<script>export default {}</script>\n<script setup lang=\"coffee\">a = 1</script>\n",
            SfcParseOptions::default(),
        )
        .expect("SFC must parse");
        let descriptor = without_foreign_script_blocks(descriptor);
        assert!(descriptor.script.is_some());
        assert!(descriptor.script_setup.is_none());
    }
}
