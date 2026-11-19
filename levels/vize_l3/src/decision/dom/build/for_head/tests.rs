use super::*;
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
use vize_l2::{
    lang::js::{NativeTemplateFile, NativeTemplateOwner},
    op::Op,
};

fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
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
    let script = admitted.setup().unwrap();
    let syntax = parse_program_once(
        arena,
        EmbedSource::authored(source, script.block().span()).unwrap(),
        ProgramOptions::module(script.lang()),
    );
    let selected = NativeTemplateComponent::parse_in(arena, admitted)
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("owner"));
    owner
        .setup_program(syntax.admitted_program().unwrap())
        .unwrap();
    {
        let mut walk = owner.begin().unwrap();
        for child in walk.selected().children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    owner.finish()
}

#[test]
fn private_join_refuses_foreign_file_and_wrong_canonical_node() {
    let arena = Allocator::default();
    let source =
        "<script setup>const items=2;</script><template><div v-for='item in items'/></template>";
    let owner = completed(&arena, source);
    let foreign_owner = completed(&arena, source);
    let file = owner.file().unwrap();
    let foreign_file = foreign_owner.file().unwrap();
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("For")
    };
    let [Op::OriginalFor(foreign)] = foreign_file.artifact().root().ops.as_slice() else {
        panic!("foreign For")
    };
    let node = original.id().node();
    assert_eq!(node, foreign.id().node());
    assert!(join(file, node, original).is_ok());
    assert!(matches!(
        join(foreign_file, node, original),
        Err(DomUnsupported::FileForHead)
    ));
    assert!(matches!(
        join(file, NodeId::from_index(1).unwrap(), original),
        Err(DomUnsupported::FileForHead)
    ));
    let head = file.for_head_for(original).unwrap();
    let foreign_head = foreign_file.for_head_for(foreign).unwrap();
    let expected = head.resolution().unwrap().value_declaration();
    assert!(!alias(
        foreign_head.value(),
        &expected,
        file,
        head.id(),
        head.scope().unwrap()
    ));
    assert!(!alias(
        head.value(),
        &expected,
        file,
        head.id(),
        head.enclosing_scope().unwrap()
    ));
}

#[test]
fn canonical_scope_stays_with_its_file_even_when_foreign_facts_are_equal() {
    let arena = Allocator::default();
    let source =
        "<script setup>const items=2;</script><template><div v-for='item in items'/></template>";
    let owner = completed(&arena, source);
    let foreign_owner = completed(&arena, source);
    let file = owner.file().unwrap();
    let foreign_file = foreign_owner.file().unwrap();
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("For")
    };
    let [Op::OriginalFor(foreign)] = foreign_file.artifact().root().ops.as_slice() else {
        panic!("foreign For")
    };
    let node = original.id().node();
    let mut row = join(file, node, original).unwrap();
    let own_scope = file.artifact().scopes().get(node).unwrap();
    let foreign_scope = foreign_file
        .artifact()
        .scopes()
        .get(foreign.id().node())
        .unwrap();
    assert!(core::ptr::eq(row.scope_facts(), own_scope));
    assert_eq!(own_scope, foreign_scope);
    assert!(!core::ptr::eq(own_scope, foreign_scope));
    assert_eq!(own_scope.tag.index(), 0);
    assert_ne!(own_scope.tag.index(), row.head().scope().unwrap().index());
    assert!(row.accepts_original(original));
    row.scope = foreign_scope;
    assert!(!row.accepts_original(original));
}

#[test]
fn canonical_alias_rejects_changed_name_range_and_synthesized_origin() {
    let arena = Allocator::default();
    let source =
        "<script setup>const items=2;</script><template><div v-for='item in items'/></template>";
    let owner = completed(&arena, source);
    let file = owner.file().unwrap();
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("For")
    };
    let head = file.for_head_for(original).unwrap();
    let declaration = head.resolution().unwrap().value_declaration();
    let binding = head.scope_facts().unwrap().bindings.first().unwrap();
    assert!(canonical_alias(binding, &declaration));
    let mut wrong_name = binding.clone();
    wrong_name.name = "other".into();
    assert!(!canonical_alias(&wrong_name, &declaration));
    let mut wrong_span = binding.clone();
    wrong_span.origin = ScopeOrigin::Authored {
        span: vize_l0::Span::new(0, 1),
    };
    assert!(!canonical_alias(&wrong_span, &declaration));
    let mut synthesized = binding.clone();
    synthesized.origin = ScopeOrigin::Synthesized {
        rule: "test.foreign".into(),
    };
    assert!(!canonical_alias(&synthesized, &declaration));
}

#[test]
fn canonical_binding_order_keeps_original_unicode_value_and_key_ranges() {
    let arena = Allocator::default();
    let source = "<script setup>const items=2;</script><template><div v-for='(café,index) in items'/></template>";
    let owner = completed(&arena, source);
    let file = owner.file().unwrap();
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("For")
    };
    let row = join(file, original.id().node(), original).unwrap();
    let [value, key] = row.scope_facts().bindings.as_slice() else {
        panic!("two original declarations")
    };
    assert_eq!(value.name.as_str(), "café");
    assert_eq!(key.name.as_str(), "index");
    for binding in [value, key] {
        let ScopeOrigin::Authored { span } = binding.origin else {
            panic!("authored declaration")
        };
        assert_eq!(span.slice(source), binding.name.as_str());
    }
}
