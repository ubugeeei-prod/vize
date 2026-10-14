use super::{CssContext, at};

fn classify(marked: &str) -> CssContext<'_> {
    let offset = marked.find('|').unwrap();
    // The cursor marker sits at the end of each probe, so slicing it off keeps offsets.
    at(marked.get(..offset).unwrap(), offset)
}

#[test]
fn indented_declarations_complete_properties_and_values() {
    assert_eq!(
        classify(".card\n  col|"),
        CssContext::Property {
            prefix: "col",
            span: (8, 11),
            has_colon: false
        }
    );
    // Sass `name: value` and Stylus `name value`.
    for marked in [".card\n  color: re|", ".card\n  color re|"] {
        let CssContext::Value {
            property, prefix, ..
        } = classify(marked)
        else {
            panic!("expected a value context for {marked:?}");
        };
        assert_eq!((property, prefix), ("color", "re"));
    }
}

#[test]
fn indented_selectors_complete_pseudos_and_at_rules() {
    assert_eq!(
        classify(".card\n  &:hov|"),
        CssContext::Pseudo {
            prefix: ":hov",
            span: (9, 13)
        }
    );
    assert_eq!(
        classify("@med|"),
        CssContext::AtRule {
            prefix: "@med",
            span: (0, 4)
        }
    );
    // A top-level selector line is not a declaration.
    assert_eq!(classify("car|"), CssContext::Unknown);
    assert_eq!(classify(".card\n  // col|"), CssContext::Unknown);
}
