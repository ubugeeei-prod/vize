use super::{Dimension, Display, Edges, FlexDirection, FlexStyle, LengthPercentageAuto};

#[test]
fn test_flex_style_default() {
    let style = FlexStyle::new();
    assert_eq!(style.flex_direction, FlexDirection::Row);
    assert_eq!(style.display, Display::Flex);
}

#[test]
fn test_flex_style_to_taffy() {
    let mut style = FlexStyle::new();
    style.flex_direction = FlexDirection::Column;
    style.width = Dimension::Points(100.0);

    let taffy_style = style.to_taffy();
    assert_eq!(taffy_style.flex_direction, taffy::FlexDirection::Column);
}

#[test]
fn test_edges_all() {
    let edges = Edges::all(10.0);
    assert_eq!(edges.top, LengthPercentageAuto::Points(10.0));
    assert_eq!(edges.right, LengthPercentageAuto::Points(10.0));
}

#[test]
fn test_edges_default_is_zero() {
    let edges = Edges::default();
    assert_eq!(edges.top, LengthPercentageAuto::Points(0.0));
    assert_eq!(edges.right, LengthPercentageAuto::Points(0.0));
    assert_eq!(edges.bottom, LengthPercentageAuto::Points(0.0));
    assert_eq!(edges.left, LengthPercentageAuto::Points(0.0));
}
