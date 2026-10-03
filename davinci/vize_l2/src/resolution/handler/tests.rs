use super::{HandlerBindingRef, HandlerDeclarationKind, resolve_handler};
use crate::lang::js::NativeHandlerInput;
use crate::resolution::{BindingId, BindingLookup, ResolutionErrorKind, Usage};
use alloc::vec::Vec;
mod statements;
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1::markup::NativeTemplateComponent;

struct Outer(&'static [(&'static str, u32)]);
impl BindingLookup for Outer {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        self.0
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, id)| BindingId::new(*id))
    }
}

fn input<'a>(allocator: &'a Allocator, text: &str) -> NativeHandlerInput<'a> {
    let file = allocator.alloc_str(&alloc::format!(
        "<template><b @click=\"{text}\"/></template>"
    ));
    let descriptor = Vue.observe_descriptor(
        allocator,
        file,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: vize_l1::SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(allocator, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let element = selected.children().next().unwrap().into_element().unwrap();
    let operand = selected
        .observe_attribute_handler(element.attributes().next().unwrap())
        .unwrap();
    NativeHandlerInput::new(operand).unwrap()
}

#[test]
fn the_real_event_parameter_and_an_equal_outer_id_remain_distinct() {
    let allocator = Allocator::default();
    let original = input(&allocator, "$event+outer");
    let body = original.body();
    let resolution = resolve_handler(original, &Outer(&[("outer", 0), ("$event", 7)])).unwrap();
    assert!(core::ptr::eq(resolution.input().body(), body));
    assert_eq!(resolution.scopes().len(), 1);
    assert_eq!(resolution.scopes()[0].span, None);
    assert_eq!(
        resolution.bindings()[0].kind,
        HandlerDeclarationKind::EventParameter
    );
    assert!(resolution.declarations().is_empty());
    let [event, outer] = resolution.references() else {
        panic!("two original references")
    };
    assert!(matches!(event.binding, HandlerBindingRef::Local(id) if id.index()==0));
    assert_eq!(outer.binding, HandlerBindingRef::Outer(BindingId::new(0)));
    assert_eq!(event.span, Span::new(0, 6));
    assert_eq!(outer.span, Span::new(7, 12));
}

#[test]
fn forward_lexical_references_resolve_from_flat_facts_without_an_ast_rewalk() {
    let allocator = Allocator::default();
    let resolution = resolve_handler(
        input(&allocator, "x;let x=outer;x++"),
        &Outer(&[("outer", 3), ("x", 9)]),
    )
    .unwrap();
    let references = resolution.references();
    assert_eq!(
        references.iter().map(|row| row.name).collect::<Vec<_>>(),
        ["x", "outer", "x"]
    );
    assert!(matches!(references[0].binding, HandlerBindingRef::Local(id) if id.index()==1));
    assert_eq!(references[0].binding, references[2].binding);
    assert_eq!(references[0].usage, Usage::Read);
    assert_eq!(references[2].usage, Usage::ReadWrite);
    assert_eq!(
        references[1].binding,
        HandlerBindingRef::Outer(BindingId::new(3))
    );
    assert_eq!(resolution.declarations()[0].span, Span::new(6, 7));
}

#[test]
fn genuine_block_shadowing_does_not_leak_a_local_outside_its_scope() {
    let allocator = Allocator::default();
    let resolution = resolve_handler(
        input(&allocator, "x;{let x=outer;x}x"),
        &Outer(&[("outer", 2), ("x", 9)]),
    )
    .unwrap();
    let references = resolution.references();
    assert_eq!(references.len(), 4);
    assert_eq!(
        references[0].binding,
        HandlerBindingRef::Outer(BindingId::new(9))
    );
    assert!(matches!(references[2].binding, HandlerBindingRef::Local(id) if id.index()==1));
    assert_eq!(references[3].binding, references[0].binding);
    assert_eq!(resolution.scopes()[1].span, Some(Span::new(2, 17)));
    assert_eq!(
        resolution.scopes()[1].parent,
        Some(resolution.scopes()[0].id)
    );
    assert_eq!(resolution.bindings()[1].scope, resolution.scopes()[1].id);
}

#[test]
fn var_is_function_scoped_while_its_original_declaration_remains_in_the_block() {
    let allocator = Allocator::default();
    let resolution = resolve_handler(
        input(&allocator, "x;{var x=outer}x"),
        &Outer(&[("outer", 2)]),
    )
    .unwrap();
    assert_eq!(resolution.bindings()[1].scope, resolution.scopes()[0].id);
    assert_eq!(
        resolution.declarations()[0].scope,
        resolution.scopes()[1].id
    );
    assert_eq!(
        resolution.declarations()[0].kind,
        HandlerDeclarationKind::Var
    );
    assert_eq!(
        resolution.references()[0].binding,
        resolution.references()[2].binding
    );
    assert!(
        matches!(resolution.references()[0].binding, HandlerBindingRef::Local(id) if id.index()==1)
    );
    let resolution = resolve_handler(input(&allocator, "var x;var x;x"), &Outer(&[])).unwrap();
    assert_eq!(resolution.bindings().len(), 2);
    assert_eq!(resolution.declarations().len(), 2);
    assert_eq!(
        resolution.declarations()[0].binding,
        resolution.declarations()[1].binding
    );
}

#[test]
fn real_event_parameter_redeclaration_and_block_shadowing_follow_scope_rules() {
    let allocator = Allocator::default();
    let resolution = resolve_handler(input(&allocator, "var $event;$event"), &Outer(&[])).unwrap();
    assert_eq!(resolution.bindings().len(), 1);
    assert_eq!(
        resolution.declarations()[0].binding,
        resolution.bindings()[0].id
    );
    let resolution = resolve_handler(
        input(&allocator, "$event;{let $event=1;$event}$event"),
        &Outer(&[]),
    )
    .unwrap();
    assert_ne!(
        resolution.references()[0].binding,
        resolution.references()[1].binding
    );
    assert_eq!(
        resolution.references()[0].binding,
        resolution.references()[2].binding
    );
}

#[test]
fn real_if_branches_keep_original_reference_order_and_write_semantics() {
    let allocator = Allocator::default();
    let resolution = resolve_handler(
        input(&allocator, "if(test){x++}else return y"),
        &Outer(&[("test", 1), ("x", 2), ("y", 3)]),
    )
    .unwrap();
    assert_eq!(
        resolution
            .references()
            .iter()
            .map(|row| (row.name, row.usage))
            .collect::<Vec<_>>(),
        [
            ("test", Usage::Read),
            ("x", Usage::ReadWrite),
            ("y", Usage::Read)
        ]
    );
    assert_eq!(resolution.scopes().len(), 2);
    assert_eq!(resolution.references()[1].scope, resolution.scopes()[1].id);
    assert_eq!(resolution.references()[2].scope, resolution.scopes()[0].id);
    assert!(
        resolution
            .references()
            .windows(2)
            .all(|pair| pair[0].span.end <= pair[1].span.start)
    );
}

#[test]
fn missing_outer_bindings_return_the_whole_original_owner_without_partial_tables() {
    let allocator = Allocator::default();
    let original = input(&allocator, "/*keep*/ known; missing");
    let body = original.body();
    let comments = original
        .operand()
        .syntax()
        .comments()
        .next()
        .unwrap()
        .text()
        .unwrap()
        .as_ptr();
    let rejected = resolve_handler(original, &Outer(&[("known", 1)])).unwrap_err();
    assert_eq!(rejected.error.kind, ResolutionErrorKind::MissingBinding);
    assert_eq!(rejected.error.span, Span::new(16, 23));
    assert!(core::ptr::eq(rejected.input().body(), body));
    assert_eq!(
        rejected
            .input()
            .operand()
            .syntax()
            .comments()
            .next()
            .unwrap()
            .text()
            .unwrap()
            .as_ptr(),
        comments
    );
    assert_eq!(
        rejected.into_input().operand().raw_value(),
        "/*keep*/ known; missing"
    );
}

#[test]
fn generated_roots_empty_comments_and_directives_never_create_authored_bindings() {
    let allocator = Allocator::default();
    for text in ["", "/*keep*/", "'use strict';", "return;"] {
        let resolution = resolve_handler(input(&allocator, text), &Outer(&[])).unwrap();
        assert_eq!(resolution.bindings().len(), 1);
        assert!(resolution.declarations().is_empty());
        assert!(resolution.references().is_empty());
        assert_eq!(resolution.scopes()[0].span, None);
    }
}

mod profile;
mod refusal;
mod source;
