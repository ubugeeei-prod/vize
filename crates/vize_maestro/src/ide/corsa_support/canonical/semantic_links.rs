use vize_canon::{LspPosition, LspRange};
use vize_carton::{String, cstr};

use super::{CanonicalVirtualDocument, location_matches_uri};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CanonicalSemanticPosition {
    pub(crate) request_uri: String,
    pub(crate) line: u32,
    pub(crate) character: u32,
}

/// Resolve the synthetic link that joins an authored setup binding to the
/// template-scope shadow used for Vue ref unwrapping.
///
/// TypeScript correctly keeps a template `v-for` local separate from the
/// setup binding, but the two generated declarations representing the setup
/// binding are intentionally connected through a type alias rather than the
/// same TS symbol. Following that generated edge lets a second semantic query
/// recover template references without a same-spelling source sweep.
pub(crate) fn linked_semantic_position(
    document: &CanonicalVirtualDocument,
    uri: &str,
    range: &LspRange,
) -> Option<CanonicalSemanticPosition> {
    let (request_uri, code) = virtual_code(document, uri)?;
    let start = crate::ide::position_to_offset(code, range.start.line, range.start.character)?;
    let end = crate::ide::position_to_offset(code, range.end.line, range.end.character)?;
    let linked_offset = linked_offset(code, start, end)?;
    let (line, character) = crate::ide::offset_to_position(code, linked_offset);
    Some(CanonicalSemanticPosition {
        request_uri: request_uri.clone(),
        line,
        character,
    })
}

fn linked_offset(code: &str, start: usize, end: usize) -> Option<usize> {
    let name = code.get(start..end)?;
    if name.is_empty() || name.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return None;
    }

    let anchor = cstr!("type __R_{name} = typeof {name};");
    let anchor_name_in_pattern = anchor.rfind(name)?;
    let anchors = code
        .match_indices(anchor.as_str())
        .map(|(offset, _)| offset + anchor_name_in_pattern)
        .collect::<Vec<_>>();
    let shadow = cstr!("var {name}: __U<__R_{name}> =");
    let shadows = code
        .match_indices(shadow.as_str())
        .map(|(offset, _)| offset + "var ".len())
        .collect::<Vec<_>>();

    let linked_offset = if anchors.contains(&start) {
        shadows.into_iter().filter(|offset| *offset > start).min()?
    } else if shadows.contains(&start) {
        anchors.into_iter().filter(|offset| *offset < start).max()?
    } else {
        return None;
    };
    Some(linked_offset)
}

fn virtual_code<'a>(
    document: &'a CanonicalVirtualDocument,
    uri: &str,
) -> Option<(&'a String, &'a str)> {
    if location_matches_uri(uri, document.request_uri.as_str()) {
        return Some((&document.request_uri, &document.virtual_result.code));
    }
    document
        .dependencies
        .iter()
        .find(|dependency| location_matches_uri(uri, dependency.request_uri.as_str()))
        .map(|dependency| {
            (
                &dependency.request_uri,
                dependency.virtual_result.code.as_str(),
            )
        })
}

pub(crate) fn tower_range(range: tower_lsp::lsp_types::Range) -> LspRange {
    LspRange {
        start: LspPosition {
            line: range.start.line,
            character: range.start.character,
        },
        end: LspPosition {
            line: range.end.line,
            character: range.end.character,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::linked_offset;

    #[test]
    fn links_the_matching_generated_pair_when_authored_text_collides() {
        let pair =
            "type __R_shared = typeof shared;\nvar shared: __U<__R_shared> = undefined as any;\n";
        let code = format!("{pair}// generated pair\n{pair}");
        let generated_start = code.rfind("typeof shared").unwrap() + "typeof ".len();
        let generated_shadow = code.rfind("var shared").unwrap() + "var ".len();

        assert_eq!(
            linked_offset(&code, generated_start, generated_start + "shared".len(),),
            Some(generated_shadow),
        );
        assert_eq!(
            linked_offset(&code, generated_shadow, generated_shadow + "shared".len(),),
            Some(generated_start),
        );
    }
}
