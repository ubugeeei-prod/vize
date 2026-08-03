use super::input_size::{input_intrinsic_size, wrapped_line_count};
use super::render_payload::{
    RenderNodeKindNapi, parse_render_node_kind, validate_render_node_kinds,
};
use super::types::{FlexStyleNapi, RenderNodeNapi};

fn render_node(id: i64, node_type: &str) -> RenderNodeNapi {
    RenderNodeNapi {
        id,
        node_type: node_type.into(),
        text: None,
        wrap: None,
        wrap_mode: None,
        value: None,
        placeholder: None,
        focused: None,
        cursor: None,
        mask: None,
        mask_char: None,
        style: None,
        appearance: None,
        border: None,
        children: None,
    }
}

fn input_node(value: &str, width: Option<&str>) -> RenderNodeNapi {
    RenderNodeNapi {
        value: Some(value.into()),
        style: width.map(|width| FlexStyleNapi {
            width: Some(width.into()),
            ..FlexStyleNapi::default()
        }),
        ..render_node(1, "input")
    }
}

#[test]
fn render_node_kind_accepts_every_public_protocol_literal() {
    for (literal, expected) in [
        ("root", RenderNodeKindNapi::Root),
        ("box", RenderNodeKindNapi::Box),
        ("text", RenderNodeKindNapi::Text),
        ("input", RenderNodeKindNapi::Input),
    ] {
        assert_eq!(parse_render_node_kind(literal).unwrap(), expected);
    }
}

#[test]
fn render_node_kind_rejects_unknown_protocol_values() {
    assert_eq!(parse_render_node_kind("grid"), None);
}

#[test]
fn render_node_kind_literals_are_exact_and_case_sensitive() {
    for value in ["", "Box", " box", "box ", "raw"] {
        assert_eq!(parse_render_node_kind(value), None);
    }
}

#[test]
fn render_tree_payload_validation_fails_the_complete_batch_closed() {
    let nodes = [render_node(1, "root"), render_node(2, "unknown")];
    let error = validate_render_node_kinds(&nodes).unwrap_err();

    assert_eq!(error, "unknown");
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
