use super::{TypeIntelligence, get_vue_global_hover, is_ident_char};
use vize_croquis::Croquis;
use vize_relief::BindingType;

#[test]
fn test_is_ident_char() {
    assert!(is_ident_char(b'a'));
    assert!(is_ident_char(b'Z'));
    assert!(is_ident_char(b'0'));
    assert!(is_ident_char(b'_'));
    assert!(is_ident_char(b'$'));
    assert!(!is_ident_char(b' '));
    assert!(!is_ident_char(b'.'));
}

#[test]
fn test_vue_global_hover() {
    assert!(get_vue_global_hover("$attrs").is_some());
    assert!(get_vue_global_hover("$emit").is_some());
    assert!(get_vue_global_hover("unknown").is_none());
}

#[test]
fn test_definition_lookup() {
    // Source: "const count = ref(0)"
    let source = "const count = ref(0)";
    let mut summary = Croquis::default();
    summary.note_binding("count", BindingType::SetupRef);
    // "count" starts at offset 6, ends at 11
    summary.note_binding_span("count", 6, 11);

    let intel = TypeIntelligence::new(source, &summary);

    // Cursor on "count" (offset 7) should find definition
    let loc = intel.definition(7);
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.span.start, 6);
    assert_eq!(loc.span.end, 11);
}

#[test]
fn test_definition_unknown_ident() {
    let source = "const count = ref(0)";
    let summary = Croquis::default();
    let intel = TypeIntelligence::new(source, &summary);

    // "count" not in binding_spans → None
    let loc = intel.definition(7);
    assert!(loc.is_none());
}

#[test]
fn test_definition_not_on_ident() {
    let source = "const count = ref(0)";
    let summary = Croquis::default();
    let intel = TypeIntelligence::new(source, &summary);

    // Offset 5 is space → None
    let loc = intel.definition(5);
    assert!(loc.is_none());
}
