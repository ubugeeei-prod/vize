//! Vue dialect profiles and structural petite-vue detection.
//!
//! A document's dialect decides which directive set, completions, and lint
//! gating apply. The dialect is resolved once per document from an explicit
//! config key when present, otherwise from a structural scan of the document's
//! `<script>` tags (petite-vue CDN/module `src`, an ES import of the
//! petite-vue package, or a `PetiteVue.createApp` global call). Raw substring
//! sniffing over the whole document is deliberately avoided so that prose or
//! comments merely mentioning "petite-vue" never flip the dialect.

use serde::{Deserialize, Serialize};

/// The Vue template dialect a document is written in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VueDialect {
    /// Standard Vue 3 (SFCs and full-build templates).
    #[default]
    Vue,
    /// [petite-vue](https://github.com/vuejs/petite-vue) standalone HTML documents.
    PetiteVue,
}

impl VueDialect {
    /// Returns true for the petite-vue dialect.
    #[inline]
    pub fn is_petite_vue(self) -> bool {
        matches!(self, Self::PetiteVue)
    }
}

/// Resolve the dialect for a standalone HTML document.
///
/// An explicit config value always wins; otherwise the document is scanned
/// structurally with [`detect_petite_vue_document`].
#[inline]
pub fn standalone_html_dialect(configured: Option<VueDialect>, content: &str) -> VueDialect {
    match configured {
        Some(dialect) => dialect,
        None if detect_petite_vue_document(content) => VueDialect::PetiteVue,
        None => VueDialect::Vue,
    }
}

/// Returns true when a module specifier resolves to the petite-vue package.
///
/// Accepts the bare specifier (`petite-vue`), deep imports
/// (`petite-vue/dist/...`), and URL or path specifiers whose path contains a
/// petite-vue segment (`https://unpkg.com/petite-vue@0.4.1/dist/petite-vue.es.js`,
/// `/node_modules/petite-vue/...`). Query strings and fragments are ignored.
/// Lookalike packages such as `petite-vuex` do not match.
pub fn is_petite_vue_module(specifier: &str) -> bool {
    let path = specifier
        .split(['?', '#'])
        .next()
        .unwrap_or(specifier)
        .trim();
    if path == "petite-vue" || path.starts_with("petite-vue/") {
        return true;
    }
    path.split('/').any(|segment| {
        segment
            .strip_prefix("petite-vue")
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('@') || rest.starts_with('.'))
    })
}

/// Structurally detect petite-vue usage in a standalone HTML document.
///
/// The scan only inspects `<script>` start tags and inline script bodies:
///
/// - `<script src="...">` where the `src` resolves to the petite-vue package
///   (CDN URL, node_modules path, or bare specifier), e.g.
///   `<script src="https://unpkg.com/petite-vue" defer init>`.
/// - Inline scripts containing an ES import (static, side-effect, or dynamic)
///   whose specifier resolves to the petite-vue package.
/// - Inline scripts calling the `PetiteVue.createApp` IIFE global.
///
/// HTML comments are skipped, and inside inline scripts JS comments and string
/// literals are skipped, so a document merely *mentioning* petite-vue is never
/// detected as petite-vue.
pub fn detect_petite_vue_document(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        let Some(relative) = content[pos..].find('<') else {
            return false;
        };
        let start = pos + relative;
        let rest = &content[start..];

        if let Some(comment) = rest.strip_prefix("<!--") {
            // Skip HTML comments entirely.
            match comment.find("-->") {
                Some(end) => pos = start + 4 + end + 3,
                None => return false,
            }
            continue;
        }

        if !starts_with_script_tag(rest) {
            pos = start + 1;
            continue;
        }

        let tag_body_start = start + "<script".len();
        let Some((attrs_end, src)) = parse_start_tag_attributes(content, tag_body_start) else {
            return false;
        };

        if let Some(src) = src {
            if is_petite_vue_module(src) {
                return true;
            }
            // External script: no inline body to inspect.
            pos = attrs_end;
            continue;
        }

        // Inline script: scan the body up to the closing tag.
        let body_start = attrs_end;
        let body_end = find_script_close(content, body_start).unwrap_or(content.len());
        if inline_script_uses_petite_vue(&content[body_start..body_end]) {
            return true;
        }
        pos = body_end;
    }

    false
}

/// Returns true when `rest` starts a `<script` start tag (case-insensitive,
/// followed by whitespace, `>`, or `/`).
fn starts_with_script_tag(rest: &str) -> bool {
    let bytes = rest.as_bytes();
    if bytes.len() < 7 || !bytes[1..7].eq_ignore_ascii_case(b"script") {
        return false;
    }
    matches!(
        bytes.get(7),
        Some(b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r')
    )
}

/// Parse the attributes of a start tag beginning right after the tag name.
///
/// Returns the offset just past the closing `>` and the value of the `src`
/// attribute, if any. Quoted attribute values are honoured so a `>` inside a
/// value does not terminate the tag. Returns `None` when the tag is unclosed.
fn parse_start_tag_attributes(content: &str, mut pos: usize) -> Option<(usize, Option<&str>)> {
    let bytes = content.as_bytes();
    let mut src = None;

    while pos < bytes.len() {
        match bytes[pos] {
            b'>' => return Some((pos + 1, src)),
            b'/' | b' ' | b'\t' | b'\n' | b'\r' => {
                pos += 1;
            }
            _ => {
                // Attribute name.
                let name_start = pos;
                while pos < bytes.len()
                    && !matches!(
                        bytes[pos],
                        b'=' | b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r'
                    )
                {
                    pos += 1;
                }
                let name = &content[name_start..pos];

                while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }
                if bytes.get(pos) != Some(&b'=') {
                    // Boolean attribute (e.g. `defer`, `init`).
                    continue;
                }
                pos += 1;
                while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }

                let value = match bytes.get(pos) {
                    Some(&quote @ (b'"' | b'\'')) => {
                        let value_start = pos + 1;
                        let relative_end = content[value_start..].find(quote as char)?;
                        pos = value_start + relative_end + 1;
                        &content[value_start..value_start + relative_end]
                    }
                    _ => {
                        let value_start = pos;
                        while pos < bytes.len()
                            && !matches!(bytes[pos], b'>' | b' ' | b'\t' | b'\n' | b'\r')
                        {
                            pos += 1;
                        }
                        &content[value_start..pos]
                    }
                };
                if name.eq_ignore_ascii_case("src") {
                    src = Some(value);
                }
            }
        }
    }

    None
}

/// Find the offset of the next `</script` close tag (case-insensitive).
fn find_script_close(content: &str, from: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut pos = from;
    while pos < bytes.len() {
        let relative = content[pos..].find('<')?;
        let start = pos + relative;
        let rest = &bytes[start..];
        if rest.len() >= 9 && rest[1] == b'/' && rest[2..8].eq_ignore_ascii_case(b"script") {
            return Some(start);
        }
        pos = start + 1;
    }
    None
}

/// Scan an inline script body for structural petite-vue usage.
///
/// Detects ES imports of the petite-vue package (static `import ... from`,
/// side-effect `import "..."`, dynamic `import("...")`) and the
/// `PetiteVue.createApp` IIFE global. Comments and unrelated string literals
/// are skipped so mentions of petite-vue in prose never match.
fn inline_script_uses_petite_vue(script: &str) -> bool {
    let bytes = script.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        match bytes[pos] {
            b'/' if bytes.get(pos + 1) == Some(&b'/') => {
                pos = match script[pos..].find('\n') {
                    Some(end) => pos + end + 1,
                    None => bytes.len(),
                };
            }
            b'/' if bytes.get(pos + 1) == Some(&b'*') => {
                pos = match script[pos + 2..].find("*/") {
                    Some(end) => pos + 2 + end + 2,
                    None => bytes.len(),
                };
            }
            b'"' | b'\'' | b'`' => {
                pos = skip_string_literal(bytes, pos);
            }
            byte if is_ident_start(byte) => {
                let word_start = pos;
                while pos < bytes.len() && is_ident_char(bytes[pos]) {
                    pos += 1;
                }
                match &script[word_start..pos] {
                    "import" | "from" if import_specifier_is_petite_vue(script, pos) => {
                        return true;
                    }
                    "PetiteVue" if followed_by_create_app(script, pos) => return true,
                    _ => {}
                }
            }
            _ => pos += 1,
        }
    }

    false
}

/// Check whether an `import`/`from` keyword at `pos` introduces a petite-vue
/// module specifier (`from "spec"`, `import "spec"`, or `import("spec")`).
fn import_specifier_is_petite_vue(script: &str, mut pos: usize) -> bool {
    let bytes = script.as_bytes();
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    // Dynamic import: import("spec")
    if bytes.get(pos) == Some(&b'(') {
        pos += 1;
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
    }
    let Some(&quote @ (b'"' | b'\'')) = bytes.get(pos) else {
        return false;
    };
    let value_start = pos + 1;
    let Some(relative_end) = script[value_start..].find(quote as char) else {
        return false;
    };
    is_petite_vue_module(&script[value_start..value_start + relative_end])
}

/// Check for `.createApp` (allowing whitespace) after a `PetiteVue` token.
fn followed_by_create_app(script: &str, mut pos: usize) -> bool {
    let bytes = script.as_bytes();
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    if bytes.get(pos) != Some(&b'.') {
        return false;
    }
    pos += 1;
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    let word_start = pos;
    while pos < bytes.len() && is_ident_char(bytes[pos]) {
        pos += 1;
    }
    &script[word_start..pos] == "createApp"
}

/// Skip a JS string literal starting at `pos`; returns the offset past it.
fn skip_string_literal(bytes: &[u8], pos: usize) -> usize {
    let quote = bytes[pos];
    let mut pos = pos + 1;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' => pos += 2,
            byte if byte == quote => return pos + 1,
            _ => pos += 1,
        }
    }
    pos
}

#[inline]
fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

#[inline]
fn is_ident_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod tests {
    use super::{
        VueDialect, detect_petite_vue_document, is_petite_vue_module, standalone_html_dialect,
    };

    #[test]
    fn module_specifier_matches_petite_vue_forms() {
        assert!(is_petite_vue_module("petite-vue"));
        assert!(is_petite_vue_module("petite-vue/dist/petite-vue.es.js"));
        assert!(is_petite_vue_module("https://unpkg.com/petite-vue"));
        assert!(is_petite_vue_module("https://unpkg.com/petite-vue?module"));
        assert!(is_petite_vue_module(
            "https://unpkg.com/petite-vue@0.4.1/dist/petite-vue.iife.js"
        ));
        assert!(is_petite_vue_module(
            "/node_modules/petite-vue/dist/petite-vue.es.js"
        ));
        assert!(is_petite_vue_module(
            "https://cdn.jsdelivr.net/npm/petite-vue@0.4/dist/petite-vue.min.js"
        ));
    }

    #[test]
    fn module_specifier_rejects_lookalikes() {
        assert!(!is_petite_vue_module("vue"));
        assert!(!is_petite_vue_module("petite-vuex"));
        assert!(!is_petite_vue_module("https://unpkg.com/petite-vuex"));
        assert!(!is_petite_vue_module("my-petite-vue"));
        assert!(!is_petite_vue_module("./libs/petite-vue-helpers.js"));
    }

    #[test]
    fn detects_cdn_script_src() {
        let content = r#"<!doctype html>
<html>
<body>
  <div v-scope>{{ count }}</div>
  <script src="https://unpkg.com/petite-vue" defer init></script>
</body>
</html>"#;
        assert!(detect_petite_vue_document(content));
    }

    #[test]
    fn detects_unquoted_script_src() {
        let content = r#"<script src=https://unpkg.com/petite-vue defer init></script>"#;
        assert!(detect_petite_vue_document(content));
    }

    #[test]
    fn detects_inline_module_import() {
        let content = r#"<script type="module">
  import { createApp } from 'https://unpkg.com/petite-vue?module'
  createApp({ count: 0 }).mount()
</script>"#;
        assert!(detect_petite_vue_document(content));
    }

    #[test]
    fn detects_side_effect_and_dynamic_imports() {
        assert!(detect_petite_vue_document(
            r#"<script type="module">import "petite-vue"</script>"#
        ));
        assert!(detect_petite_vue_document(
            r#"<script type="module">const m = await import("petite-vue")</script>"#
        ));
    }

    #[test]
    fn detects_petite_vue_global_create_app() {
        let content = r#"<script src="/js/vendor.js"></script>
<script>
PetiteVue.createApp({ count: 0 }).mount()
</script>"#;
        assert!(detect_petite_vue_document(content));
    }

    #[test]
    fn ignores_mentions_in_html_comments_and_text() {
        let content = r#"<!doctype html>
<!-- This page is NOT using petite-vue, see PetiteVue.createApp docs -->
<!-- <script src="https://unpkg.com/petite-vue" defer init></script> -->
<body>
  <p>petite-vue is a 6kb subset of Vue.</p>
  <script src="https://unpkg.com/vue@3/dist/vue.global.js"></script>
</body>"#;
        assert!(!detect_petite_vue_document(content));
    }

    #[test]
    fn ignores_mentions_in_script_comments_and_strings() {
        let content = r#"<script>
// import { createApp } from 'petite-vue'
/* PetiteVue.createApp() is the IIFE entrypoint */
const docs = "https://unpkg.com/petite-vue";
const note = 'PetiteVue.createApp';
</script>"#;
        assert!(!detect_petite_vue_document(content));
    }

    #[test]
    fn ignores_plain_vue_documents() {
        let content = r#"<script src="https://unpkg.com/vue@3"></script>
<script>
Vue.createApp({ data: () => ({ count: 0 }) }).mount('#app')
</script>"#;
        assert!(!detect_petite_vue_document(content));
    }

    #[test]
    fn config_overrides_detection() {
        let petite = r#"<script src="https://unpkg.com/petite-vue" defer init></script>"#;
        let plain = "<div>{{ count }}</div>";

        assert_eq!(
            standalone_html_dialect(Some(VueDialect::Vue), petite),
            VueDialect::Vue
        );
        assert_eq!(
            standalone_html_dialect(Some(VueDialect::PetiteVue), plain),
            VueDialect::PetiteVue
        );
        assert_eq!(standalone_html_dialect(None, petite), VueDialect::PetiteVue);
        assert_eq!(standalone_html_dialect(None, plain), VueDialect::Vue);
    }

    #[test]
    fn dialect_serde_uses_kebab_case() {
        assert_eq!(
            serde_json::from_str::<VueDialect>("\"petite-vue\"").unwrap(),
            VueDialect::PetiteVue
        );
        assert_eq!(
            serde_json::from_str::<VueDialect>("\"vue\"").unwrap(),
            VueDialect::Vue
        );
        assert_eq!(
            serde_json::to_string(&VueDialect::PetiteVue).unwrap(),
            "\"petite-vue\""
        );
    }
}
