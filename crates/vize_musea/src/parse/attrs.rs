//! Attribute reads over S1 open tags.
//!
//! Names and values are the authored S1 tokens: names compare exactly
//! (Art attributes are case-sensitive, as before), values are the verbatim
//! bytes between the quotes (entities undecoded — `args` / `viewport`
//! decode their own payloads).

use vize_s1::{Attribute, OpenTag};

/// The value of the first attribute named `name` that carries one.
///
/// A valueless occurrence (`title`) or an announced-but-absent value
/// (`title= >`, an S1 `Missing` hole) does not answer; a later valued
/// duplicate still can.
pub(crate) fn attr_value<'a>(open: &OpenTag<'a>, name: &str) -> Option<&'a str> {
    open.attrs
        .iter()
        .filter(|attr| attr.name.text == name)
        .find_map(present_value)
}

/// Whether any attribute named `name` is authored, valued or not.
pub(crate) fn has_attr(open: &OpenTag<'_>, name: &str) -> bool {
    open.attrs.iter().any(|attr| attr.name.text == name)
}

fn present_value<'a>(attr: &Attribute<'a>) -> Option<&'a str> {
    let value = attr.value.as_ref()?;
    (!value.content.is_missing()).then_some(value.content.text)
}

#[cfg(test)]
mod tests {
    use super::{attr_value, has_attr};
    use vize_s0::Allocator;
    use vize_s1::{OpenTag, SurfaceChild, parse};

    fn with_open_tag(source: &str, check: impl FnOnce(&OpenTag<'_>)) {
        let allocator = Allocator::new();
        let (tree, _) = parse(&allocator, source);
        let Some(SurfaceChild::Element(element)) = tree.children.first() else {
            panic!("expected one element in {source:?}");
        };
        check(&element.open);
    }

    #[test]
    fn values_are_the_authored_bytes() {
        with_open_tag(r#"<a t="Hello" u='x' v=w/x y="&amp;">"#, |open| {
            assert_eq!(attr_value(open, "t"), Some("Hello"));
            assert_eq!(attr_value(open, "u"), Some("x"));
            assert_eq!(attr_value(open, "v"), Some("w/x"));
            assert_eq!(attr_value(open, "y"), Some("&amp;"));
            assert_eq!(attr_value(open, "z"), None);
        });
    }

    #[test]
    fn valueless_and_missing_values_do_not_answer() {
        with_open_tag(r#"<a t e="" t="late" u= >"#, |open| {
            assert_eq!(attr_value(open, "t"), Some("late"));
            assert_eq!(attr_value(open, "u"), None);
            assert_eq!(attr_value(open, "e"), Some(""));
        });
    }

    #[test]
    fn names_compare_exactly() {
        with_open_tag(
            r#"<a :title="x" data-title="y" Title="z" default>"#,
            |open| {
                assert_eq!(attr_value(open, "title"), None);
                assert!(has_attr(open, "default"));
                assert!(!has_attr(open, "defaults"));
                assert!(!has_attr(open, "Default"));
            },
        );
    }
}
