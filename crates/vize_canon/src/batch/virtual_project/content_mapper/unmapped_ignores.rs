//! Ignore directives for virtual ranges no span maps, as Vue's mapper does.

use super::protocol::{
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperSpan,
    ContentMapperTransform, DIRECTIVE_POLICY_IGNORE,
};

impl ContentMapperTransform {
    /// Like Vue's mapper, ignore every virtual range no span covers:
    /// TypeScript otherwise reports scaffolding diagnostics at virtual
    /// positions. Explicit tuples keep their intervals, so expectations are
    /// never swallowed.
    pub(super) fn with_unmapped_ignores(mut self) -> Self {
        self.diagnostic_directives = unmapped_ignores(
            self.diagnostic_directives.take(),
            &self.mappings,
            &self.text,
        );
        self
    }
}

pub(super) fn unmapped_ignores(
    directives: Option<ContentMapperDiagnosticDirectives>,
    spans: &[ContentMapperSpan],
    generated: &str,
) -> Option<ContentMapperDiagnosticDirectives> {
    let generated_len = generated.len();
    let mut mapped = spans
        .iter()
        .filter(|ContentMapperSpan([_, length, ..])| *length > 0)
        .map(|ContentMapperSpan([start, length, ..])| *start..start + length)
        .collect::<Vec<_>>();
    mapped.sort_unstable_by_key(|range| range.start);
    let mut unmapped = Vec::new();
    let mut cursor = 0;
    for range in mapped {
        let end = opening_parentheses(generated.as_bytes(), cursor, range.start);
        if cursor < end {
            unmapped.push(cursor..end);
        }
        cursor = cursor.max(range.end);
    }
    if cursor < generated_len {
        unmapped.push(cursor..generated_len);
    }
    let mut covered = directives
        .as_ref()
        .into_iter()
        .flat_map(|table| &table.directives)
        .filter_map(|directive| {
            let [_, _, start, end, _, _] = directive.0;
            (start < end).then_some(start..end)
        })
        .collect::<Vec<_>>();
    // One ignore per prop-navigation block replaces an ignore per gap between
    // its key spans; a block an explicit tuple already touches keeps its gaps.
    let blocks = navigation_blocks(generated)
        .into_iter()
        .filter(|block| {
            !covered
                .iter()
                .any(|cover| cover.start < block.end && block.start < cover.end)
        })
        .collect::<Vec<_>>();
    covered.extend(blocks.iter().cloned());
    covered.sort_unstable_by_key(|range| range.start);
    let mut ignores = blocks;
    let mut next = 0;
    for range in unmapped {
        let mut start = range.start;
        while let Some(cover) = covered.get(next)
            && cover.end <= start
        {
            next += 1;
        }
        for cover in covered.iter().skip(next) {
            if cover.start >= range.end {
                break;
            }
            if start < cover.start {
                ignores.push(start..cover.start);
            }
            start = start.max(cover.end);
        }
        if start < range.end {
            ignores.push(start..range.end);
        }
    }
    if ignores.is_empty() {
        return directives;
    }
    // A navigation block and the gaps it touches form one ignored interval.
    ignores.sort_unstable_by_key(|range| range.start);
    ignores.dedup_by(|next, previous| {
        let touches = previous.end == next.start;
        if touches {
            previous.end = next.end;
        }
        touches
    });
    let mut directives = directives.unwrap_or(ContentMapperDiagnosticDirectives {
        unused_expect_directive_diagnostics: Vec::new(),
        directives: Vec::new(),
    });
    directives
        .directives
        .extend(ignores.into_iter().map(|range| {
            ContentMapperDiagnosticDirective([
                0,
                0,
                range.start,
                range.end,
                DIRECTIVE_POLICY_IGNORE,
                0,
            ])
        }));
    directives
        .directives
        .sort_by_key(|directive| directive.0[2]);
    Some(directives)
}

/// The `@ts-ignore` line `component_prop_navigation.rs` emits before its references.
const NAVIGATION_IGNORE: &str = "// @ts-ignore Navigation-only references; no diagnostic owner.";

/// `const __vize_props_nav_N = …;`, its `@ts-ignore` line and the one line of
/// `void __vize_props_nav_N…;` references that comment covers. No span maps
/// the const, and TypeScript reports nothing on the ignored line, so no owner
/// reads a diagnostic anywhere in the block.
fn navigation_blocks(generated: &str) -> Vec<std::ops::Range<usize>> {
    const DECLARATION: &str = "const __vize_props_nav_";
    let mut blocks = Vec::new();
    for start in memchr::memmem::find_iter(generated.as_bytes(), DECLARATION.as_bytes()) {
        let (Some(before), Some(rest)) = (generated.get(..start), generated.get(start..)) else {
            continue;
        };
        let indent = before.rsplit('\n').next().unwrap_or_default();
        if indent.bytes().any(|byte| byte != b' ') {
            continue;
        }
        let mut lines = rest.split_inclusive('\n');
        let (Some(declaration), Some(comment), Some(references)) =
            (lines.next(), lines.next(), lines.next())
        else {
            continue;
        };
        let Some((index, _)) = declaration
            .strip_prefix(DECLARATION)
            .and_then(|tail| tail.split_once(" = "))
        else {
            continue;
        };
        if index.is_empty() || !index.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        let owner = ["void __vize_props_nav_", index].concat();
        let statements = references.trim();
        let owned = statements.ends_with(';')
            && statements.split_inclusive(';').all(|statement| {
                statement
                    .trim_start()
                    .strip_prefix(owner.as_str())
                    .is_some_and(|key| key.starts_with(['.', '[']))
            });
        if comment.trim() == NAVIGATION_IGNORE && owned {
            let end = start + declaration.len() + comment.len() + references.trim_end().len();
            blocks.push(start..end);
        }
    }
    blocks
}

/// Parentheses opening a mapped span belong to the authored expression they
/// wrap: TypeScript anchors that expression's errors at the first `(`, so a
/// start-based ignore there would drop an authored check (#11657).
fn opening_parentheses(generated: &[u8], floor: usize, span_start: usize) -> usize {
    let mut end = span_start;
    let mut position = span_start;
    while position > floor {
        match generated.get(position - 1) {
            Some(b'(') => end = position - 1,
            Some(byte) if byte.is_ascii_whitespace() => {}
            _ => break,
        }
        position -= 1;
    }
    end
}
