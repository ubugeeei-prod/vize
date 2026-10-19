//! Style reads complete the demanded setup candidate relation.

use crate::SfcDescriptor;
use vize_croquis::Croquis;
use vize_croquis::binding_occurrences::BindingOccurrences;
use vize_croquis::drawer::{extract_identifier_refs_with_witness, extract_identifiers_checked};

pub(super) fn apply_style_reads(
    croquis: &mut Croquis,
    descriptor: &SfcDescriptor<'_>,
    derived: bool,
    mut occurrences: Option<&mut BindingOccurrences>,
) -> bool {
    let mut packet_valid = true;
    if descriptor.script_setup.as_ref().is_some_and(|block| {
        block.src.is_some()
            || block
                .lang
                .as_deref()
                .is_some_and(|lang| !matches!(lang, "js" | "ts" | "jsx" | "tsx"))
    }) || descriptor.template.as_ref().is_some_and(|block| {
        block.src.is_some()
            || (!derived && block.lang.as_deref().is_some_and(|lang| lang != "html"))
    }) || descriptor.styles.iter().any(|style| style.src.is_some())
    {
        // External or unsupported blocks can contain reads we cannot see.
        croquis.unused_bindings.clear();
        return false;
    }
    for (style_index, style) in descriptor.styles.iter().enumerate() {
        let Some(ranges) =
            vize_croquis::sfc::__internal::checked_v_bind_expression_ranges(&style.content)
        else {
            croquis.unused_bindings.clear();
            return false;
        };
        for range in ranges {
            let Some(expression) = style.content.get(range.clone()) else {
                if occurrences.is_some() {
                    packet_valid = false;
                    occurrences = None;
                }
                continue;
            };
            let reads = if let Some(packet) = occurrences.as_deref_mut() {
                let Some((references, witnessed)) =
                    extract_identifier_refs_with_witness(expression)
                else {
                    croquis.unused_bindings.clear();
                    return false;
                };
                let mut reads = Vec::with_capacity(references.len());
                let mut refused = !witnessed;
                for reference in references {
                    if !witnessed {
                        reads.push(reference.name);
                        continue;
                    }
                    if let Some(&declaration) = croquis.binding_spans.get(reference.name.as_str()) {
                        let Some(start) = (range.start as u32).checked_add(reference.offset) else {
                            refused = true;
                            reads.push(reference.name);
                            continue;
                        };
                        let Some(end) = start.checked_add(reference.name.len() as u32) else {
                            refused = true;
                            reads.push(reference.name);
                            continue;
                        };
                        if style.content.get(start as usize..end as usize)
                            != Some(reference.name.as_str())
                            || packet
                                .note_style_read(
                                    &reference.name,
                                    declaration,
                                    style_index as u32,
                                    start,
                                    end,
                                )
                                .is_none()
                        {
                            refused = true;
                        }
                    }
                    reads.push(reference.name);
                }
                if refused {
                    packet_valid = false;
                    occurrences = None;
                }
                reads
            } else {
                let Some(reads) = extract_identifiers_checked(expression) else {
                    croquis.unused_bindings.clear();
                    return false;
                };
                reads
            };
            for name in reads {
                croquis
                    .unused_bindings
                    .retain(|candidate| candidate.as_str() != name.as_str());
            }
        }
    }
    packet_valid
}

#[cfg(test)]
mod tests {
    //! Packet refusal must conserve every ordinary style-read result.

    use super::apply_style_reads;
    use crate::{SfcParseOptions, parse_sfc};
    use vize_carton::CompactString;
    use vize_croquis::{Croquis, binding_occurrences::BindingOccurrences};

    #[test]
    fn refused_packet_still_removes_reads_in_the_same_and_later_style_blocks() {
        let source = "<style>.a{color:v-bind(first);width:v-bind(later)}</style>\
                  <style>.b{height:v-bind(last)}</style>";
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let mut ordinary = Croquis::default();
        let mut captured = Croquis::default();
        for croquis in [&mut ordinary, &mut captured] {
            for (index, name) in ["first", "later", "last"].into_iter().enumerate() {
                croquis.unused_bindings.push(CompactString::new(name));
                croquis
                    .binding_spans
                    .insert(CompactString::new(name), (index as u32, index as u32 + 1));
            }
        }
        let mut packet = BindingOccurrences::default();
        assert!(apply_style_reads(&mut ordinary, &descriptor, false, None));
        assert!(!apply_style_reads(
            &mut captured,
            &descriptor,
            false,
            Some(&mut packet),
        ));
        assert!(ordinary.unused_bindings.is_empty());
        assert_eq!(ordinary.to_vir(), captured.to_vir());
        assert_eq!(
            serde_json::to_value(ordinary.semantic_snapshot()).unwrap(),
            serde_json::to_value(captured.semantic_snapshot()).unwrap(),
        );
    }
}
