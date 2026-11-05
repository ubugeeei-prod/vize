//! Rewrites of generated content-mapper text.

use std::ops::Range;

use vize_l0::String as CompactString;

use crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector;
use crate::virtual_ts::{VizeMapping, VizeSemanticLink};

use super::super::vue_codegen::generic_attribute::GenericCopies;

const PRIVATE_MODULE: &str = "vize:virtual-ts";
const PUBLISHED_MODULE: &str = "vize/virtual-ts";
const TYPESCRIPT_NOCHECK_DIRECTIVE: &str = "// @ts-nocheck\n";
const REFERENCE_TYPES_DIRECTIVE: &str = "/// <reference types=";
const PUBLIC_DIAGNOSTIC_HELPERS: [(&str, &str); 2] = [
    ("$ComponentCheckProps", "ComponentPropsForTemplate"),
    ("$PublicComponentAttrs", "VueTemplatePublicAttrsType"),
];

pub(super) fn publish_content_mapper_text(
    code: &mut CompactString,
    mappings: &mut [VizeMapping],
    semantic_links: &mut [VizeSemanticLink],
    prop_default_key_links: &mut [VizeSemanticLink],
    generic_copies: Option<&mut GenericCopies>,
    diagnostic_ownership: Option<&DiagnosticOwnershipCollector>,
    share_virtual_module: bool,
) {
    publicize_generated_diagnostic_helpers(
        code,
        mappings,
        semantic_links,
        prop_default_key_links,
        generic_copies,
        diagnostic_ownership,
    );
    if !share_virtual_module {
        return;
    }
    debug_assert_eq!(PRIVATE_MODULE.len(), PUBLISHED_MODULE.len());
    let starts = memchr::memmem::find_iter(code.as_bytes(), PRIVATE_MODULE).collect::<Vec<_>>();
    for start in starts.into_iter().rev() {
        let end = start + PRIVATE_MODULE.len();
        code.replace_range(start..end, PUBLISHED_MODULE);
    }
}

pub(super) fn prepend_typescript_nocheck(
    code: &mut CompactString,
    mappings: &mut [VizeMapping],
    semantic_links: &mut [VizeSemanticLink],
    prop_default_key_links: &mut [VizeSemanticLink],
) {
    let offset = TYPESCRIPT_NOCHECK_DIRECTIVE.len();
    let mut prefixed = CompactString::with_capacity(offset + code.len());
    prefixed.push_str(TYPESCRIPT_NOCHECK_DIRECTIVE);
    prefixed.push_str(code.as_str());
    *code = prefixed;
    for mapping in mappings {
        mapping.gen_range.start += offset;
        mapping.gen_range.end += offset;
        for sub_span in &mut mapping.sub_spans {
            sub_span.gen_range.start += offset;
            sub_span.gen_range.end += offset;
        }
    }
    for link in semantic_links.iter_mut().chain(prop_default_key_links) {
        link.shift_generated_ranges(offset);
    }
}

/// Give generated component-input helpers public display names before the
/// upstream host parses the virtual module.
///
/// `$ComponentCheckProps` and `$PublicComponentAttrs` are shorter than the
/// display names. Each replacement records that growth so later generated
/// coordinates stay on the same text. Script `/// <reference types>` values
/// are copied into the generated prefix without mappings. Treat those complete
/// lines as authored ranges as well, so a package that happens to use an
/// internal helper name is never rewritten.
fn publicize_generated_diagnostic_helpers(
    code: &mut CompactString,
    mappings: &mut [VizeMapping],
    semantic_links: &mut [VizeSemanticLink],
    prop_default_key_links: &mut [VizeSemanticLink],
    mut generic_copies: Option<&mut GenericCopies>,
    diagnostic_ownership: Option<&DiagnosticOwnershipCollector>,
) {
    if PUBLIC_DIAGNOSTIC_HELPERS
        .iter()
        .all(|(internal, _)| memchr::memmem::find(code.as_bytes(), internal.as_bytes()).is_none())
    {
        return;
    }
    let mut authored_ranges = mappings
        .iter()
        .map(|mapping| mapping.gen_range.clone())
        .collect::<Vec<_>>();
    authored_ranges.extend(reference_type_directive_ranges(code));
    authored_ranges.sort_unstable_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for range in authored_ranges {
        if let Some(last) = merged.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
        } else {
            merged.push(range);
        }
    }

    for (internal, display) in PUBLIC_DIAGNOSTIC_HELPERS {
        debug_assert!(display.len() > internal.len());
        let occurrences = memchr::memmem::find_iter(code.as_bytes(), internal).collect::<Vec<_>>();
        for start in occurrences.into_iter().rev() {
            let old_len = internal.len();
            let end = start + old_len;
            let candidate = start..end;
            let next = merged.partition_point(|range| range.end <= candidate.start);
            let authored = merged
                .get(next)
                .is_some_and(|range| candidate.start < range.end && range.start < candidate.end);
            if authored {
                continue;
            }
            let new_len = display.len();
            code.replace_range(candidate, display);
            note_public_name_growth(
                mappings,
                &mut merged,
                semantic_links,
                prop_default_key_links,
                generic_copies.as_deref_mut(),
                diagnostic_ownership,
                start,
                old_len,
                new_len,
            );
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "all generated coordinate families must shift in one atomic operation"
)]
fn note_public_name_growth(
    mappings: &mut [VizeMapping],
    authored_ranges: &mut [Range<usize>],
    semantic_links: &mut [VizeSemanticLink],
    prop_default_key_links: &mut [VizeSemanticLink],
    generic_copies: Option<&mut GenericCopies>,
    diagnostic_ownership: Option<&DiagnosticOwnershipCollector>,
    start: usize,
    old_len: usize,
    new_len: usize,
) {
    let old_end = start + old_len;
    let extra = new_len - old_len;
    for mapping in mappings {
        shift_generated_range(&mut mapping.gen_range, start, old_end, extra);
        for sub_span in &mut mapping.sub_spans {
            shift_generated_range(&mut sub_span.gen_range, start, old_end, extra);
        }
    }
    for range in authored_ranges {
        shift_generated_range(range, start, old_end, extra);
    }
    for link in semantic_links.iter_mut().chain(prop_default_key_links) {
        shift_generated_range(&mut link.source_range, start, old_end, extra);
        shift_generated_range(&mut link.target_range, start, old_end, extra);
    }
    if let Some(copies) = generic_copies {
        copies.note_generated_replacement(start, old_len, new_len);
    }
    if let Some(collector) = diagnostic_ownership {
        collector
            .borrow_mut()
            .note_generated_replacement(start, old_len, new_len);
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

/// Complete lines whose first non-whitespace text is a types reference.
fn reference_type_directive_ranges(code: &str) -> Vec<Range<usize>> {
    let bytes = code.as_bytes();
    memchr::memmem::find_iter(bytes, REFERENCE_TYPES_DIRECTIVE)
        .filter_map(|at| {
            let start = memchr::memrchr(b'\n', bytes.get(..at)?).map_or(0, |newline| newline + 1);
            code.get(start..at)?
                .chars()
                .all(char::is_whitespace)
                .then(|| {
                    let end = bytes
                        .get(at..)
                        .and_then(|rest| memchr::memchr(b'\n', rest))
                        .map_or(code.len(), |newline| at + newline + 1);
                    start..end
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::{
        CompactString, GenericCopies, PRIVATE_MODULE, PUBLIC_DIAGNOSTIC_HELPERS, PUBLISHED_MODULE,
        publicize_generated_diagnostic_helpers, publish_content_mapper_text,
        reference_type_directive_ranges,
    };
    use crate::virtual_ts::diagnostic_ownership::{
        DiagnosticOwnership, DiagnosticOwnershipCollector,
    };
    use crate::virtual_ts::generic_parameters::GenericParameters;
    use crate::virtual_ts::{VizeMapping, VizeSemanticLink, VizeSemanticLinkKind, VizeSubSpan};

    type State = (
        CompactString,
        Vec<VizeMapping>,
        Vec<VizeSemanticLink>,
        Vec<VizeSemanticLink>,
        Option<(Range<usize>, Vec<Range<usize>>)>,
        CompactString,
    );

    fn state(
        code: &str,
        mappings: &[VizeMapping],
        links: &[VizeSemanticLink],
        prop_links: &[VizeSemanticLink],
        copies: Option<&GenericCopies>,
        ownership: &DiagnosticOwnershipCollector,
    ) -> State {
        (
            code.into(),
            mappings.to_vec(),
            links.to_vec(),
            prop_links.to_vec(),
            copies.map(|copies| (copies.attribute.clone(), copies.echoes().cloned().collect())),
            vize_l0::cstr!("{:?}", ownership.borrow()),
        )
    }

    fn collector(range: Range<usize>) -> DiagnosticOwnershipCollector {
        let mut ownership = DiagnosticOwnership::default();
        ownership.record_diagnostic_primary(range.clone(), 30..35);
        ownership.protect_callback_primary(range);
        ownership.into()
    }

    #[test]
    fn absent_public_helpers_preserve_every_coordinate_family_and_generic_copy() {
        let raw = "T extends Missing";
        let mut code = vize_l0::cstr!(
            "/// <reference types=\"日本 🧭\" />\r\nimport type {{}} from \"vize:virtual-ts\";\r\nfunction __setup<{raw}>() {{}}\r\ntype Echo<{raw}> = T;\r\n"
        );
        let mut parameters = GenericParameters::parse(raw);
        parameters.origin = Some(100);
        let start = code.find(raw).expect("authored generic");
        let echo = code.rfind(raw).expect("generic echo");
        let mut mappings: Vec<_> = parameters.authored_rows(start).collect();
        mappings
            .first_mut()
            .expect("authored generic row")
            .sub_spans
            .push(VizeSubSpan {
                gen_range: start..start + 1,
                src_range: 100..101,
            });
        let mut copies = GenericCopies::find(&code, &mappings, &parameters).expect("generic copy");
        let mut links = vec![VizeSemanticLink {
            source_range: start..start + raw.len(),
            target_range: echo..echo + raw.len(),
            kind: VizeSemanticLinkKind::VueSetupImportSpecialization,
        }];
        let mut prop_links = links.clone();
        let ownership = collector(echo..echo + raw.len());
        let before = state(
            &code,
            &mappings,
            &links,
            &prop_links,
            Some(&copies),
            &ownership,
        );
        publicize_generated_diagnostic_helpers(
            &mut code,
            &mut mappings,
            &mut links,
            &mut prop_links,
            Some(&mut copies),
            Some(&ownership),
        );
        assert_eq!(
            state(
                &code,
                &mappings,
                &links,
                &prop_links,
                Some(&copies),
                &ownership
            ),
            before
        );
        let mut expected = before;
        expected.0 = code.replace(PRIVATE_MODULE, PUBLISHED_MODULE).into();
        publish_content_mapper_text(
            &mut code,
            &mut mappings,
            &mut links,
            &mut prop_links,
            Some(&mut copies),
            Some(&ownership),
            true,
        );
        assert_eq!(
            state(
                &code,
                &mappings,
                &links,
                &prop_links,
                Some(&copies),
                &ownership
            ),
            expected
        );
    }

    #[test]
    fn generated_helper_renames_keep_mapped_user_names_and_shift_all_endpoints() {
        for (internal, display) in PUBLIC_DIAGNOSTIC_HELPERS {
            let user = vize_l0::cstr!("const {internal} = 1;\n");
            let mut code = vize_l0::cstr!("type {internal}<T> = T;\n{user}let tail = 2;\n");
            let expected = vize_l0::cstr!("type {display}<T> = T;\n{user}let tail = 2;\n");
            let user_at = code.find(user.as_str()).expect("mapped user line");
            let expected_user = expected.find(user.as_str()).expect("unchanged user line");
            let tail = code.find("tail").expect("following endpoint");
            let expected_tail = expected.find("tail").expect("shifted endpoint");
            let row = |at| VizeMapping {
                gen_range: at..at + user.len(),
                src_range: 100..100 + user.len(),
                sub_spans: vec![VizeSubSpan {
                    gen_range: at + 6..at + 6 + internal.len(),
                    src_range: 106..106 + internal.len(),
                }],
            };
            let link = |at, target| VizeSemanticLink {
                source_range: at + 6..at + 6 + internal.len(),
                target_range: target..target + 4,
                kind: VizeSemanticLinkKind::VueTemplatePropBinding,
            };
            let mut mappings = vec![row(user_at)];
            let mut links = vec![link(user_at, tail)];
            let mut prop_links = links.clone();
            let ownership = collector(tail..tail + 4);
            let expected_ownership = collector(expected_tail..expected_tail + 4);
            let expected_state = state(
                &expected,
                &[row(expected_user)],
                &[link(expected_user, expected_tail)],
                &[link(expected_user, expected_tail)],
                None,
                &expected_ownership,
            );
            publicize_generated_diagnostic_helpers(
                &mut code,
                &mut mappings,
                &mut links,
                &mut prop_links,
                None,
                Some(&ownership),
            );
            assert_eq!(
                state(&code, &mappings, &links, &prop_links, None, &ownership),
                expected_state
            );
        }
    }

    #[test]
    fn authored_only_helper_names_and_reference_packages_remain_wholly_unchanged() {
        let mut code = CompactString::from(
            "/// <reference types=\"$PublicComponentAttrs\" />\r\nconst $ComponentCheckProps = 1;\r\n",
        );
        let start = code.find("const ").expect("authored declaration");
        let mut mappings = vec![VizeMapping::new(
            start..code.len(),
            100..100 + code.len() - start,
        )];
        let mut links = Vec::new();
        let mut prop_links = Vec::new();
        let ownership = collector(start..code.len());
        let before = state(&code, &mappings, &links, &prop_links, None, &ownership);
        publicize_generated_diagnostic_helpers(
            &mut code,
            &mut mappings,
            &mut links,
            &mut prop_links,
            None,
            Some(&ownership),
        );
        assert_eq!(
            state(&code, &mappings, &links, &prop_links, None, &ownership),
            before
        );
    }

    #[test]
    fn reference_directive_lines_match_a_line_scan() {
        let code = "/// <reference types=\"a\" />\n  /// <reference types=\"b\" />\r\nx /// <reference types=\"c\" />\n\u{3000}/// <reference types=\"d\" /> /// <reference types=\"e\" />\n/// <reference path=\"f\" />\n\t/// <reference types=\"g\" />";
        let mut start = 0;
        let expected = code
            .split_inclusive('\n')
            .filter_map(|line| {
                let range = start..start + line.len();
                start = range.end;
                line.trim_start()
                    .starts_with("/// <reference types=")
                    .then_some(range)
            })
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), 4);
        assert_eq!(reference_type_directive_ranges(code), expected);
    }
}
