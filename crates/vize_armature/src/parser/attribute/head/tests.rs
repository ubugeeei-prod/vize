//! Completed-head custody and original public prefixes without side storage.

use crate::parser::Parser;
use crate::tokenizer::QuoteType;
use vize_l0::{Allocator, cstr};
use vize_relief::{PropNode, TemplateChildNode, errors::ErrorCode};

#[test]
fn completed_heads_join_only_their_original_source_and_semantic_prefix() {
    let arena = Allocator::new();
    for (authored, semantic, prefix_end) in [
        ("v-bind:[keys['name]']].camel.stop", "bind", 6),
        ("v-äö:arg.mod", "äö", 6),
        (":[keys['name]']].camel", "bind", 1),
        ("@click.stop", "on", 1),
        ("#head", "slot", 1),
        (".foo", "bind", 1),
    ] {
        let parser = Parser::new(&arena, authored);
        let name = if authored.starts_with("v-") {
            &authored[2..prefix_end]
        } else {
            semantic
        };
        let (prefix, complete, end) = parser
            .completed_directive_head(name, Some(authored), 0, authored.len())
            .expect("original borrowed head");
        assert_eq!(prefix, &authored[..prefix_end]);
        assert!(std::ptr::eq(complete, authored));
        assert_eq!(end, authored.len());
        assert_eq!(
            parser.completed_directive_head(name, None, 0, authored.len()),
            None
        );
        let copy = arena.alloc_str(authored);
        assert_eq!(
            parser.completed_directive_head(name, Some(copy), 0, authored.len()),
            None
        );
    }
    let source = "v-bind:arg";
    let parser = Parser::new(&arena, source);
    let copied_name = arena.alloc_str(&source[2..6]);
    assert_eq!(
        parser.completed_directive_head(copied_name, Some(source), 0, source.len()),
        None
    );
    assert_eq!(
        parser.completed_directive_head(&source[3..6], Some(source), 0, source.len()),
        None
    );
    let source = ":arg";
    let parser = Parser::new(&arena, source);
    assert_eq!(
        parser.completed_directive_head("on", Some(source), 0, source.len()),
        None
    );
}

#[test]
fn completed_heads_use_exact_source_boundaries_without_normalized_recovery() {
    let arena = Allocator::new();
    let source = "<div v-bind:x='v'>";
    let parser = Parser::new(&arena, source);
    let (prefix, authored, end) = parser
        .completed_directive_head(&source[7..11], Some(&source[5..13]), 5, 16)
        .expect("exact already-tokenized boundaries");
    assert_eq!(prefix, "v-bind");
    assert_eq!(authored, "v-bind:x");
    assert_eq!(end, 13);
    for end in [8, usize::MAX] {
        let mut parser = Parser::new(&arena, "<div v-äö>");
        parser.frozen_elements = Some(vize_l0::Vec::new_in(&parser.allocator));
        parser.on_open_tag_name_impl(1, 4);
        parser.on_dir_name_impl(5, 11);
        parser.on_attrib_name_end_impl(end);
        parser.on_attrib_end_impl(QuoteType::NoValue, 11);
        assert!(parser.current_element.is_none());
        parser.on_open_tag_end_impl(11);
        assert!(parser.stack.is_empty());
        let (root, errors) = parser.into_result();
        assert!(root.children.is_empty());
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].code, ErrorCode::MissingDirectiveName);
        assert!(!errors[0].is_recoverable());
    }
}

#[test]
fn wide_normal_openings_restore_all_original_prefixes_and_public_fields() {
    let source = "<div id='plain' :a='v' @b.stop='f' #slot='.slot' .prop='v' v-bind:label='v' v-on:click.stop='f' v-custom:äö.mod='v' :[keys['name]']].camel='v' @extra='f' :last='v'></div>";
    let arena = Allocator::new();
    let (root, errors) = Parser::new(&arena, source).parse();
    let (control, control_errors, frozen) =
        Parser::new(&arena, source).parse_with_frozen_elements();
    assert!(errors.is_empty(), "{errors:?}");
    assert!(frozen.spans_for(control.source, source).unwrap().is_empty());
    assert_eq!(cstr!("{root:#?}"), cstr!("{control:#?}"));
    assert_eq!(cstr!("{errors:#?}"), cstr!("{control_errors:#?}"));
    let Some(TemplateChildNode::Element(element)) = root.children.first() else {
        panic!("whole original opening");
    };
    let raw_names: Vec<_> = element
        .props
        .iter()
        .filter_map(|prop| match prop {
            PropNode::Directive(dir) => dir.raw_name,
            PropNode::Attribute(_) => None,
        })
        .collect();
    assert_eq!(
        raw_names,
        [
            ":", "@", "#", ".", "v-bind", "v-on", "v-custom", ":", "@", ":"
        ]
    );
    let PropNode::Directive(dot) = &element.props[4] else {
        panic!("original dot shorthand");
    };
    assert_eq!(dot.modifiers[0].content, "prop");
}

#[test]
fn eof_recovery_restores_original_normal_prefixes_before_attachment() {
    for (source, prefix) in [
        ("<div v-bind:label='unfinished", "v-bind"),
        ("<div :label", ":"),
        ("<div @click.stop", "@"),
        ("<div #slot", "#"),
        ("<div .foo", "."),
        ("<div v-custom:[key", "v-custom"),
    ] {
        let arena = Allocator::new();
        let (root, errors) = Parser::new(&arena, source).parse();
        let (control, control_errors, _) = Parser::new(&arena, source).parse_with_frozen_elements();
        assert_eq!(cstr!("{root:#?}"), cstr!("{control:#?}"));
        assert_eq!(cstr!("{errors:#?}"), cstr!("{control_errors:#?}"));
        assert!(!errors.is_empty(), "original incomplete opening {source:?}");
        let Some(TemplateChildNode::Element(element)) = root.children.first() else {
            panic!("original recovered opening {source:?}");
        };
        let Some(PropNode::Directive(dir)) = element.props.first() else {
            panic!("original recovered directive {source:?}");
        };
        assert_eq!(dir.raw_name, Some(prefix), "{source:?}");
    }
}

fn completed_pre<'a>(parser: &mut Parser<'a>) {
    parser.on_open_tag_name_impl(1, 4);
    parser.on_dir_name_impl(5, 10);
    parser.on_attrib_name_end_impl(10);
    parser.on_attrib_end_impl(QuoteType::NoValue, 10);
}

#[test]
fn foreign_head_or_semantic_prefix_mismatch_is_fatal_without_an_element_escape() {
    for foreign in [false, true] {
        let arena = Allocator::new();
        let source = "<div v-pre>";
        let mut parser = Parser::new(&arena, source);
        parser.frozen_elements = Some(vize_l0::Vec::new_in(&parser.allocator));
        completed_pre(&mut parser);
        let current = parser.current_element.as_mut().unwrap();
        if foreign {
            let Some(PropNode::Directive(dir)) = current.props.first_mut() else {
                panic!("original completed pre");
            };
            dir.raw_name = Some(arena.alloc_str("v-pre"));
        } else {
            let Some(PropNode::Directive(dir)) = current.props.first_mut() else {
                panic!("completed pre")
            };
            dir.name = "bind";
        }
        parser.on_open_tag_end_impl(10);
        assert!(parser.stack.is_empty());
        let (root, errors) = parser.into_result();
        assert!(root.children.is_empty());
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].code, ErrorCode::MissingDirectiveName);
        assert!(!errors[0].is_recoverable());
    }
}

#[test]
fn an_ignored_nested_form_keeps_its_original_recovery_before_custody_checks() {
    let arena = Allocator::new();
    let mut parser = Parser::new(&arena, "<form :id>");
    parser.frozen_elements = Some(vize_l0::Vec::new_in(&parser.allocator));
    parser.open_form_count = 1;
    parser.on_open_tag_name_impl(1, 5);
    parser.on_dir_name_impl(6, 7);
    parser.on_attrib_name_end_impl(9);
    parser.on_attrib_end_impl(QuoteType::NoValue, 9);
    let current = parser.current_element.as_mut().unwrap();
    let Some(PropNode::Directive(dir)) = current.props.first_mut() else {
        panic!("original ignored form attribute");
    };
    dir.raw_name = None;
    parser.on_open_tag_end_impl(9);
    assert!(parser.stack.is_empty());
    let (root, errors) = parser.into_result();
    assert!(root.children.is_empty());
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, ErrorCode::ExtendPoint);
    assert_eq!(
        errors[0].message,
        "HTML tree construction ignored this start tag because an equivalent element is already open."
    );
}

#[test]
fn another_same_source_shorthand_cannot_replace_the_completed_property_head() {
    let arena = Allocator::new();
    let source = "<div :id :other>";
    let mut parser = Parser::new(&arena, source);
    parser.frozen_elements = Some(vize_l0::Vec::new_in(&parser.allocator));
    parser.on_open_tag_name_impl(1, 4);
    parser.on_dir_name_impl(5, 6);
    parser.on_dir_arg_impl(6, 8);
    parser.on_attrib_name_end_impl(8);
    parser.on_attrib_end_impl(QuoteType::NoValue, 8);
    let current = parser.current_element.as_mut().unwrap();
    let Some(PropNode::Directive(dir)) = current.props.first_mut() else {
        panic!("original first shorthand");
    };
    dir.raw_name = Some(&source[9..15]);
    parser.on_open_tag_end_impl(15);
    assert!(parser.stack.is_empty());
    let (root, errors) = parser.into_result();
    assert!(root.children.is_empty());
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, ErrorCode::MissingDirectiveName);
    assert!(!errors[0].is_recoverable());
}

#[test]
fn successful_props_determine_first_pre_and_all_duplicate_markers_are_removed() {
    let arena = Allocator::new();
    let source = "<div :before.mod='v' v-pre :after.mod='v' v-pre></div>";
    let (default, default_errors) = Parser::new(&arena, source).parse();
    let (compiler, compiler_errors, _) = Parser::new(&arena, source).parse_with_frozen_elements();
    assert_eq!(cstr!("{default_errors:?}"), cstr!("{compiler_errors:?}"));
    for (root, expected) in [
        (&default, [":before", ":after"]),
        (&compiler, [":before.mod", ":after.mod"]),
    ] {
        let TemplateChildNode::Element(element) = &root.children[0] else {
            panic!("owner")
        };
        let names: Vec<_> = element
            .props
            .iter()
            .map(|prop| match prop {
                PropNode::Attribute(attr) => attr.name,
                PropNode::Directive(_) => panic!("semantic marker escaped"),
            })
            .collect();
        assert_eq!(names, expected);
    }
}
