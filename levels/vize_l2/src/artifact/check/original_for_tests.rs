//! Revalidate the actual original receiver without fabricating neutral scopes.

use crate::lang::js::{NativeTemplateFile, NativeTemplateOwner};
use crate::op::Op;
use crate::scope::{ScopeBinding, ScopeFacts, ScopeOrigin, ScopeTag};
use crate::walk::{NodeEvent, NodeRef};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    embed::{
        EmbedSource,
        syntax::{ProgramOptions, parse_program_once},
    },
    markup::{NativeTemplateComponent, SurfaceParseOptions},
};

fn native<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor.admitted().unwrap();
    let setup = admitted.setup().unwrap();
    let syntax = parse_program_once(
        arena,
        EmbedSource::authored(source, setup.block().span()).unwrap(),
        ProgramOptions::module(setup.lang()),
    );
    let selected = NativeTemplateComponent::parse_in(arena, admitted)
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("actual owner"));
    owner
        .setup_program(syntax.admitted_program().unwrap())
        .unwrap();
    {
        let mut walk = owner.begin().unwrap();
        walk.child(walk.selected().children().next().unwrap())
            .unwrap();
        walk.complete().unwrap();
    }
    owner.finish()
}

#[test]
fn actual_original_for_tree_revalidates_with_file_owned_scope_custody() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='item in items'>body</div></template>";
    let output = native(&arena, source);
    let file = output.view().unwrap().file().unwrap();
    let [crate::op::Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("actual original For");
    };
    let head = file.for_head_for(original).unwrap();
    assert_ne!(head.scope(), head.enclosing_scope());
    assert_eq!(file.artifact().scopes().len(), 1);
    let facts = file.artifact().scopes().get(original.id().node()).unwrap();
    let start = source.find("item in items").unwrap() as u32;
    assert_eq!(
        facts,
        &ScopeFacts {
            tag: ScopeTag::from_index(0),
            bindings: alloc::vec![ScopeBinding {
                name: "item".into(),
                origin: ScopeOrigin::Authored {
                    span: vize_l0::Span::new(start, start + 4)
                },
            }],
        }
    );
    assert_ne!(facts.tag.index(), head.scope().unwrap().index());
    assert_eq!(
        super::check(&file.artifact().parts).unwrap(),
        file.artifact().node_count()
    );
}

#[test]
fn nested_original_for_tags_are_dense_canonical_sites_not_file_scope_ids() {
    let arena = Allocator::default();
    let source = "\r\n<script setup>const title='雪🌸';const items=2;</script>\r\n<template><div v-for='(café,key) in items'><span v-for='item in café'/></div></template>\r\n";
    let output = native(&arena, source);
    let file = output.view().unwrap().file().unwrap();
    let mut rows = alloc::vec::Vec::new();
    file.artifact()
        .visit_events(&mut |event| {
            if let NodeEvent::Enter {
                id,
                node: NodeRef::Op(Op::OriginalFor(original)),
                ..
            } = event
            {
                let head = file.for_head_for(original).unwrap();
                let facts = file.artifact().scopes().get(id).unwrap();
                assert_ne!(facts.tag.index(), head.scope().unwrap().index());
                let resolution = head.resolution().unwrap();
                let declarations = core::iter::once(resolution.value_declaration())
                    .chain(resolution.key_declaration());
                for (binding, declaration) in facts.bindings.iter().zip(declarations) {
                    assert_eq!(binding.name.as_str(), declaration.fact().name());
                    assert_eq!(
                        binding.origin,
                        ScopeOrigin::Authored {
                            span: declaration.authored_span()
                        }
                    );
                }
                rows.push((id.index(), facts.clone()));
            }
        })
        .unwrap();
    assert_eq!(rows.len(), 2);
    let [(_, outer), (inner_node, inner)] = rows.as_slice() else {
        panic!("two original For scopes")
    };
    assert_eq!(outer.tag, ScopeTag::from_index(0));
    assert_eq!(inner.tag, ScopeTag::from_index(1));
    assert_eq!(
        outer
            .bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<alloc::vec::Vec<_>>(),
        ["café", "key"]
    );
    assert_eq!(
        inner
            .bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<alloc::vec::Vec<_>>(),
        ["item"]
    );
    assert_ne!(*inner_node, inner.tag.index());
    let [value_binding, key_binding] = outer.bindings.as_slice() else {
        panic!("value and key")
    };
    let [inner_binding] = inner.bindings.as_slice() else {
        panic!("inner value")
    };
    let start = source.find("café,key").unwrap() as u32;
    assert_eq!(
        value_binding.origin,
        ScopeOrigin::Authored {
            span: vize_l0::Span::new(start, start + "café".len() as u32)
        }
    );
    let key = source.find("key) in").unwrap() as u32;
    assert_eq!(
        key_binding.origin,
        ScopeOrigin::Authored {
            span: vize_l0::Span::new(key, key + 3)
        }
    );
    let inner_start = source.find("item in café").unwrap() as u32;
    assert_eq!(
        inner_binding.origin,
        ScopeOrigin::Authored {
            span: vize_l0::Span::new(inner_start, inner_start + 4)
        }
    );
    assert_eq!(
        super::check(&file.artifact().parts).unwrap(),
        file.artifact().node_count()
    );
}
