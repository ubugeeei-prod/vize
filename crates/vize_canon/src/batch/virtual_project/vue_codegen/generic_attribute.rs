//! Map `<script setup generic="...">` through its generated copies.
//!
//! One parser-owned parameter plan supplies the exact rendered spellings and
//! relative mapping rows. The setup signature owns authored diagnostics; the
//! other generated copies are echoes of that same authored list.

use std::ops::Range;

use crate::virtual_ts::generic_parameters::GenericParameters;
use crate::virtual_ts::{ProjectionFeatures, ProjectionMeta, VizeMapping};

/// Generated copies of one authored parameter list. Preparation happens before
/// mapping the reporting copy, so both mapping and suppression share ownership.
pub(in crate::batch::virtual_project) struct GenericCopies {
    pub(in crate::batch::virtual_project) attribute: Range<usize>,
    copies: Vec<GenericCopy>,
}

struct GenericCopy {
    range: Range<usize>,
    rows: Option<Vec<Row>>,
    reporting: bool,
}

type Row = (Range<usize>, Range<usize>);

impl GenericCopies {
    pub(in crate::batch::virtual_project) fn find(
        code: &str,
        mappings: &[VizeMapping],
        parameters: &GenericParameters,
        attribute: Range<usize>,
    ) -> Option<Self> {
        let mut covered: Vec<_> = mappings.iter().map(|row| row.gen_range.clone()).collect();
        covered.sort_by_key(|range| range.start);
        let mut copies = Vec::new();
        // Match complete, already-rendered lists. Quoted commas, nested types,
        // arrows and comments remain opaque here; OXC prepared their ranges.
        for (rendered, rows) in parameters.declaration_variants() {
            for list in exact_lists(code, rendered) {
                if overlaps_mapped(&covered, list.clone()) {
                    continue;
                }
                let start = list.start + 1;
                copies.push(GenericCopy {
                    range: instantiation(code, list.start)..list.end,
                    rows: Some(
                        rows.iter()
                            .map(|(generated, authored)| {
                                (
                                    start + generated.start..start + generated.end,
                                    attribute.start + authored.start
                                        ..attribute.start + authored.end,
                                )
                            })
                            .collect(),
                    ),
                    reporting: false,
                });
            }
        }
        // Only a repeated authored constraint can carry an echoed diagnostic.
        // Bare names or unconstrained `unknown`/`any` arguments can occur in
        // unrelated generated helpers and do not establish that ownership.
        if parameters.has_repeated_fallback_constraint() {
            for list in exact_lists(code, &parameters.fallback) {
                if overlaps_mapped(&covered, list.clone()) {
                    continue;
                }
                copies.push(GenericCopy {
                    range: instantiation(code, list.start)..list.end,
                    rows: None,
                    reporting: false,
                });
            }
        }
        // Prefer declaration rows when spellings coincide. Outer lists own any
        // matching list nested inside their constraints/defaults, preventing an
        // inner `Foo<T>` from suppressing the reporting signature around it.
        copies.sort_by(|left, right| {
            left.range
                .start
                .cmp(&right.range.start)
                .then_with(|| right.range.end.cmp(&left.range.end))
                .then_with(|| right.rows.is_some().cmp(&left.rows.is_some()))
        });
        let mut retained: Vec<GenericCopy> = Vec::new();
        for mut copy in copies {
            if retained
                .last()
                .is_some_and(|last| copy.range.start < last.range.end)
            {
                continue;
            }
            copy.reporting = copy.rows.is_some() && is_setup_copy(code, &copy.range);
            retained.push(copy);
        }
        (!retained.is_empty()).then_some(Self {
            attribute,
            copies: retained,
        })
    }

    /// Generated ranges whose diagnostics repeat the setup signature's.
    pub(in crate::batch::virtual_project) fn echoes(&self) -> impl Iterator<Item = &Range<usize>> {
        self.copies
            .iter()
            .filter(|copy| !copy.reporting)
            .map(|copy| &copy.range)
    }

    pub(in crate::batch::virtual_project) fn shift_generated(&mut self, offset: usize) {
        for copy in &mut self.copies {
            copy.range.start += offset;
            copy.range.end += offset;
            for (generated, _) in copy.rows.iter_mut().flatten() {
                generated.start += offset;
                generated.end += offset;
            }
        }
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

fn is_setup_copy(code: &str, copy: &Range<usize>) -> bool {
    code.get(..copy.start)
        .is_some_and(|before| before.ends_with("function "))
        && code
            .get(copy.start..)
            .is_some_and(|after| after.starts_with("__setup<"))
}

/// Map the setup signature copy back to the attribute, diagnostics only.
pub(super) fn map_generic_attribute(
    code: &str,
    mappings: &mut Vec<VizeMapping>,
    meta: &mut Vec<ProjectionMeta>,
    copies: Option<&GenericCopies>,
) {
    let Some(rows) = copies.and_then(|copies| {
        copies.copies.iter().find_map(|copy| {
            (copy.reporting && is_setup_copy(code, &copy.range))
                .then_some(copy.rows.as_ref())
                .flatten()
        })
    }) else {
        return;
    };
    meta.resize(mappings.len(), ProjectionMeta::default());
    for (gen_range, src_range) in rows {
        mappings.push(VizeMapping {
            gen_range: gen_range.clone(),
            src_range: src_range.clone(),
            sub_spans: Vec::new(),
        });
        meta.push(ProjectionMeta {
            features: ProjectionFeatures::DIAGNOSTICS,
            ..ProjectionMeta::default()
        });
    }
}

#[cfg(test)]
pub(crate) use crate::virtual_ts::generic_parameters::attribute_value;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::batch::generate_vue_content_mapper_transform;
    use crate::virtual_ts::generic_parameters::GenericParameters;

    use super::{GenericCopies, map_generic_attribute};

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

    #[test]
    fn prepared_copies_keep_nested_names_inside_their_outer_owner() {
        let parameters = GenericParameters::parse("T extends Box<T>, U extends External = 'a,b'");
        assert!(parameters.diagnostics.is_empty());
        let code = vize_l0::cstr!(
            "function __setup<{}>() {{}}\ntype Props<{}> = {{}};\ntype Named = Props<{}>;\ntype Fallback = Props<{}>;\n",
            parameters.raw(),
            parameters.alias_declaration,
            parameters.names,
            parameters.fallback,
        );
        let attribute = 100..100 + parameters.raw().len();
        let copies = GenericCopies::find(&code, &[], &parameters, attribute.clone())
            .expect("prepared copies");
        let echoes: Vec<_> = copies.echoes().cloned().collect();
        assert_eq!(echoes.len(), 2, "{code}");
        let named = code.find("type Named = ").expect("named argument control");
        let fallback = code.find("type Fallback = ").expect("fallback copy");
        assert!(
            echoes
                .iter()
                .all(|echo| echo.end <= named || fallback <= echo.start)
        );
        let mut mappings = Vec::new();
        let mut meta = Vec::new();
        map_generic_attribute(&code, &mut mappings, &mut meta, Some(&copies));
        assert_eq!(mappings.len(), 3);
        assert_eq!(meta.len(), mappings.len());
        for mapping in &mappings {
            let authored =
                mapping.src_range.start - attribute.start..mapping.src_range.end - attribute.start;
            assert_eq!(
                code.get(mapping.gen_range.clone()),
                parameters.raw().get(authored),
            );
            assert!(echoes.iter().all(|echo| {
                mapping.gen_range.end <= echo.start || echo.end <= mapping.gen_range.start
            }));
        }
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
        let parameters = GenericParameters::parse("T extends");
        assert!(!parameters.diagnostics.is_empty());
        let code = vize_l0::cstr!("function __setup<{}>() {{}}", parameters.raw());
        let attribute = 40..40 + parameters.raw().len();
        let copies = GenericCopies::find(&code, &[], &parameters, attribute.clone())
            .expect("invalid authored signature");
        let mut mappings = Vec::new();
        let mut meta = Vec::new();
        map_generic_attribute(&code, &mut mappings, &mut meta, Some(&copies));
        assert_eq!(mappings.len(), 1);
        assert_eq!(
            mappings.first().map(|mapping| &mapping.src_range),
            Some(&attribute)
        );
        assert!(copies.echoes().next().is_none());
    }

    #[test]
    fn unrelated_bare_names_and_unconstrained_fallbacks_are_not_echoes() {
        for generic in ["T extends 'a,b'", "T = 'a,b'"] {
            let parameters = GenericParameters::parse(generic);
            assert!(parameters.diagnostics.is_empty());
            let code = vize_l0::cstr!(
                "function __setup<{}>() {{}}\ntype UnrelatedName = Other<T>;\ntype UnrelatedFallback = Other<unknown>;\n",
                parameters.raw(),
            );
            let copies = GenericCopies::find(&code, &[], &parameters, 0..generic.len())
                .expect("reporting signature");
            assert!(copies.echoes().next().is_none(), "{code}");
        }
    }
}
