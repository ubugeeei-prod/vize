//! Map `<script setup generic="...">` through its generated copies.
//!
//! The generator repeats the authored type-parameter list in every signature
//! that must stay generic (setup, `Props`, the component contract), adding
//! `= any` defaults or dropping `const` where a position requires it, and
//! instantiates `Props` with the constraints as fallback arguments. None of
//! those copies had a mapping, so one bad constraint surfaced as one unmapped
//! diagnostic per copy (#9753). The setup signature copy now maps its
//! parameters back to the attribute; the rest are echoes whose diagnostics
//! that copy already reports, so the content mapper suppresses them.

use std::ops::Range;

use vize_atelier_sfc::SfcDescriptor;

use crate::virtual_ts::{ProjectionFeatures, ProjectionMeta, VizeMapping};

/// One authored type parameter: `name extends constraint` and its default.
struct Param<'a> {
    core: (&'a str, Range<usize>),
    constraint: Option<&'a str>,
    default: Option<(&'a str, Range<usize>)>,
}

/// Unmapped generated copies of the authored parameter list.
pub(in crate::batch::virtual_project) struct GenericCopies {
    /// The authored attribute value.
    pub(in crate::batch::virtual_project) attribute: Range<usize>,
    /// Each copy's `[typeof ]Name<...>` and its parameter rows, if it declares them.
    copies: Vec<(Range<usize>, Option<Vec<Row>>)>,
}

impl GenericCopies {
    pub(in crate::batch::virtual_project) fn find(
        code: &str,
        mappings: &[VizeMapping],
        source: &str,
        descriptor: &SfcDescriptor<'_>,
    ) -> Option<Self> {
        let block = descriptor.script_setup.as_ref()?;
        let attribute = attribute_value(source, block.loc.tag_start..block.loc.start, "generic")?;
        let params: Vec<_> = list_items(source, attribute.clone())
            .into_iter()
            .filter_map(|item| authored_param(source, item))
            .collect();
        if params.is_empty() {
            return None;
        }
        let mut covered: Vec<_> = mappings.iter().map(|row| row.gen_range.clone()).collect();
        covered.sort_by_key(|range| range.start);
        let mut copies = Vec::new();
        for (open, _) in code.match_indices('<') {
            let Some(list) = generated_list(code, open + 1) else {
                continue;
            };
            let items = list_items(code, list.clone());
            let after = covered.partition_point(|row| row.start <= list.end);
            if items.len() != params.len()
                || covered
                    .get(..after)
                    .is_some_and(|rows| rows.iter().any(|row| open < row.end))
            {
                continue;
            }
            if let Some(rows) = declaration_rows(code, &items, &params) {
                copies.push((instantiation(code, open)..list.end + 1, Some(rows)));
            } else if is_fallback_argument_list(code, &items, &params) {
                copies.push((instantiation(code, open)..list.end + 1, None));
            }
        }
        (!copies.is_empty()).then_some(Self { attribute, copies })
    }

    /// Generated ranges whose diagnostics repeat another copy's.
    pub(in crate::batch::virtual_project) fn echoes(&self) -> impl Iterator<Item = &Range<usize>> {
        self.copies.iter().map(|(list, _)| list)
    }
}

/// Diagnostics on `Name<...>` (arity, `typeof` constraints) belong to the
/// copied list as much as those inside it.
fn instantiation(code: &str, open: usize) -> usize {
    let before = code.get(..open).unwrap_or_default();
    let name = before
        .trim_end_matches(|ch: char| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '.'));
    name.strip_suffix("typeof ").unwrap_or(name).len()
}

/// Map the setup signature copy back to the attribute, diagnostics only.
pub(super) fn map_generic_attribute(
    code: &str,
    mappings: &mut Vec<VizeMapping>,
    meta: &mut Vec<ProjectionMeta>,
    source: &str,
    descriptor: &SfcDescriptor<'_>,
) {
    let Some(rows) = GenericCopies::find(code, mappings, source, descriptor).and_then(|found| {
        found.copies.into_iter().find_map(|(copy, rows)| {
            // The setup function is the scope the authored parameters belong
            // to; other copies can carry artifacts such as an unused `T`.
            let setup = code.get(..copy.start)?.ends_with("function ")
                && code.get(copy.start..)?.starts_with("__setup<");
            setup.then_some(rows).flatten()
        })
    }) else {
        return;
    };
    meta.resize(mappings.len(), ProjectionMeta::default());
    for (gen_range, src_range) in rows {
        mappings.push(VizeMapping {
            gen_range,
            src_range,
            sub_spans: Vec::new(),
        });
        meta.push(ProjectionMeta {
            features: ProjectionFeatures::DIAGNOSTICS,
            ..ProjectionMeta::default()
        });
    }
}

type Row = (Range<usize>, Range<usize>);

/// A generated type-parameter list equal to the authored one up to `const`
/// modifiers and synthetic `= any` defaults.
fn declaration_rows(code: &str, items: &[Range<usize>], params: &[Param<'_>]) -> Option<Vec<Row>> {
    let mut rows = Vec::new();
    for (item, param) in items.iter().zip(params) {
        let (core, default) = split_default(code, without_const(code, item.clone()));
        if code.get(core.clone())? != param.core.0 {
            return None;
        }
        rows.push((core, param.core.1.clone()));
        match (default, &param.default) {
            (Some(generated), Some((text, authored))) if code.get(generated.clone())? == *text => {
                rows.push((generated, authored.clone()));
            }
            (None, None) => {}
            (Some(generated), None) if code.get(generated.clone())? == "any" => {}
            _ => return None,
        }
    }
    Some(rows)
}

/// `Props<C1, C2>`: each argument is its parameter's constraint, or the
/// `any` / `unknown` the generator substitutes when there is none to repeat.
fn is_fallback_argument_list(code: &str, items: &[Range<usize>], params: &[Param<'_>]) -> bool {
    let mut repeats_constraint = false;
    for (item, param) in items.iter().zip(params) {
        let text = code.get(trimmed(code, item.clone())).unwrap_or_default();
        if param
            .constraint
            .is_some_and(|constraint| text == constraint)
        {
            repeats_constraint = true;
        } else if !matches!(text, "any" | "unknown") {
            return false;
        }
    }
    repeats_constraint
}

fn authored_param(source: &str, item: Range<usize>) -> Option<Param<'_>> {
    let (core, default) = split_default(source, without_const(source, item));
    let piece = |range: Range<usize>| Some((source.get(range.clone())?, range));
    let text = source.get(core.clone())?;
    let constraint = text
        .match_indices("extends")
        .find(|(at, _)| {
            let spaced = |byte: Option<&u8>| byte.is_some_and(u8::is_ascii_whitespace);
            spaced(text.as_bytes().get(at.wrapping_sub(1)))
                && spaced(text.as_bytes().get(at + "extends".len()))
        })
        .and_then(|(at, _)| {
            source.get(trimmed(source, core.start + at + "extends".len()..core.end))
        });
    Some(Param {
        core: piece(core)?,
        constraint,
        default: default.and_then(piece),
    })
}

/// A parameter without its leading `const` modifier.
fn without_const(text: &str, item: Range<usize>) -> Range<usize> {
    let item = trimmed(text, item);
    text.get(item.clone())
        .and_then(|param| param.strip_prefix("const"))
        .filter(|rest| rest.starts_with(|ch: char| ch.is_ascii_whitespace()))
        .map_or(item.clone(), |rest| {
            item.end - rest.trim_start().len()..item.end
        })
}

/// Split `core = default` at its depth-0 `=` (never an `=>` arrow).
fn split_default(text: &str, item: Range<usize>) -> (Range<usize>, Option<Range<usize>>) {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut index = item.start;
    while index < item.end {
        match bytes.get(index) {
            Some(b'<' | b'(' | b'[' | b'{') => depth += 1,
            Some(b'>' | b')' | b']' | b'}') => depth -= 1,
            Some(b'=') if bytes.get(index + 1) == Some(&b'>') => index += 1,
            Some(b'=') if depth == 0 => {
                return (
                    trimmed(text, item.start..index),
                    Some(trimmed(text, index + 1..item.end)),
                );
            }
            _ => {}
        }
        index += 1;
    }
    (trimmed(text, item), None)
}

/// The body of the `<...>` list opened just before `start`.
fn generated_list(code: &str, start: usize) -> Option<Range<usize>> {
    let bytes = code.as_bytes();
    let mut depth = 0i32;
    let mut index = start;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'=' if bytes.get(index + 1) == Some(&b'>') => index += 1,
            b'<' | b'(' | b'[' | b'{' => depth += 1,
            b'>' if depth == 0 => return Some(start..index),
            b'>' | b')' | b']' | b'}' => depth -= 1,
            b';' if depth == 0 => return None,
            _ => {}
        }
        index += 1;
    }
    None
}

fn list_items(text: &str, list: Range<usize>) -> Vec<Range<usize>> {
    let bytes = text.as_bytes();
    let mut items = Vec::new();
    let mut depth = 0i32;
    let mut start = list.start;
    let mut index = list.start;
    while index < list.end {
        match bytes.get(index) {
            Some(b'=') if bytes.get(index + 1) == Some(&b'>') => index += 1,
            Some(b'<' | b'(' | b'[' | b'{') => depth += 1,
            Some(b'>' | b')' | b']' | b'}') => depth -= 1,
            Some(b',') if depth == 0 => {
                items.push(start..index);
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    items.push(start..list.end);
    items
}

fn trimmed(text: &str, range: Range<usize>) -> Range<usize> {
    let Some(slice) = text.get(range.clone()) else {
        return range;
    };
    let start = range.start + slice.len() - slice.trim_start().len();
    let end = range.end - (slice.len() - slice.trim_end().len());
    start..end.max(start)
}

/// The quoted value of `name="..."` inside an opening tag.
fn attribute_value(source: &str, tag: Range<usize>, name: &str) -> Option<Range<usize>> {
    let text = source.get(tag.clone())?;
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(found) = text.get(from..)?.find(name) {
        let start = from + found;
        let end = start + name.len();
        from = end;
        if !crate::text_scan::byte_before(bytes, start).is_some_and(|b| b.is_ascii_whitespace()) {
            continue;
        }
        let rest = text.get(end..)?.trim_start();
        let Some(value) = rest.strip_prefix('=').map(str::trim_start) else {
            continue;
        };
        let quote = *value.as_bytes().first()?;
        if !matches!(quote, b'"' | b'\'') {
            return None;
        }
        let value_start = tag.start + text.len() - value.len() + 1;
        let length = value.get(1..)?.find(quote as char)?;
        return Some(value_start..value_start + length);
    }
    None
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::batch::generate_vue_content_mapper_transform;

    /// One generated copy of `authored` maps onto the attribute and every other
    /// copy is an ignored echo, so each authored error reports once (#9753).
    fn assert_one_reporting_copy(generic: &str, authored: &str) {
        let source = vize_l0::cstr!(
            "<script setup lang=\"ts\" generic=\"{generic}\">\nconst props = defineProps<{{ value: T }}>()\n</script>\n<template>\n{{{{ props.value }}}}\n</template>\n"
        );
        let original = source.find(authored).expect("authored token");
        let result = generate_vue_content_mapper_transform(Path::new("App.vue"), &source)
            .expect("transform");
        let echoes: Vec<_> = result
            .diagnostic_directives
            .iter()
            .flat_map(|directives| &directives.directives)
            .map(|directive| directive.0[2]..directive.0[3])
            .collect();
        let mut reporting = 0;
        for copy in result.text.match_indices(authored).map(|(at, _)| at) {
            let span = result
                .mappings
                .iter()
                .map(|span| span.0)
                .find(|span| span[0] <= copy && copy + authored.len() <= span[0] + span[1]);
            if let Some([start, _, original_start, _, kind, _]) = span {
                assert_eq!((kind, original_start + copy - start), (0, original));
                reporting += 1;
            } else {
                assert!(
                    echoes
                        .iter()
                        .any(|echo| echo.start <= copy && copy + authored.len() <= echo.end),
                    "copy of {authored} at {copy} neither maps nor is ignored:\n{}",
                    result.text
                );
            }
        }
        assert_eq!(reporting, 1, "{}", result.text);
        assert!(!echoes.is_empty());
    }

    #[test]
    fn generic_constraints_and_defaults_report_from_one_copy() {
        assert_one_reporting_copy("T extends MissingType", "MissingType");
        assert_one_reporting_copy("T extends string = MissingDefault", "MissingDefault");
        assert_one_reporting_copy("const T extends MissingConst", "MissingConst");
        assert_one_reporting_copy("T extends\n  MissingMulti,\n  U", "MissingMulti");
    }
}
