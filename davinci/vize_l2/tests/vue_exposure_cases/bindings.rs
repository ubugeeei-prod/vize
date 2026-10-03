use super::*;
use vize_l2::file::{DeclarationKind, Namespace};

#[test]
fn only_real_direct_program_const_let_and_var_bindings_supply_the_bounded_view() {
    let arena = Allocator::default();
    let observed = Observed::new(
        &arena,
        "<script setup>let mutable; var count = 1; const fixed = 2; function compute(param) { let local = param; return local; }</script>",
    ).unwrap();
    let file = observed.file(&arena).unwrap();
    let view = observed.view(&file).unwrap().unwrap();
    for binding in file.bindings() {
        let declaration = binding.declaration().unwrap();
        let actual = view.binding(binding);
        if !declaration.is_direct_program() {
            assert!(
                matches!(actual, Err(issue) if issue.kind == ExposureIssueKind::DirectDeclaration)
            );
            assert_ne!(declaration.scope, view.scope());
        } else if matches!(
            declaration.kind,
            DeclarationKind::Const | DeclarationKind::Let | DeclarationKind::Var
        ) {
            assert!(actual.is_ok());
        } else {
            assert!(
                matches!(actual, Err(issue) if issue.kind == ExposureIssueKind::DeclarationKind)
            );
        }
    }
    assert_eq!(file.bindings().count(), 6);
}

#[test]
fn foreign_file_binding_handles_are_refused_even_with_matching_numeric_indices() {
    let arena = Allocator::default();
    let observed = Observed::new(&arena, "<script setup>let message = 1;</script>").unwrap();
    let first = observed.file(&arena).unwrap();
    let second = observed.file(&arena).unwrap();
    let view = observed.view(&first).unwrap().unwrap();
    let own = first.bindings().next().unwrap();
    let foreign = second.bindings().next().unwrap();
    assert_eq!(own.id(), foreign.id());
    assert!(view.binding(own).is_ok());
    assert!(
        matches!(view.binding(foreign), Err(issue) if issue.kind == ExposureIssueKind::ForeignBinding)
    );
}

#[test]
fn type_namespace_and_imports_remain_real_but_never_gain_runtime_let_access() {
    let arena = Allocator::default();
    let observed = Observed::new(
        &arena,
        "<script setup lang=ts>import type { Value } from 'types'; import { ref } from 'vue'; let message = 1;</script>",
    ).unwrap();
    let file = observed.file(&arena).unwrap();
    let view = observed.view(&file).unwrap().unwrap();
    for binding in file.bindings() {
        let declaration = binding.declaration().unwrap();
        let result = view.binding(binding);
        if declaration.namespace == Namespace::Type {
            assert!(matches!(result, Err(issue) if issue.kind == ExposureIssueKind::Namespace));
        } else if declaration.kind == DeclarationKind::Import {
            assert!(
                matches!(result, Err(issue) if issue.kind == ExposureIssueKind::DeclarationKind)
            );
        } else {
            assert!(result.is_ok());
        }
    }
}

#[test]
fn semantic_intrinsic_collisions_refuse_unused_and_escaped_declarations() {
    let arena = Allocator::default();
    for source in [
        "<script setup>let __proto__ = 1;</script>",
        "<script setup>let __isScriptSetup = 1;</script>",
        "<script setup>let __returned__ = 1;</script>",
        "<script setup>let __props = 1;</script>",
        "<script setup>let __expose = 1;</script>",
        r"<script setup>let \u005f_proto__ = 1;</script>",
    ] {
        let observed = Observed::new(&arena, source).unwrap();
        let file = observed.file(&arena).unwrap();
        assert!(file.is_complete());
        assert_eq!(
            kind(observed.view(&file).unwrap()).unwrap(),
            ExposureIssueKind::ReservedName
        );
    }
}

#[test]
fn ordinary_runtime_names_and_escaped_unicode_use_actual_semantic_identities() {
    let arena = Allocator::default();
    let observed = Observed::new(
        &arena,
        r"<script setup>let _ctx = 1, $setup = 2, _toDisplayString = 3, \u006dsg = 4, 日本語 = 5;</script>",
    ).unwrap();
    let file = observed.file(&arena).unwrap();
    let view = observed.view(&file).unwrap().unwrap();
    let names = file
        .bindings()
        .map(|binding| {
            assert!(view.binding(binding).is_ok());
            binding.declaration().unwrap().name.as_str()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        ["_ctx", "$setup", "_toDisplayString", "msg", "日本語"]
    );
}

#[test]
#[cfg(target_pointer_width = "64")]
fn new_inline_authority_fields_have_actual_measured_layout() {
    assert_eq!(core::mem::size_of::<vize_l2::file::ScriptUnit>(), 48);
    assert_eq!(core::mem::align_of::<vize_l2::file::ScriptUnit>(), 8);
}
