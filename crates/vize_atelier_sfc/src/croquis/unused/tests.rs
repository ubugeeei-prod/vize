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
