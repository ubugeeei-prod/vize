//! Assignability and late original-header/root refusals retain honest custody.

use super::*;

#[test]
fn const_global_import_nested_and_type_namespace_are_not_mutable_setup_model_targets() {
    for (source, unresolved, declaration) in [
        (
            "<script setup>const draft=''</script><template><input v-model='draft'/></template>",
            false,
            Some((DeclarationKind::Const, Namespace::Value, true)),
        ),
        (
            "<script setup>let other=''</script><template><input v-model='Math'/></template>",
            true,
            None,
        ),
        (
            "<script setup>import draft from 'outside'</script><template><input v-model='draft'/></template>",
            true,
            Some((DeclarationKind::Import, Namespace::Value, true)),
        ),
        (
            "<script setup>function scope(){let draft='nested';return draft}let other=0</script><template><input v-model='draft'/></template>",
            true,
            Some((DeclarationKind::Let, Namespace::Value, false)),
        ),
        (
            "<script setup lang='ts'>import type { Draft as draft } from 'outside';let other=''</script><template><input v-model='draft'/></template>",
            true,
            Some((DeclarationKind::Import, Namespace::Type, true)),
        ),
    ] {
        let arena = Allocator::default();
        let mut owner = owner(&arena, source);
        let error;
        {
            let mut walk = owner.begin().unwrap();
            let child = walk.selected().children().next().unwrap();
            error = walk.child(child).unwrap_err().kind;
            assert_eq!(walk.complete().unwrap_err().kind, error);
        }
        let output = owner.finish();
        assert!(output.retained_setup().unwrap().program().is_some());
        assert!(output.view().is_err() && output.setup().is_err());
        let file = output.file().unwrap();
        assert!(!file.is_complete());
        let [record] = file.native_models() else {
            panic!("whole original Model before target refusal")
        };
        assert_eq!(record.state(), NativeFileModelState::Refused(error));
        assert!(record.input().expression().is_some());
        assert!(record.input().attribute_span().is_some());
        assert!(
            record.binding().is_none()
                && record.original.is_none()
                && record.element_node().is_none()
        );
        assert!(core::ptr::eq(
            record.input().operand().syntax().source().authored_root(),
            source
        ));
        if unresolved {
            let Kind::Artifact(crate::artifact::ArtifactError::InvalidSpan { node, span }) = error
            else {
                panic!("actual unresolved source occurrence: {error:?}")
            };
            assert!(node.is_none());
            assert_eq!(span, record.input().operand().value_span());
            assert!(
                file.template_issues()
                    .iter()
                    .any(|issue| issue.kind == crate::file::FileIssueKind::UnresolvedReference)
            );
        } else {
            assert_eq!(error, Kind::UnsupportedChild);
        }
        if let Some((kind, namespace, direct)) = declaration {
            let actual = file
                .bindings()
                .find_map(|binding| {
                    binding
                        .declaration()
                        .filter(|declaration| declaration.name.as_str() == "draft")
                })
                .unwrap();
            assert_eq!(actual.kind, kind);
            assert_eq!(actual.namespace, namespace);
            assert_eq!(actual.is_direct_program(), direct);
            if direct {
                assert_eq!(actual.scope, record.scope());
            } else {
                assert_ne!(actual.scope, record.scope());
            }
        }
    }
}

#[test]
fn actual_root_geometry_and_late_header_failures_keep_inputs_without_publishing_any_model() {
    for (template, pending, rows) in [
        ("<div v-model='draft'/>", false, 1),
        ("<input v-model='draft'/><b/>", false, 1),
        ("<b/><input v-model='draft'/>", false, 1),
        ("<div><input v-model='draft'/></div>", false, 1),
        ("<input v-model='draft' disabled/>", false, 1),
        ("<input v-model='draft' type='checkbox'/>", false, 1),
        ("<input v-model='draft' :class='draft'/>", true, 1),
        ("<input v-model='draft' v-model='draft'/>", true, 2),
    ] {
        let arena = Allocator::default();
        let source =
            alloc::format!("<script setup>let draft=''</script><template>{template}</template>");
        let mut owner = owner(&arena, &source);
        {
            let mut walk = owner.begin().unwrap();
            let mut refused = false;
            for child in walk.selected().children() {
                if let Err(error) = walk.child(child) {
                    assert_eq!(error.kind, Kind::UnsupportedChild, "{template}");
                    refused = true;
                    break;
                }
            }
            assert!(refused, "{template}");
            assert_eq!(walk.complete().unwrap_err().kind, Kind::UnsupportedChild);
        }
        let output = owner.finish();
        assert!(output.view().is_err() && output.setup().is_err());
        let file = output.file().unwrap();
        assert!(!file.is_complete());
        assert_eq!(file.native_models().len(), rows);
        for record in file.native_models() {
            assert_eq!(
                record.state(),
                if pending {
                    NativeFileModelState::Pending
                } else {
                    NativeFileModelState::Refused(Kind::UnsupportedChild)
                }
            );
            assert_eq!(record.input().operand().raw_value(), "draft");
            assert!(record.input().expression().is_some());
            assert!(core::ptr::eq(
                record.input().operand().syntax().source().authored_root(),
                source.as_str()
            ));
            assert!(
                record.binding().is_none()
                    && record.original.is_none()
                    && record.element_node().is_none()
            );
        }
    }
}
