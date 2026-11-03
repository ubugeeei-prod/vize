//! Authenticate stored references; never look up or rewrite authored names.

use vize_l0::Span;
use vize_l2::resolution::{
    HandlerBindingRef, HandlerDeclarationKind, HandlerReference, HandlerResolution, HandlerScopeId,
    Usage,
};

pub(super) fn original(
    resolution: &HandlerResolution<'_>,
    index: usize,
    reference: &HandlerReference<'_>,
) -> bool {
    if !resolution
        .references()
        .get(index)
        .is_some_and(|actual| core::ptr::eq(actual, reference))
    {
        return false;
    }
    let HandlerBindingRef::Local(id) = reference.binding else {
        return false;
    };
    let Some(binding) = resolution
        .bindings()
        .get(id.index() as usize)
        .filter(|binding| binding.id == id && binding.name == reference.name)
    else {
        return false;
    };
    let source = resolution.input().operand().syntax().source();
    let plain = |span: Span| {
        source.text().get(span.start as usize..span.end as usize) == Some(binding.name)
            && source.authored_span(span).ok().and_then(|authored| {
                source
                    .authored_root()
                    .get(authored.start as usize..authored.end as usize)
            }) == Some(binding.name)
    };
    if !plain(reference.span) || !within(resolution, reference.scope, binding.scope) {
        return false;
    }
    if binding.kind == HandlerDeclarationKind::EventParameter {
        return id.index() == 0 && binding.scope.index() == 0 && binding.name == "$event";
    }
    if reference.usage != Usage::Read || reference.shorthand || reference.constructor {
        return false;
    }
    // Vue's pinned statement transform prefixes root/sibling-block reads. An
    // active original block declaration preserves this writer's spelling.
    // Declaration identity does not promise a successful read before lexical
    // initialization: the unchanged body still performs JavaScript's TDZ checks.
    resolution.declarations().iter().any(|declaration| {
        declaration.binding == id
            && declaration.name == binding.name
            && declaration.kind == binding.kind
            && resolution
                .scopes()
                .get(declaration.scope.index() as usize)
                .is_some_and(|scope| scope.id == declaration.scope && scope.span.is_some())
            && within(resolution, reference.scope, declaration.scope)
            && (binding.kind == HandlerDeclarationKind::Var || binding.scope == declaration.scope)
            && plain(declaration.span)
    })
}

fn within(
    resolution: &HandlerResolution<'_>,
    mut scope: HandlerScopeId,
    ancestor: HandlerScopeId,
) -> bool {
    for _ in 0..resolution.scopes().len() {
        let Some(row) = resolution
            .scopes()
            .get(scope.index() as usize)
            .filter(|row| row.id == scope)
        else {
            return false;
        };
        if scope == ancestor {
            return true;
        }
        let Some(parent) = row.parent.filter(|parent| parent.index() < scope.index()) else {
            return false;
        };
        scope = parent;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::{
        Allocator,
        config::{VueDialect, VueVersion},
    };
    use vize_l1::{
        container::{Vue, vue::DescriptorOptions},
        markup::{NativeTemplateComponent, SurfaceParseOptions},
    };
    use vize_l2::{
        lang::js::{NativeTemplateFile, NativeTemplateOwner},
        op::{BindingOp, Op},
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
        let selected = NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
            .unwrap()
            .unwrap();
        let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("owner"));
        {
            let mut walk = owner.begin().unwrap();
            for child in walk.selected().children() {
                walk.child(child).unwrap();
            }
            walk.complete().unwrap();
        }
        owner.finish()
    }

    fn resolution<'o, 'a>(owner: &'o NativeTemplateFile<'a>) -> &'o HandlerResolution<'a> {
        let file = owner.file().unwrap();
        let Some(Op::Element(element)) = file.artifact().root().ops.first() else {
            panic!("actual element")
        };
        let Some(BindingOp::On(on)) = element.bindings.first() else {
            panic!("actual On")
        };
        file.handler_for(on).unwrap().resolution().unwrap()
    }

    #[test]
    fn exact_active_block_declarations_preserve_original_local_identities() {
        for body in [
            "let u;{const x=1;$event.c=x;}",
            "let u;{let x=2;$event.c=x;}",
            "let u;{var x=3;$event.c=x;}",
            "let u;{$event.c=x;let x=1;}",
            "let x;{let x=2;$event.c=x;}",
            "let u;{$event.c=x;var x=3;}",
            "let u;{var x=1,x=2;$event.c=x;}",
            "let u;{let x=2;{$event.c=x;}}",
            "let u;{let 雪=&quot;🌸&quot;;$event.c=雪;}",
        ] {
            let arena = Allocator::default();
            let source =
                alloc::format!("<!--雪🌸-->\r\n<template><button @click='{body}'/></template>");
            let owner = completed(&arena, &source);
            let resolution = resolution(&owner);
            assert!(
                resolution
                    .references()
                    .iter()
                    .enumerate()
                    .all(|(index, reference)| { original(resolution, index, reference) }),
                "{body}"
            );
            let local = resolution
                .references()
                .iter()
                .find(|reference| reference.name != "$event")
                .unwrap();
            let HandlerBindingRef::Local(id) = local.binding else {
                panic!("original local")
            };
            let binding = resolution.bindings().get(id.index() as usize).unwrap();
            assert_eq!(binding.id, id);
            assert!(matches!(
                binding.kind,
                HandlerDeclarationKind::Var
                    | HandlerDeclarationKind::Let
                    | HandlerDeclarationKind::Const
            ));
            let source = resolution.input().operand().syntax().source();
            assert_eq!(
                source
                    .text()
                    .get(local.span.start as usize..local.span.end as usize),
                Some(local.name)
            );
            assert_eq!(
                source
                    .authored_span(local.span)
                    .unwrap()
                    .slice(source.authored_root()),
                local.name
            );
        }
    }

    #[test]
    fn root_and_cross_block_reads_keep_the_pinned_prefix_refusal() {
        for body in [
            "let x=1;$event.c=x;",
            "let x=1;{$event.c=x;}",
            "var u;{var x=1;}$event.c=x;",
            "let u;{var x=1;}{$event.c=x;}",
        ] {
            let arena = Allocator::default();
            let source = alloc::format!("<template><button @click='{body}'/></template>");
            let owner = completed(&arena, &source);
            let resolution = resolution(&owner);
            let (index, reference) = resolution
                .references()
                .iter()
                .enumerate()
                .find(|(_, reference)| reference.name == "x")
                .unwrap();
            assert!(!original(resolution, index, reference), "{body}");
            assert_eq!(resolution.input().operand().syntax().source().text(), body);
        }
    }

    #[test]
    fn copied_foreign_and_cross_scope_reference_facts_confer_no_authority() {
        let arena = Allocator::default();
        let source = "<template><button @click='let u;{let x=2;$event.c=x;}'/></template>";
        let owner = completed(&arena, source);
        let foreign = completed(&arena, source);
        let own = resolution(&owner);
        let other = resolution(&foreign);
        for (index, reference) in own.references().iter().enumerate() {
            assert!(original(own, index, reference));
            assert!(!original(
                own,
                index,
                other.references().get(index).unwrap()
            ));
            let copied = *reference;
            assert!(!original(own, index, &copied));
            let mut wrong_scope = copied;
            wrong_scope.scope = own.scopes().first().unwrap().id;
            assert!(!original(own, index, &wrong_scope));
            let mut wrong_binding = copied;
            wrong_binding.binding = own.references().first().unwrap().binding;
            assert!(!original(own, index, &wrong_binding));
        }
    }

    #[test]
    fn forward_lexical_identity_is_not_a_promise_of_initialization() {
        let arena = Allocator::default();
        let source = "<template><button @click='let u;{$event.c=x;let x=1;}'/></template>";
        let owner = completed(&arena, source);
        let resolution = resolution(&owner);
        let (index, reference) = resolution
            .references()
            .iter()
            .enumerate()
            .find(|(_, reference)| reference.name == "x")
            .unwrap();
        let HandlerBindingRef::Local(id) = reference.binding else {
            panic!("original let")
        };
        let declaration = resolution
            .declarations()
            .iter()
            .find(|declaration| declaration.binding == id)
            .unwrap();
        assert_eq!(declaration.kind, HandlerDeclarationKind::Let);
        assert!(reference.span.end < declaration.span.start);
        assert!(original(resolution, index, reference));
        assert_eq!(
            resolution.input().operand().syntax().source().text(),
            "let u;{$event.c=x;let x=1;}"
        );
    }

    #[test]
    fn genuine_local_write_update_shorthand_and_constructor_roles_stay_refused() {
        for body in [
            "let u;{let x=1;x=2;$event.c=x;}",
            "let u;{let x=1;x++;$event.c=x;}",
            "let u;{let x=1;$event.c={x};}",
            "let u;{let x=1;new x;}",
        ] {
            let arena = Allocator::default();
            let source = alloc::format!("<template><button @click='{body}'/></template>");
            let owner = completed(&arena, &source);
            let resolution = resolution(&owner);
            let (index, reference) = resolution
                .references()
                .iter()
                .enumerate()
                .find(|(_, reference)| {
                    reference.name == "x"
                        && (reference.usage != Usage::Read
                            || reference.shorthand
                            || reference.constructor)
                })
                .unwrap();
            assert!(!original(resolution, index, reference), "{body}");
        }
    }
}
