use super::input_size::{input_intrinsic_size, wrapped_line_count};
use super::types::{FlexStyleNapi, RenderNodeNapi};

fn input_node(value: &str, width: Option<&str>) -> RenderNodeNapi {
    RenderNodeNapi {
        id: 1,
        node_type: "input".into(),
        text: None,
        wrap: None,
        wrap_mode: None,
        value: Some(value.into()),
        placeholder: None,
        focused: None,
        cursor: None,
        mask: None,
        mask_char: None,
        style: width.map(|width| FlexStyleNapi {
            width: Some(width.into()),
            ..FlexStyleNapi::default()
        }),
        appearance: None,
        border: None,
        children: None,
    }
}

#[test]
fn input_height_respects_explicit_point_width() {
    let (_width, height) =
        input_intrinsic_size(&input_node("abcdefghijklmnopqrstuvwxy", Some("10")));

    assert_eq!(height, 3.0);
}

#[test]
fn input_height_does_not_wrap_exact_width_content() {
    assert_eq!(wrapped_line_count(30, 30), 1);
}

#[test]
fn input_height_falls_back_for_non_point_widths() {
    let (width, height) =
        input_intrinsic_size(&input_node("abcdefghijklmnopqrstuvwxy", Some("50%")));

    assert_eq!(width, 30.0);
    assert_eq!(height, 1.0);
}
