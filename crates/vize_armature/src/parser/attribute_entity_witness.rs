//! Named BEFORE/AFTER consumer witnesses, never normalization of actual output.
//!
//! BEFORE executes the unchanged public first-scalar compatibility boundary.
//! An authored attribute-only mutation of that whole AST supplies expected
//! AFTER. Current production captures must equal every expected field, and
//! complete errors/raw spans stay exact. Original frozen bins remain history.

use super::{Parser, compat_reference::parse_with_compat};
use vize_l0::{Allocator, String, cstr};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode, options::ParserOptions};

#[derive(Clone, Copy)]
pub(super) struct Change {
    pub(super) directive: bool,
    pub(super) name: &'static str,
    pub(super) before: &'static str,
    pub(super) after: &'static str,
}

fn apply_authored_changes(
    children: &mut [TemplateChildNode<'_>],
    changes: &[Change],
    hits: &mut [usize],
) {
    for child in children {
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        for prop in &mut element.props {
            for (index, change) in changes.iter().enumerate() {
                let content = match prop {
                    PropNode::Attribute(attribute)
                        if !change.directive && attribute.name == change.name =>
                    {
                        attribute.value.as_mut().map(|value| &mut value.content)
                    }
                    PropNode::Directive(directive)
                        if change.directive && directive.name == change.name =>
                    {
                        // The two frozen option witnesses have invalid expression
                        // bytes before and after. No JS AST is rewritten here.
                        let Some(ExpressionNode::Simple(expression)) = directive.exp.as_mut()
                        else {
                            panic!(
                                "authored direct Simple expression; refuse Compound or absent value"
                            );
                        };
                        assert!(expression.js_ast.is_none(), "no retained-AST workaround");
                        Some(&mut expression.content)
                    }
                    _ => None,
                };
                if let Some(content) = content {
                    assert_eq!(*content, change.before, "whole BEFORE value");
                    *content = change.after;
                    hits[index] += 1;
                }
            }
        }
        apply_authored_changes(&mut element.children, changes, hits);
    }
}

pub(super) fn prove_named_after(
    id: &str,
    source: &str,
    document: bool,
    options: &ParserOptions,
    actual_tree: &str,
    actual_errors: &str,
    changes: &[Change],
) -> String {
    let allocator = Allocator::new();
    let parser = if document {
        Parser::document_with_options(&allocator, source, options.clone())
    } else {
        Parser::with_options(&allocator, source, options.clone())
    };
    let (mut before, errors) = parse_with_compat(parser);
    let historical_tree = cstr!("{before:#?}");
    let historical_errors = cstr!("{errors:#?}");
    let option_fields = cstr!("{options:?}");
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/attribute-entity-consumer-8142");
    std::fs::create_dir_all(&directory).expect("whole named consumer packets");
    let mut raw = Vec::new();
    for field in [
        source,
        option_fields.as_str(),
        historical_tree.as_str(),
        historical_errors.as_str(),
        actual_tree,
        actual_errors,
    ] {
        raw.extend_from_slice(&(field.len() as u64).to_le_bytes());
        raw.extend_from_slice(field.as_bytes());
    }
    std::fs::write(directory.join(cstr!("{id}.raw.bin").as_str()), raw)
        .expect("retain actual whole packets before judging");
    let mut hits = vec![0; changes.len()];
    apply_authored_changes(&mut before.children, changes, &mut hits);
    let expected_tree = cstr!("{before:#?}");
    // Preserve whole original source/options, true BEFORE and actual AFTER
    // before any assertion. AFTER is never reduced to a historical projection.
    let fields = [
        source,
        option_fields.as_str(),
        historical_tree.as_str(),
        historical_errors.as_str(),
        expected_tree.as_str(),
        actual_tree,
        actual_errors,
    ];
    let mut packet = Vec::new();
    for field in fields {
        packet.extend_from_slice(&(field.len() as u64).to_le_bytes());
        packet.extend_from_slice(field.as_bytes());
    }
    std::fs::write(directory.join(cstr!("{id}.bin").as_str()), packet)
        .expect("retain complete BEFORE/expected/actual AFTER");
    assert_eq!(
        hits,
        vec![1; changes.len()],
        "each authored mutation occurs once"
    );
    assert_eq!(
        actual_errors,
        historical_errors.as_str(),
        "every original diagnostic field: {source}"
    );
    assert_eq!(
        actual_tree,
        expected_tree.as_str(),
        "every authored AFTER field: {source}"
    );
    historical_tree
}

#[cfg(feature = "legacy")]
pub(super) fn named_frozen_options_before(
    index: usize,
    source: &str,
    document: bool,
    options: &ParserOptions,
    tree: &str,
    errors: &str,
) -> Option<String> {
    let (required_source, in_tag_comments, change) = match index {
        6 => (
            "<p v-pre title='&fjlig;'>&fjlig; {{x}}</p>",
            false,
            Change {
                directive: false,
                name: "title",
                before: "f",
                after: "fj",
            },
        ),
        9 => (
            "<X // comment\n :title='&acE;'/>",
            true,
            Change {
                directive: true,
                name: "bind",
                before: "∾",
                after: "∾\u{333}",
            },
        ),
        _ => return None,
    };
    assert_eq!(source, required_source, "original frozen source");
    assert!(!document, "original frozen Component profile");
    assert_original_options(options, in_tag_comments);
    Some(prove_named_after(
        cstr!("frozen-option-{index}").as_str(),
        source,
        document,
        options,
        tree,
        errors,
        &[change],
    ))
}

#[cfg(feature = "legacy")]
fn assert_original_options(options: &ParserOptions, expected_in_tag_comments: bool) {
    // Exhaustive fields refuse future option growth without explicit custody.
    let ParserOptions {
        mode,
        whitespace,
        delimiters,
        is_pre_tag,
        is_native_tag,
        is_custom_element,
        custom_renderer,
        is_void_tag,
        get_namespace,
        on_error,
        on_warn,
        comments,
        experimental_in_tag_comments,
        dialect,
    } = options;
    let defaults = ParserOptions::default();
    assert_eq!(*mode, defaults.mode);
    assert_eq!(*whitespace, defaults.whitespace);
    assert_eq!(delimiters, &defaults.delimiters);
    assert!(core::ptr::fn_addr_eq(*is_pre_tag, defaults.is_pre_tag));
    assert!(is_native_tag.is_none());
    assert!(is_custom_element.is_none());
    assert!(!*custom_renderer);
    assert!(core::ptr::fn_addr_eq(*is_void_tag, defaults.is_void_tag));
    assert!(core::ptr::fn_addr_eq(
        *get_namespace,
        defaults.get_namespace
    ));
    assert!(on_error.is_none() && on_warn.is_none());
    assert!(*comments);
    assert_eq!(*experimental_in_tag_comments, expected_in_tag_comments);
    assert_eq!(*dialect, defaults.dialect);
}

pub(super) mod tests;
