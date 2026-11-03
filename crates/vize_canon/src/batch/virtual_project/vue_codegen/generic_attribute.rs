//! Echoes of `<script setup generic="...">` in generated code.
//!
//! The generator maps the setup signature onto the attribute: that copy is
//! the authored list's one owner, for diagnostics and navigation alike. Every
//! other unmapped copy of one of the plan's rendered spellings is an echo.

use std::ops::Range;

use crate::virtual_ts::VizeMapping;
use crate::virtual_ts::generic_parameters::GenericParameters;

/// Generated echoes of one authored parameter list.
pub(in crate::batch::virtual_project) struct GenericCopies {
    pub(in crate::batch::virtual_project) attribute: Range<usize>,
    echoes: Vec<Range<usize>>,
}

impl GenericCopies {
    pub(in crate::batch::virtual_project) fn find(
        code: &str,
        mappings: &[VizeMapping],
        parameters: &GenericParameters,
    ) -> Option<Self> {
        let attribute = parameters.attribute()?;
        let mut covered: Vec<_> = mappings.iter().map(|row| row.gen_range.clone()).collect();
        covered.sort_by_key(|range| range.start);
        // Match complete, already-rendered lists. Quoted commas, nested types,
        // arrows and comments remain opaque here; OXC prepared their ranges.
        let declarations = parameters.declaration_variants().into_iter();
        // Only a repeated authored constraint can carry an echoed diagnostic.
        // Bare names or unconstrained `unknown`/`any` arguments can occur in
        // unrelated generated helpers and do not establish that ownership.
        let fallback = parameters
            .has_repeated_fallback_constraint()
            .then_some(parameters.fallback.as_str());
        let mut copies: Vec<_> = declarations
            .chain(fallback)
            .flat_map(|rendered| exact_lists(code, rendered))
            .filter(|list| !overlaps_mapped(&covered, list.clone()))
            .map(|list| instantiation(code, list.start)..list.end)
            .collect();
        // Outer lists own any matching list nested inside their
        // constraints/defaults.
        copies.sort_by(|left, right| {
            left.start
                .cmp(&right.start)
                .then_with(|| right.end.cmp(&left.end))
        });
        let mut echoes: Vec<Range<usize>> = Vec::new();
        for copy in copies {
            if echoes.last().is_none_or(|last| last.end <= copy.start) {
                echoes.push(copy);
            }
        }
        (!echoes.is_empty()).then_some(Self { attribute, echoes })
    }

    /// Generated ranges whose diagnostics repeat the setup signature's.
    pub(in crate::batch::virtual_project) fn echoes(&self) -> impl Iterator<Item = &Range<usize>> {
        self.echoes.iter()
    }

    pub(in crate::batch::virtual_project) fn shift_generated(&mut self, offset: usize) {
        for echo in &mut self.echoes {
            echo.start += offset;
            echo.end += offset;
        }
    }

    /// The generated text `[start, start + old_len)` grew to `new_len` bytes.
    pub(in crate::batch::virtual_project) fn note_generated_replacement(
        &mut self,
        start: usize,
        old_len: usize,
        new_len: usize,
    ) {
        let Some(extra) = new_len.checked_sub(old_len).filter(|extra| *extra > 0) else {
            return;
        };
        let old_end = start + old_len;
        for echo in &mut self.echoes {
            shift_generated_range(echo, start, old_end, extra);
        }
    }
}

fn shift_generated_range(range: &mut Range<usize>, start: usize, old_end: usize, extra: usize) {
    if range.start >= old_end {
        range.start += extra;
        range.end += extra;
    } else if range.end > start && range.end >= old_end {
        range.end += extra;
    }
}

fn exact_lists<'a>(code: &'a str, rendered: &'a str) -> impl Iterator<Item = Range<usize>> + 'a {
    code.match_indices(rendered).filter_map(move |(start, _)| {
        let open = start.checked_sub(1)?;
        let end = start + rendered.len();
        (code.as_bytes().get(open) == Some(&b'<') && code.as_bytes().get(end) == Some(&b'>'))
            .then_some(open..end + 1)
    })
}

fn overlaps_mapped(covered: &[Range<usize>], list: Range<usize>) -> bool {
    let after = covered.partition_point(|row| row.start < list.end);
    covered
        .get(..after)
        .is_some_and(|rows| rows.iter().any(|row| list.start < row.end))
}

/// Diagnostics on `Name<...>` (arity, `typeof` constraints) belong to the
/// copied list as much as those inside it.
fn instantiation(code: &str, open: usize) -> usize {
    let before = code.get(..open).unwrap_or_default();
    let name = before
        .trim_end_matches(|ch: char| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '.'));
    name.strip_suffix("typeof ").unwrap_or(name).len()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::batch::generate_vue_content_mapper_transform;
    use crate::virtual_ts::generic_parameters::GenericParameters;

    use super::GenericCopies;
    use crate::virtual_ts::VizeMapping;

    /// One generated copy of `authored` maps onto the attribute and every other
    /// copy is an ignored echo, so each authored error reports once (#9753).
    /// That copy navigates like setup code (#11503).
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
        let setup = source.find("value: T").expect("setup body");
        let setup_features = result
            .mappings
            .iter()
            .find(|span| span.0[2] <= setup && setup < span.0[2] + span.0[3])
            .map(|span| span.0[5])
            .expect("mapped setup body");
        assert_ne!(setup_features, 0);
        let mut reporting = 0;
        for copy in result.text.match_indices(authored).map(|(at, _)| at) {
            let span = result
                .mappings
                .iter()
                .map(|span| span.0)
                .find(|span| span[0] <= copy && copy + authored.len() <= span[0] + span[1]);
            if let Some([start, _, original_start, _, kind, features]) = span {
                assert_eq!(
                    (kind, original_start + copy - start, features),
                    (0, original, setup_features)
                );
                assert!(
                    echoes
                        .iter()
                        .all(|echo| { copy + authored.len() <= echo.start || echo.end <= copy })
                );
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

    #[test]
    fn quoted_and_template_commas_keep_one_reporting_copy() {
        assert_one_reporting_copy(
            "T extends 'a,b' = MissingQuotedDefault, U",
            "MissingQuotedDefault",
        );
        assert_one_reporting_copy(
            "T extends `a,${MissingTemplate}`, U = 'c,d'",
            "MissingTemplate",
        );
    }

    #[test]
    fn nested_functions_and_comments_keep_one_reporting_copy() {
        assert_one_reporting_copy(
            "T extends ((value: 'a,b', fn: <U>(value: U) => U) => MissingNested), U",
            "MissingNested",
        );
        assert_one_reporting_copy(
            "T /* name, <=> */ extends /* constraint, */ MissingComment, /* separator, */ U = 'a,b'",
            "MissingComment",
        );
        assert_one_reporting_copy("T extends MissingLine // comma, <=>", "MissingLine");
    }

    #[test]
    fn unicode_and_lf_crlf_keep_exact_authored_offsets() {
        for newline in ["\n", "\r\n"] {
            let generic = vize_l0::cstr!(
                "名前 extends 'a,b',{newline}  const T extends{newline}  MissingUnicode"
            );
            assert_one_reporting_copy(&generic, "MissingUnicode");
        }
    }

    /// `raw` anchored at SFC offset `origin`, with the rows the generator
    /// writes for `function __setup<raw>`.
    fn anchored(raw: &str, origin: usize, code: &str) -> (GenericParameters, Vec<VizeMapping>) {
        let mut parameters = GenericParameters::parse(raw);
        parameters.origin = Some(origin);
        let list = code.find("function __setup<").expect("setup signature") + 17;
        let rows = parameters.authored_rows(list).collect();
        (parameters, rows)
    }

    #[test]
    fn prepared_copies_keep_nested_names_inside_their_outer_owner() {
        let raw = "T extends Box<T>, U extends External = 'a,b'";
        let parameters = GenericParameters::parse(raw);
        assert!(parameters.diagnostics.is_empty());
        let code = vize_l0::cstr!(
            "function __setup<{}>() {{}}\ntype Props<{}> = {{}};\ntype Named = Props<{}>;\ntype Fallback = Props<{}>;\n",
            parameters.raw(),
            parameters.alias_declaration,
            parameters.names,
            parameters.fallback,
        );
        let (parameters, mappings) = anchored(raw, 100, &code);
        let attribute = parameters.attribute().expect("anchored");
        assert_eq!(mappings.len(), 3);
        for mapping in &mappings {
            let authored =
                mapping.src_range.start - attribute.start..mapping.src_range.end - attribute.start;
            assert_eq!(
                code.get(mapping.gen_range.clone()),
                parameters.raw().get(authored),
            );
        }
        let copies = GenericCopies::find(&code, &mappings, &parameters).expect("prepared copies");
        assert_eq!(copies.attribute, attribute);
        let echoes: Vec<_> = copies.echoes().cloned().collect();
        assert_eq!(echoes.len(), 2, "{code}");
        let named = code.find("type Named = ").expect("named argument control");
        let fallback = code.find("type Fallback = ").expect("fallback copy");
        assert!(
            echoes
                .iter()
                .all(|echo| echo.end <= named || fallback <= echo.start)
        );
        assert!(echoes.iter().all(|echo| {
            mappings
                .iter()
                .all(|row| row.gen_range.end <= echo.start || echo.end <= row.gen_range.start)
        }));
        // Without the generator's rows the setup signature is just another copy.
        let unmapped = GenericCopies::find(&code, &[], &parameters).expect("unmapped copies");
        assert_eq!(unmapped.echoes().count(), 3);
        let mut shifted = copies;
        shifted.shift_generated(17);
        assert_eq!(
            shifted.echoes().cloned().collect::<Vec<_>>(),
            echoes
                .iter()
                .map(|range| range.start + 17..range.end + 17)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn invalid_parameters_retain_the_authored_reporting_range() {
        let raw = "T extends";
        let code = vize_l0::cstr!("function __setup<{raw}>() {{}}");
        let (parameters, mappings) = anchored(raw, 40, &code);
        assert!(!parameters.diagnostics.is_empty());
        assert_eq!(
            mappings
                .iter()
                .map(|mapping| &mapping.src_range)
                .collect::<Vec<_>>(),
            vec![&(40..40 + raw.len())]
        );
        assert!(GenericCopies::find(&code, &mappings, &parameters).is_none());
    }

    #[test]
    fn unanchored_plans_have_no_rows_or_copies() {
        let parameters = GenericParameters::parse("T extends string");
        let code = vize_l0::cstr!("function __setup<{}>() {{}}", parameters.raw());
        assert_eq!(parameters.authored_rows(17).count(), 0);
        assert!(GenericCopies::find(&code, &[], &parameters).is_none());
    }

    #[test]
    fn unrelated_bare_names_and_unconstrained_fallbacks_are_not_echoes() {
        for generic in ["T extends 'a,b'", "T = 'a,b'"] {
            let code = vize_l0::cstr!(
                "function __setup<{generic}>() {{}}\ntype UnrelatedName = Other<T>;\ntype UnrelatedFallback = Other<unknown>;\n",
            );
            let (parameters, mappings) = anchored(generic, 0, &code);
            assert!(parameters.diagnostics.is_empty());
            assert!(!mappings.is_empty());
            assert!(
                GenericCopies::find(&code, &mappings, &parameters).is_none(),
                "{code}"
            );
        }
    }
}
