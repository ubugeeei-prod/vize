//! One namespace import for the shared SDK's type references.
//!
//! Every `import("vize/virtual-ts").X` type node makes TypeScript resolve the
//! module specifier and look its export up again. A file that names the SDK
//! often enough imports it once with `import type * as __vizeSdk` instead, and
//! each reference becomes `__vizeSdk.X`: a qualified name of the same
//! declaration, so the same types and diagnostics, for one module resolution
//! per file. Generated `__vize*` names are reserved, so authored code cannot
//! capture the namespace.
//!
//! The file keeps one module reference per rewritten type reference, as
//! empty type-only imports after the namespace import. tsgo assigns files to
//! checkers by a weight that counts a file's module references
//! (`file.Imports()`, `compiler/checkerpool.go`) and links files by them; the
//! SDK references track a file's template contract work, its expensive part.
//! Collapsing them to one import made template-heavy files look cheap and
//! cluster on one checker (#13135: misskey +11.9 % at the default checker
//! count). An empty import is checked once, without resolving a qualifier or
//! instantiating anything, and leaves the partition where it was (#13225).

use std::ops::Range;

use vize_l0::String as CompactString;

use crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector;
use crate::virtual_ts::{VizeMapping, VizeSemanticLink};

use super::GenericCopies;

const PRIVATE_REFERENCE: &str = "import(\"vize:virtual-ts\").";
const NAMESPACE_REFERENCE: &str = "__vizeSdk.";
const NAMESPACE_IMPORT: &str = "import type * as __vizeSdk from \"vize/virtual-ts\";\n";
/// One per rewritten reference beyond the namespace import's own.
const WEIGHT_IMPORT: &str = "import type {} from \"vize/virtual-ts\";\n";
const SAVED: usize = PRIVATE_REFERENCE.len() - NAMESPACE_REFERENCE.len();
/// Fewer references than this keep their import types: the namespace import
/// itself is a statement to check.
const MIN_REFERENCES: usize = 4;

/// Rewrite the SDK references through one namespace import when that shrinks
/// the file. References inside verbatim rows, sub-spans or `/// <reference>`
/// lines, and any reference a generated coordinate points into, keep their
/// import type. Every coordinate family moves with the shrunk text.
pub(super) fn name_sdk_through_namespace(
    code: &mut CompactString,
    mappings: &mut [VizeMapping],
    semantic_links: &mut [VizeSemanticLink],
    prop_default_key_links: &mut [VizeSemanticLink],
    generic_copies: Option<&mut GenericCopies>,
    diagnostic_ownership: Option<&DiagnosticOwnershipCollector>,
    authored_lines: &[Range<usize>],
) {
    let found = memchr::memmem::find_iter(code.as_bytes(), PRIVATE_REFERENCE).collect::<Vec<_>>();
    if found.len() < MIN_REFERENCES {
        return;
    }
    let mut kept = authored_lines.to_vec();
    let mut boundaries = Vec::new();
    for mapping in mappings.iter() {
        if mapping.gen_range.len() == mapping.src_range.len() {
            kept.push(mapping.gen_range.clone());
        }
        boundaries.extend([mapping.gen_range.start, mapping.gen_range.end]);
        for sub_span in &mapping.sub_spans {
            kept.push(sub_span.gen_range.clone());
            boundaries.extend([sub_span.gen_range.start, sub_span.gen_range.end]);
        }
    }
    for link in semantic_links.iter().chain(prop_default_key_links.iter()) {
        for range in [&link.source_range, &link.target_range] {
            boundaries.extend([range.start, range.end]);
        }
    }
    if let Some(copies) = generic_copies.as_deref() {
        for echo in copies.echoes() {
            boundaries.extend([echo.start, echo.end]);
        }
    }
    if let Some(collector) = diagnostic_ownership {
        for range in collector.borrow().generated_ranges() {
            boundaries.extend([range.start, range.end]);
        }
    }
    boundaries.sort_unstable();
    let starts = found
        .into_iter()
        .filter(|&start| {
            let end = start + PRIVATE_REFERENCE.len();
            let first_after = boundaries.partition_point(|&at| at <= start);
            let pointed_into = boundaries.get(first_after).is_some_and(|&at| at < end);
            !pointed_into
                && !kept
                    .iter()
                    .any(|range| range.start < end && start < range.end)
        })
        .collect::<Vec<_>>();
    let separator = !code.is_empty() && !code.ends_with('\n');
    if starts.len() < MIN_REFERENCES {
        return;
    }
    let Some(rewritten) = rewrite(code, &starts, separator) else {
        return;
    };
    *code = rewritten;

    let remap = |at: usize| {
        at - SAVED * starts.partition_point(|&start| start + PRIVATE_REFERENCE.len() <= at)
    };
    let remap_range = |range: &mut Range<usize>| {
        *range = remap(range.start)..remap(range.end);
    };
    for mapping in mappings.iter_mut() {
        remap_range(&mut mapping.gen_range);
        for sub_span in &mut mapping.sub_spans {
            remap_range(&mut sub_span.gen_range);
        }
    }
    for link in semantic_links
        .iter_mut()
        .chain(prop_default_key_links.iter_mut())
    {
        remap_range(&mut link.source_range);
        remap_range(&mut link.target_range);
    }
    if let Some(copies) = generic_copies {
        copies.remap_generated(remap);
    }
    if let Some(collector) = diagnostic_ownership {
        // No row boundary lies inside a rewritten reference, so every row
        // keeps its text and its proof.
        collector
            .borrow_mut()
            .note_generated_rewrite(|range| (remap(range.start)..remap(range.end), true));
    }
}

fn rewrite(code: &str, starts: &[usize], separator: bool) -> Option<CompactString> {
    let weights = starts.len().saturating_sub(1);
    let mut out = CompactString::with_capacity(
        code.len() - starts.len() * SAVED
            + NAMESPACE_IMPORT.len()
            + weights * WEIGHT_IMPORT.len()
            + usize::from(separator),
    );
    let mut last = 0;
    for &start in starts {
        out.push_str(code.get(last..start)?);
        out.push_str(NAMESPACE_REFERENCE);
        last = start + PRIVATE_REFERENCE.len();
    }
    out.push_str(code.get(last..)?);
    if separator {
        out.push('\n');
    }
    out.push_str(NAMESPACE_IMPORT);
    for _ in 0..weights {
        out.push_str(WEIGHT_IMPORT);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{CompactString, NAMESPACE_IMPORT, WEIGHT_IMPORT, name_sdk_through_namespace};
    use crate::virtual_ts::diagnostic_ownership::{
        DiagnosticOwnership, DiagnosticOwnershipCollector,
    };
    use crate::virtual_ts::{VizeMapping, VizeSemanticLink, VizeSemanticLinkKind, VizeSubSpan};

    const REF: &str = "import(\"vize:virtual-ts\").";

    fn mapping(
        gen_range: std::ops::Range<usize>,
        src_range: std::ops::Range<usize>,
    ) -> VizeMapping {
        VizeMapping {
            gen_range,
            src_range,
            sub_spans: Vec::new(),
        }
    }

    fn aliases(count: usize) -> String {
        (0..count)
            .map(|index| format!("type A{index} = {REF}$UsageProps<typeof C>;\n"))
            .collect()
    }

    #[test]
    fn few_references_keep_their_import_types() {
        let source = aliases(3);
        let mut code = CompactString::from(source.as_str());
        let mut mappings = vec![mapping(5..7, 0..2)];
        name_sdk_through_namespace(&mut code, &mut mappings, &mut [], &mut [], None, None, &[]);
        assert_eq!(code.as_str(), source.as_str());
        assert_eq!(mappings, vec![mapping(5..7, 0..2)]);
    }

    #[test]
    fn many_references_name_one_namespace_and_move_every_coordinate() {
        let source = format!("const x = 1;\n{}const y = x;", aliases(5));
        let mut code = CompactString::from(source.as_str());
        let x = source.find("x =").unwrap();
        let y = source.rfind("y =").unwrap();
        let second = source.match_indices("type A1").next().unwrap().0;
        let second_end = second + source[second..].find('\n').unwrap() + 1;
        let mut mappings = vec![
            mapping(x..x + 1, 6..7),
            // A generated statement collapsed onto its tag shrinks with it.
            mapping(second..second_end, 40..47),
            mapping(y..y + 1, 90..91),
        ];
        let mut links = vec![VizeSemanticLink {
            source_range: x..x + 1,
            target_range: y..y + 1,
            kind: VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
        }];
        let ownership: DiagnosticOwnershipCollector = DiagnosticOwnership::default().into();
        ownership.borrow_mut().protect_callback_primary(y..y + 1);
        name_sdk_through_namespace(
            &mut code,
            &mut mappings,
            &mut links,
            &mut [],
            None,
            Some(&ownership),
            &[],
        );
        let expected = format!(
            "const x = 1;\n{}const y = x;\n{NAMESPACE_IMPORT}{}",
            aliases(5).replace(REF, "__vizeSdk."),
            WEIGHT_IMPORT.repeat(4)
        );
        assert_eq!(code.as_str(), expected.as_str());
        let new_y = expected.rfind("y =").unwrap();
        let new_second = expected.match_indices("type A1").next().unwrap().0;
        let new_second_end = new_second + expected[new_second..].find('\n').unwrap() + 1;
        assert_eq!(
            mappings,
            vec![
                mapping(x..x + 1, 6..7),
                mapping(new_second..new_second_end, 40..47),
                mapping(new_y..new_y + 1, 90..91),
            ]
        );
        assert_eq!(links[0].source_range, x..x + 1);
        assert_eq!(links[0].target_range, new_y..new_y + 1);
        assert_eq!(
            ownership.borrow_mut().take_callback_primaries(),
            vec![new_y..new_y + 1]
        );
    }

    #[test]
    fn verbatim_rows_and_pointed_references_keep_their_text() {
        let source = aliases(8);
        let mut code = CompactString::from(source.as_str());
        let starts = source
            .match_indices(REF)
            .map(|(at, _)| at)
            .collect::<Vec<_>>();
        // A byte-for-byte row over the first reference, a sub-span over the
        // second and a link boundary inside the third keep those three.
        let first = starts[0];
        let second = starts[1];
        let third = starts[2];
        let mut sub_spanned = mapping(second - 2..second + REF.len() + 4, 10..12);
        sub_spanned.sub_spans.push(VizeSubSpan {
            gen_range: second..second + 3,
            src_range: 10..11,
        });
        let mut mappings = vec![mapping(first..first + REF.len(), 0..REF.len()), sub_spanned];
        let mut links = vec![VizeSemanticLink {
            source_range: third + 3..third + 30,
            target_range: 0..1,
            kind: VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
        }];
        name_sdk_through_namespace(
            &mut code,
            &mut mappings,
            &mut links,
            &mut [],
            None,
            None,
            &[],
        );
        let expected = (0..8)
            .map(|index| {
                let reference = if index < 3 { REF } else { "__vizeSdk." };
                format!("type A{index} = {reference}$UsageProps<typeof C>;\n")
            })
            .chain([NAMESPACE_IMPORT.to_owned(), WEIGHT_IMPORT.repeat(4)])
            .collect::<String>();
        assert_eq!(code.as_str(), expected.as_str());
        // Coordinates before the first rewrite do not move.
        assert_eq!(mappings[0].gen_range, first..first + REF.len());
        assert_eq!(links[0].source_range, third + 3..third + 30);
    }

    #[test]
    fn authored_reference_lines_are_never_rewritten() {
        let source = aliases(6);
        let mut code = CompactString::from(source.as_str());
        name_sdk_through_namespace(
            &mut code,
            &mut [],
            &mut [],
            &mut [],
            None,
            None,
            &[0..source.len()],
        );
        assert_eq!(code.as_str(), source.as_str());
    }
}
