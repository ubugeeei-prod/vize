use super::input_size::{input_intrinsic_size, wrapped_line_count};
use super::render_payload::{
    RenderNodeKindNapi, parse_render_node_kind, validate_render_node_kinds,
};
use super::types::{FlexStyleNapi, InputEventNapi, RenderNodeNapi, StyleNapi};
use crate::input::{Event, Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

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
    let nodes = [
        RenderNodeNapi {
            style: Some(FlexStyleNapi {
                width: Some("80".into()),
                height: Some("24".into()),
                ..FlexStyleNapi::default()
            }),
            children: Some(vec![2]),
            ..render_node(1, "root")
        },
        RenderNodeNapi {
            style: Some(FlexStyleNapi {
                flex_direction: Some("row".into()),
                padding_left: Some(1.0),
                ..FlexStyleNapi::default()
            }),
            appearance: Some(StyleNapi {
                fg: Some("cyan".into()),
                bold: Some(true),
                ..StyleNapi::default()
            }),
            border: Some("rounded".into()),
            children: Some(vec![3, 4]),
            ..render_node(2, "box")
        },
        RenderNodeNapi {
            text: Some("hello".into()),
            wrap: Some(true),
            wrap_mode: Some("truncate-end".into()),
            ..render_node(3, "text")
        },
        RenderNodeNapi {
            value: Some("secret".into()),
            placeholder: Some("type here".into()),
            focused: Some(true),
            cursor: Some(6),
            mask: Some(true),
            mask_char: Some("#".into()),
            ..render_node(4, "input")
        },
    ];

    assert_eq!(
        validate_render_node_kinds(&nodes).unwrap(),
        vec![
            RenderNodeKindNapi::Root,
            RenderNodeKindNapi::Box,
            RenderNodeKindNapi::Text,
            RenderNodeKindNapi::Input,
        ]
    );

    assert_eq!(nodes[0].id, 1);
    assert_eq!(nodes[0].node_type, "root");
    let root_style = nodes[0].style.as_ref().unwrap();
    assert_eq!(root_style.width.as_deref(), Some("80"));
    assert_eq!(root_style.height.as_deref(), Some("24"));
    assert_eq!(nodes[0].children.as_deref(), Some([2].as_slice()));

    assert_eq!(nodes[1].id, 2);
    assert_eq!(nodes[1].node_type, "box");
    let box_style = nodes[1].style.as_ref().unwrap();
    assert_eq!(box_style.flex_direction.as_deref(), Some("row"));
    assert_eq!(box_style.padding_left, Some(1.0));
    let box_appearance = nodes[1].appearance.as_ref().unwrap();
    assert_eq!(box_appearance.fg.as_deref(), Some("cyan"));
    assert_eq!(box_appearance.bold, Some(true));
    assert_eq!(nodes[1].border.as_deref(), Some("rounded"));
    assert_eq!(nodes[1].children.as_deref(), Some([3, 4].as_slice()));

    assert_eq!(nodes[2].id, 3);
    assert_eq!(nodes[2].node_type, "text");
    assert_eq!(nodes[2].text.as_deref(), Some("hello"));
    assert_eq!(nodes[2].wrap, Some(true));
    assert_eq!(nodes[2].wrap_mode.as_deref(), Some("truncate-end"));

    assert_eq!(nodes[3].id, 4);
    assert_eq!(nodes[3].node_type, "input");
    assert_eq!(nodes[3].value.as_deref(), Some("secret"));
    assert_eq!(nodes[3].placeholder.as_deref(), Some("type here"));
    assert_eq!(nodes[3].focused, Some(true));
    assert_eq!(nodes[3].cursor, Some(6));
    assert_eq!(nodes[3].mask, Some(true));
    assert_eq!(nodes[3].mask_char.as_deref(), Some("#"));
}

#[test]
fn input_event_conversion_confirms_every_existing_discriminator() {
    let events = [
        Event::Key(KeyEvent::new(
            Key::Char('x'),
            KeyModifiers {
                ctrl: true,
                alt: true,
                shift: true,
                super_key: true,
                hyper: true,
                meta: true,
            },
        )),
        Event::Mouse(MouseEvent::new(
            MouseEventKind::Down(MouseButton::Left),
            2,
            3,
            KeyModifiers::NONE,
        )),
        Event::Resize(80, 24),
        Event::FocusGained,
        Event::FocusLost,
        Event::Paste("pasted".into()),
    ];

    let converted = events
        .into_iter()
        .map(InputEventNapi::from)
        .collect::<Vec<_>>();
    let discriminators = converted
        .iter()
        .map(|event| event.event_type.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        discriminators,
        ["key", "mouse", "resize", "focus", "focus", "paste"]
    );

    let key = &converted[0];
    assert_eq!(key.key, None);
    assert_eq!(key.char.as_deref(), Some("x"));
    assert_eq!(key.key_event_type.as_deref(), Some("press"));
    let modifiers = key.modifiers.as_ref().unwrap();
    assert!(modifiers.ctrl);
    assert!(modifiers.alt);
    assert!(modifiers.shift);
    assert!(modifiers.meta);
    assert!(modifiers.super_key);
    assert!(modifiers.hyper);
    assert!(!modifiers.caps_lock);
    assert!(!modifiers.num_lock);

    let mouse = &converted[1];
    assert_eq!(mouse.button.as_deref(), Some("left"));
    assert_eq!(mouse.x, Some(2));
    assert_eq!(mouse.y, Some(3));
    assert!(mouse.modifiers.is_none());

    let resize = &converted[2];
    assert_eq!(resize.width, Some(80));
    assert_eq!(resize.height, Some(24));

    assert_eq!(converted[3].key.as_deref(), Some("gained"));
    assert_eq!(converted[4].key.as_deref(), Some("lost"));
    assert_eq!(converted[5].text.as_deref(), Some("pasted"));
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
