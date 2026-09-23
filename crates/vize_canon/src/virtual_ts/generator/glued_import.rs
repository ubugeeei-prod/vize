//! The `<script setup>` import section that runs straight into the next
//! statement: `import { ref } from 'vue'ref('')`.
//!
//! The Vue toolchain hoists the import section verbatim to the top of the
//! module and emits the classic `<script>` directly behind it, so the missing
//! terminator is reported where the classic block begins, once. Every other
//! shape of the section ends in a terminator or a line break and needs no
//! special layout, and a lone `<script setup>` already reports the position
//! where its section ends.

/// A setup import section glued to the statement that follows it, in offsets
/// of the merged script.
pub(super) struct GluedImportSection {
    /// The import section, from its first import to the glued statement.
    pub(super) span: (u32, u32),
    /// The first token of the classic `<script>`: where the toolchain reports
    /// the missing terminator.
    pub(super) classic_token: (u32, u32),
}

impl GluedImportSection {
    /// `split` is the merged-script offset at which `<script setup>` starts;
    /// the classic block and one separator newline precede it.
    pub(super) fn find(script: &str, split: Option<(usize, usize)>) -> Option<Self> {
        let (setup_start, _) = split?;
        let classic = script.get(..setup_start.checked_sub(1)?)?;
        let setup = script.get(setup_start..)?;
        let section_start = skip_trivia(setup, 0);
        let section_end = glued_section_end(setup, section_start)?;
        let token_start = skip_trivia(classic, 0);
        let token_len = token_len(classic.get(token_start..)?)?;
        Some(Self {
            span: (
                u32::try_from(setup_start + section_start).ok()?,
                u32::try_from(setup_start + section_end).ok()?,
            ),
            classic_token: (
                u32::try_from(token_start).ok()?,
                u32::try_from(token_start + token_len).ok()?,
            ),
        })
    }
}

/// The offset of the statement an unterminated import runs into, when the
/// import section ends that way.
fn glued_section_end(source: &str, mut cursor: usize) -> Option<usize> {
    loop {
        let rest = source.get(cursor..)?;
        let after_keyword = rest.strip_prefix("import")?;
        // `import(...)` and `import.meta` are expressions, not declarations.
        if !after_keyword
            .starts_with(|c: char| c.is_whitespace() || matches!(c, '{' | '*' | '\'' | '"'))
        {
            return None;
        }
        let mut end = cursor + module_specifier_end(rest)?;
        loop {
            end +=
                source.get(end..)?.len() - source.get(end..)?.trim_start_matches([' ', '\t']).len();
            let rest = source.get(end..)?;
            if let Some(attributes) = import_attributes_len(rest) {
                end += attributes;
                continue;
            }
            break;
        }
        let rest = source.get(end..)?;
        match rest.chars().next() {
            None | Some('\n' | '\r') => cursor = skip_trivia(source, end),
            Some(';') => cursor = skip_trivia(source, end + 1),
            Some('/') if rest.starts_with("//") => cursor = skip_trivia(source, end),
            // Another import on the same line is still part of the section.
            Some(_) if rest.starts_with("import") => return None,
            Some(_) => return Some(end),
        }
    }
}

/// The length of `import … 'specifier'`, measured from the `import` keyword.
fn module_specifier_end(import: &str) -> Option<usize> {
    let open = import.find(['\'', '"', ';'])?;
    let quote = import.get(open..)?.chars().next().filter(|c| *c != ';')?;
    let body = import.get(open + 1..)?;
    let close = body.find([quote, '\n'])?;
    body.get(close..)?
        .starts_with(quote)
        .then_some(open + 1 + close + 1)
}

/// `with { … }` / `assert { … }` behind a module specifier.
fn import_attributes_len(rest: &str) -> Option<usize> {
    let keyword = ["with", "assert"]
        .into_iter()
        .find(|keyword| rest.starts_with(keyword))?;
    let after = rest.get(keyword.len()..)?;
    let braces = after.trim_start();
    braces.starts_with('{').then_some(())?;
    let close = braces.find('}')?;
    Some(keyword.len() + (after.len() - braces.len()) + close + 1)
}

/// Skip whitespace and comments.
fn skip_trivia(source: &str, mut cursor: usize) -> usize {
    loop {
        let rest = source.get(cursor..).unwrap_or_default();
        let trimmed = rest.trim_start();
        cursor += rest.len() - trimmed.len();
        let comment = if trimmed.starts_with("//") {
            trimmed.find('\n').unwrap_or(trimmed.len())
        } else if trimmed.starts_with("/*") {
            trimmed.find("*/").map_or(trimmed.len(), |end| end + 2)
        } else {
            return cursor;
        };
        cursor += comment;
    }
}

fn token_len(source: &str) -> Option<usize> {
    let first = source.chars().next()?;
    if !(first.is_alphanumeric() || matches!(first, '_' | '$')) {
        return Some(first.len_utf8());
    }
    Some(
        source
            .find(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '$')))
            .unwrap_or(source.len()),
    )
}

#[cfg(test)]
mod tests {
    use super::GluedImportSection;

    fn find(classic: &str, setup: &str) -> Option<((u32, u32), (u32, u32))> {
        let script = vize_carton::cstr!("{classic}\n{setup}");
        GluedImportSection::find(&script, Some((classic.len() + 1, 0)))
            .map(|glued| (glued.span, glued.classic_token))
    }

    #[test]
    fn an_import_that_runs_into_a_statement_is_glued() {
        // `export default {};\n` is 19 bytes, the separator one more.
        assert_eq!(
            find(
                "export default {};\n",
                "\nimport { ref } from 'vue'ref('');\n"
            ),
            Some(((21, 46), (0, 6)))
        );
        assert_eq!(
            find(
                "// c\n  foo();",
                "import a from 'a';\nimport b from \"b\" with { type: 'json' } run()"
            ),
            Some(((14, 73), (7, 10)))
        );
    }

    #[test]
    fn terminated_sections_are_not() {
        for setup in [
            "import { ref } from 'vue'\nref('');",
            "import { ref } from 'vue';ref('');",
            "import { ref } from 'vue' // why\nref('');",
            "import a from 'a' import b from 'b'\n",
            "const x = import('x')",
            "import.meta.env",
            "ref('')",
        ] {
            assert_eq!(find("export default {};\n", setup), None, "{setup}");
        }
        assert_eq!(find("", "import { ref } from 'vue'ref('');"), None);
    }
}
