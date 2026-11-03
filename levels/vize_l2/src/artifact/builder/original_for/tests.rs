//! Real original resolutions enter the canonical factory without File ID casts.

extern crate std;

use crate::artifact::{Artifact, ArtifactError, Builder};
use crate::lang::js::NativeForInput;
use crate::resolution::{BindingId, BindingLookup, ForResolution, resolve_for_head};
use crate::scope::{ScopeOrigin, ScopeTag};
use vize_l0::{Allocator, Span, String, id::NodeId};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::{NativeTemplateComponent, SurfaceParseOptions},
};

struct Enclosing;

impl BindingLookup for Enclosing {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        (name == "items").then_some(BindingId::new(0))
    }
}

fn original<'a>(arena: &'a Allocator, source: &'a str) -> (ForResolution<'a>, Span) {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: vize_l0::config::VueVersion::V3,
            dialect: vize_l0::config::VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let element = selected.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    let input =
        NativeForInput::new(selected.observe_attribute_for_head(attribute).unwrap()).unwrap();
    (
        resolve_for_head(input, &Enclosing).unwrap(),
        selected.component().block().span(),
    )
}

fn artifact<'a>(arena: &'a Allocator, source: &'a str) -> Artifact<'a> {
    let (head, span) = original(arena, source);
    let mut builder = Builder::new(arena, source).unwrap();
    let node = builder.begin_original_for(&head, span).unwrap();
    builder.finish_original_for(node).unwrap();
    builder.finish().unwrap()
}

#[test]
fn foreign_equal_source_and_invalid_owner_do_not_mint_nodes_or_scopes() {
    let arena = Allocator::default();
    let source = "<template><div v-for='item in items'/></template>";
    let foreign = String::from(source);
    assert!(!core::ptr::eq(source, foreign.as_str()));
    let (head, span) = original(&arena, source);
    let mut builder = Builder::new(&arena, foreign.as_str()).unwrap();
    assert_eq!(
        builder.begin_original_for(&head, span),
        Err(ArtifactError::MismatchedJsSource {
            node: NodeId::FIRST,
            span: head.input().operand().syntax().source().span(),
        })
    );
    assert_eq!(builder.walk.minted(), 0);
    assert!(builder.parts.scopes.is_empty());
    let (own, own_span) = original(&arena, foreign.as_str());
    let invalid_owner = own.input().operand().name_span();
    assert_eq!(
        builder.begin_original_for(&own, invalid_owner),
        Err(ArtifactError::OutsideOwner {
            node: NodeId::FIRST,
            span: own.input().operand().syntax().source().span(),
            owner: invalid_owner,
        })
    );
    assert_eq!(builder.walk.minted(), 0);
    assert!(builder.parts.scopes.is_empty());
    assert_eq!(
        builder.begin_original_for(&own, own_span).unwrap(),
        NodeId::FIRST
    );
    builder.finish_original_for(NodeId::FIRST).unwrap();
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), 1);
    assert_eq!(
        artifact.scopes().get(NodeId::FIRST).unwrap().tag,
        ScopeTag::from_index(0)
    );
}

#[test]
fn canonical_recheck_rejects_missing_scope_invalid_tags_and_detached_authored_origins() {
    let arena = Allocator::default();
    let source = "<template><div v-for='α in items'/></template>";
    let mut parts = artifact(&arena, source).into_parts();
    parts.scopes.remove(NodeId::FIRST).unwrap();
    assert_eq!(
        Artifact::try_new(parts).unwrap_err().error,
        ArtifactError::MissingScope {
            node: NodeId::FIRST
        }
    );

    let mut parts = artifact(&arena, source).into_parts();
    parts.scopes.get_mut(NodeId::FIRST).unwrap().tag = ScopeTag::from_index(1);
    assert_eq!(
        Artifact::try_new(parts).unwrap_err().error,
        ArtifactError::InvalidScopeTag {
            node: NodeId::FIRST,
            tag: ScopeTag::from_index(1)
        }
    );

    let mut parts = artifact(&arena, source).into_parts();
    let [crate::op::Op::OriginalFor(original)] = parts.root.ops.as_slice() else {
        panic!("original For")
    };
    let owner = original.span;
    parts
        .scopes
        .get_mut(NodeId::FIRST)
        .unwrap()
        .bindings
        .first_mut()
        .unwrap()
        .origin = ScopeOrigin::Authored {
        span: Span::new(0, 1),
    };
    assert_eq!(
        Artifact::try_new(parts).unwrap_err().error,
        ArtifactError::OutsideOwner {
            node: NodeId::FIRST,
            span: Span::new(0, 1),
            owner,
        }
    );

    let mut parts = artifact(&arena, source).into_parts();
    let start = source.find('α').unwrap() as u32;
    parts
        .scopes
        .get_mut(NodeId::FIRST)
        .unwrap()
        .bindings
        .first_mut()
        .unwrap()
        .origin = ScopeOrigin::Authored {
        span: Span::new(start + 1, start + 2),
    };
    assert_eq!(
        Artifact::try_new(parts).unwrap_err().error,
        ArtifactError::InvalidSpan {
            node: Some(NodeId::FIRST),
            span: Span::new(start + 1, start + 2),
        }
    );
}

#[test]
fn caught_original_owner_interruption_retains_scope_and_whole_rejected_node() {
    let arena = Allocator::default();
    let source = "<template><div v-for='(item,key) in items'/></template>";
    let (head, span) = original(&arena, source);
    let mut builder = Builder::new(&arena, source).unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            builder.begin_original_for(&head, span).unwrap();
            panic!("original owner callback interrupted");
        }))
        .is_err()
    );
    let rejected = builder.finish().unwrap_err();
    assert_eq!(
        rejected.error,
        ArtifactError::UnfinishedOwner {
            node: NodeId::FIRST
        }
    );
    let facts = rejected.parts.scopes.get(NodeId::FIRST).unwrap();
    assert_eq!(facts.tag, ScopeTag::from_index(0));
    assert_eq!(
        facts
            .bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<alloc::vec::Vec<_>>(),
        ["item", "key"]
    );
    let restored = Artifact::try_new(*rejected.parts).unwrap();
    assert_eq!(restored.node_count(), 1);
    assert_eq!(restored.scopes().len(), 1);
}
