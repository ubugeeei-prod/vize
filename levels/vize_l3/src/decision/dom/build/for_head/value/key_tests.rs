use super::*;
use crate::decision::dom::{DomChild, DomChildren, DomUnsupported, LiteralExpressions};
use crate::decision::native::build_native_selected_setup_dom_decisions;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
use vize_l2::{
    file::{DeclarationKind, NativeFileInterpolationState},
    lang::js::NativeSelectedSetup,
    scope::ScopeOrigin,
};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

struct Reads<'view, 'owner, 'arena>(&'view NativeSelectedSetup<'owner, 'arena>);
impl<'owner, 'arena> FileReads<'owner, 'arena> for Reads<'_, 'owner, 'arena> {
    const RECORD: bool = true;
    const NATIVE_SETUP: bool = true;
    const ORIGINAL_FOR: bool = true;

    fn classify(
        &self,
        occurrence: &Occurrence<'arena>,
        binding: BindingRef<'owner, 'arena>,
    ) -> Option<VueReadKind> {
        if occurrence.usage != Usage::Read {
            return None;
        }
        let declaration = self.0.binding(binding).ok()?.declaration()?;
        match declaration.kind {
            DeclarationKind::Const if declaration.initializer.is_primitive() => {
                Some(VueReadKind::SetupConst)
            }
            DeclarationKind::Let | DeclarationKind::Var => Some(VueReadKind::SetupLet),
            _ => None,
        }
    }
}

#[test]
fn actual_key_role_keeps_its_original_parameter_scope_occurrence_and_canonical_row() {
    for (source, value_name, key_name) in [
        (
            "<script setup>let count=2</script><template><i v-for='(item,key) in count'>{{key}}</i></template>",
            "item",
            "key",
        ),
        (
            "<script setup>const count=2</script><template><i v-for='(key,item) in count'>{{item}}</i></template>",
            "key",
            "item",
        ),
        (
            "<script setup>let count=2</script><template><i v-for='(item,count) in count'>{{count}}</i></template>",
            "item",
            "count",
        ),
        (
            "\r\n<script setup>const count='雪🌸'</script>\r\n<template><i v-for='(café,clé) in count'>{{clé}}</i></template>\r\n",
            "café",
            "clé",
        ),
    ] {
        let arena = Allocator::default();
        let observation = lower_selected_setup_sfc_native(&arena, source, options());
        let view = observation.admitted().unwrap();
        let setup = view.setup();
        let file = setup.file();
        let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
            panic!("one authentic original For");
        };
        let analysis = build_native_selected_setup_dom_decisions(setup).unwrap();
        let dom = analysis.dom().unwrap();
        assert!(dom.unsupported().is_empty(), "{source}");
        assert!(dom.node(original.id().node()).unwrap().block_eligible);
        let head = dom.file_for_head(original.id().node()).unwrap();
        let key = head.head().key().unwrap();
        let declaration = head.resolution().key_declaration().unwrap();
        let retained = key.template_declaration().unwrap();
        let retained_declaration = retained.declaration().original().unwrap();
        assert_eq!(declaration.fact().role(), ForAliasRole::Key);
        assert_eq!(declaration.fact().name(), key_name);
        assert_eq!(retained.declaration().id(), key.id());
        assert_eq!(retained.declaration().origin(), original.id());
        assert_eq!(retained.declaration().scope(), head.head().scope().unwrap());
        assert!(core::ptr::eq(retained.file(), file));
        assert!(core::ptr::eq(
            retained_declaration.parameter(),
            declaration.parameter()
        ));
        assert!(core::ptr::eq(
            retained_declaration.pattern(),
            declaration.pattern()
        ));
        assert!(core::ptr::eq(
            retained_declaration.resolution(),
            head.resolution()
        ));
        let [value_fact, key_fact] = head.scope_facts().bindings.as_slice() else {
            panic!("exactly two canonical bindings");
        };
        assert_eq!(value_fact.name.as_str(), value_name);
        assert_eq!(key_fact.name.as_str(), key_name);
        assert_eq!(
            key_fact.origin,
            ScopeOrigin::Authored {
                span: declaration.authored_span()
            }
        );
        assert_eq!(declaration.authored_span().slice(source), key_name);
        assert!(core::ptr::eq(
            head.scope_facts(),
            head.head().scope_facts().unwrap()
        ));
        assert_ne!(key.id(), head.head().value().unwrap().id());
        assert_ne!(key.id(), head.collection().id());
        assert!(key.declaration().is_none());
        assert!(setup.binding(key).is_err());
        let [record] = file.native_interpolations() else {
            panic!("one original interpolation");
        };
        let NativeFileInterpolationState::Admitted(node) = record.state() else {
            panic!("actual canonical interpolation");
        };
        let expression = analysis.expression(node).unwrap();
        let [read] = expression.reads() else {
            panic!("one actual callback read");
        };
        assert_eq!(read.kind(), VueReadKind::ForKey);
        assert_eq!(read.binding().id(), key.id());
        assert!(read.binding().same_owner(key));
        assert_eq!(expression.resolution().scope(), head.head().scope());
        let [occurrence] = expression.resolution().table().unwrap().occurrences() else {
            panic!("one retained resolver occurrence");
        };
        assert!(core::ptr::eq(read.occurrence(), occurrence));
        assert_eq!(occurrence.binding, key.id());
        assert_eq!(occurrence.name, key_name);
        assert_eq!(occurrence.usage, Usage::Read);
    }
}

#[test]
fn key_receipt_refuses_equal_foreign_files_copied_occurrences_value_roles_and_foreign_scopes() {
    let arena = Allocator::default();
    let source = "<script setup>let count=2</script><template><i v-for='(item,count) in count'>{{count}}</i></template>";
    let observation = lower_selected_setup_sfc_native(&arena, source, options());
    let other = lower_selected_setup_sfc_native(&arena, source, options());
    let view = observation.admitted().unwrap();
    let foreign_view = other.admitted().unwrap();
    let setup = view.setup();
    let file = setup.file();
    let foreign = foreign_view.setup().file();
    let [root @ Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("For");
    };
    let [body @ Op::Element(_)] = original.region.ops.as_slice() else {
        panic!("direct Element");
    };
    let complete = build_native_selected_setup_dom_decisions(setup).unwrap();
    let dom = complete.dom().unwrap();
    let DomChildren::Array(children) = &dom.node(original.id().node()).unwrap().children else {
        panic!("carrier");
    };
    let [DomChild::Node(body_node)] = children.as_slice() else {
        panic!("one body");
    };
    let [record] = file.native_interpolations() else {
        panic!("one interpolation");
    };
    let NativeFileInterpolationState::Admitted(node) = record.state() else {
        panic!("attached");
    };
    let resolution = file.expression(node).unwrap();
    let expression = resolution.table().unwrap().expression();
    let [occurrence] = resolution.table().unwrap().occurrences() else {
        panic!("one occurrence");
    };
    let binding = resolution.binding(occurrence.binding).unwrap();
    let row = dom.file_for_head(original.id().node()).unwrap();
    let foreign_resolution = foreign.expression(node).unwrap();
    let foreign_binding = foreign_resolution.binding(occurrence.binding).unwrap();
    assert_eq!(foreign_binding.id(), binding.id());
    let reads = Reads(setup);
    let mut builder = DomBuilder::new(&LiteralExpressions, 1, Some(file), &reads);
    builder
        .enter(original.id().node(), root, None, &mut None)
        .unwrap();
    builder.enter(*body_node, body, None, &mut None).unwrap();
    assert_eq!(
        builder.for_alias(resolution, expression, occurrence, binding),
        Some(VueReadKind::ForKey)
    );
    assert_eq!(
        builder.for_alias(foreign_resolution, expression, occurrence, binding),
        None
    );
    assert_eq!(
        builder.for_alias(resolution, expression, occurrence, foreign_binding),
        None
    );
    assert_eq!(
        builder.for_alias(
            resolution,
            expression,
            occurrence,
            row.head().value().unwrap()
        ),
        None
    );
    assert_eq!(
        builder.for_alias(resolution, expression, occurrence, row.collection()),
        None
    );
    let copied = *occurrence;
    assert_eq!(
        builder.for_alias(resolution, expression, &copied, binding),
        None
    );
    let key_declaration = row.resolution().key_declaration().unwrap();
    assert!(!super::super::alias(
        Some(binding),
        &key_declaration,
        file,
        row.head().id(),
        row.head().enclosing_scope().unwrap()
    ));
    assert!(!super::super::canonical_alias(
        row.scope_facts().bindings.first().unwrap(),
        &key_declaration
    ));
    let [Op::OriginalFor(foreign_original)] = foreign.artifact().root().ops.as_slice() else {
        panic!("foreign original");
    };
    let foreign_scope = foreign
        .for_head_for(foreign_original)
        .unwrap()
        .scope_facts()
        .unwrap();
    assert_eq!(foreign_scope, row.scope_facts());
    assert!(!core::ptr::eq(foreign_scope, row.scope_facts()));
    let mut detached = super::super::join(file, original.id().node(), original).unwrap();
    detached.scope = foreign_scope;
    builder
        .facts
        .file_for_heads
        .insert(original.id().node(), detached);
    assert_eq!(
        builder.for_alias(resolution, expression, occurrence, binding),
        None
    );
}

mod refusal;
