use tower_lsp::lsp_types::{DocumentChangeOperation, DocumentChanges, OneOf, Range, WorkspaceEdit};
use vize_canon::LspLocation;
use vize_carton::{FxHashSet, String};

use crate::ide::{IdeContext, corsa_support};

struct ComponentPropEditFilter<'a, 'ctx> {
    ctx: &'a IdeContext<'ctx>,
    document: &'a corsa_support::CanonicalVirtualDocument,
    names: &'a FxHashSet<String>,
    authored_definitions: &'a [tower_lsp::lsp_types::Location],
    navigation_identities: &'a corsa_support::ComponentPropNavigationIdentities,
    source_cache: &'a mut corsa_support::ComponentPropSourceCache,
}

impl ComponentPropEditFilter<'_, '_> {
    fn matches(&mut self, uri: &tower_lsp::lsp_types::Url, range: Range) -> bool {
        let raw = LspLocation {
            uri: uri.to_string(),
            range: corsa_support::tower_range(range),
        };
        if !corsa_support::component_prop_navigation_identity_matches(
            self.document,
            &raw,
            self.authored_definitions,
            self.navigation_identities,
        ) {
            return false;
        }
        let Some(authored) =
            corsa_support::map_canonical_corsa_location(self.ctx, self.document, &raw)
        else {
            return false;
        };
        corsa_support::component_prop_location_matches(
            self.ctx,
            self.document,
            &authored,
            self.names,
            self.source_cache,
        )
    }
}

pub(super) fn retain_component_prop_edits(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    edit: &mut WorkspaceEdit,
    names: &FxHashSet<String>,
    authored_definitions: &[tower_lsp::lsp_types::Location],
    navigation_identities: &corsa_support::ComponentPropNavigationIdentities,
    source_cache: &mut corsa_support::ComponentPropSourceCache,
) {
    let mut filter = ComponentPropEditFilter {
        ctx,
        document,
        names,
        authored_definitions,
        navigation_identities,
        source_cache,
    };
    if let Some(changes) = edit.changes.as_mut() {
        changes.retain(|uri, edits| {
            edits.retain(|edit| filter.matches(uri, edit.range));
            !edits.is_empty()
        });
    }
    if let Some(document_changes) = edit.document_changes.as_mut() {
        match document_changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits.iter_mut() {
                    edit.edits.retain(|entry| {
                        filter.matches(&edit.text_document.uri, annotatable_range(entry))
                    });
                }
                edits.retain(|edit| !edit.edits.is_empty());
            }
            DocumentChanges::Operations(operations) => {
                for operation in operations.iter_mut() {
                    if let DocumentChangeOperation::Edit(edit) = operation {
                        edit.edits.retain(|entry| {
                            filter.matches(&edit.text_document.uri, annotatable_range(entry))
                        });
                    }
                }
                operations.retain(|operation| match operation {
                    DocumentChangeOperation::Edit(edit) => !edit.edits.is_empty(),
                    DocumentChangeOperation::Op(_) => true,
                });
            }
        }
    }
}

fn annotatable_range(
    edit: &OneOf<tower_lsp::lsp_types::TextEdit, tower_lsp::lsp_types::AnnotatedTextEdit>,
) -> Range {
    match edit {
        OneOf::Left(edit) => edit.range,
        OneOf::Right(edit) => edit.text_edit.range,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ops::Range as ByteRange;

    use tower_lsp::lsp_types::{Location, Position, TextEdit, Url, WorkspaceEdit};
    use vize_canon::virtual_ts::VizeSemanticLinkKind;

    use super::retain_component_prop_edits;
    use crate::ide::{DiagnosticService, IdeContext, corsa_support};
    use crate::server::ServerState;

    const SOURCE: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
import Other from "./Other.vue";
const greeting = "hello";
</script>
<template>
  <Child :title="greeting" />
  <Other :title="greeting" />
</template>
"#;

    #[test]
    fn same_named_navigation_edit_requires_the_resolved_component_identity() {
        let source_uri = Url::parse("file:///workspace/App.vue").expect("source URI");
        let request_uri = Url::parse("file:///workspace/App.vue.ts").expect("request URI");
        let virtual_result =
            DiagnosticService::generate_virtual_ts(&source_uri, SOURCE, false, false)
                .expect("virtual TS");
        let child_target = navigation_target(&virtual_result, "Child");
        let other_target = navigation_target(&virtual_result, "Other");
        let child_range = generated_range(&virtual_result.code, &child_target);
        let other_range = generated_range(&virtual_result.code, &other_target);
        let document = corsa_support::CanonicalVirtualDocument {
            source_uri: source_uri.clone(),
            request_uri: request_uri.to_string().into(),
            virtual_result,
            dependencies: Vec::new(),
            materialized_sources: Vec::new(),
            session_project_roots: Vec::new(),
        };
        let child_definition = Location::new(
            Url::parse("file:///workspace/Child.vue").expect("Child URI"),
            tower_lsp::lsp_types::Range::new(Position::new(1, 15), Position::new(1, 20)),
        );
        let other_definition = Location::new(
            Url::parse("file:///workspace/Other.vue").expect("Other URI"),
            tower_lsp::lsp_types::Range::new(Position::new(1, 15), Position::new(1, 20)),
        );
        let mut navigation_identities = corsa_support::ComponentPropNavigationIdentities::default();
        navigation_identities.insert(
            semantic_position(
                request_uri.as_str(),
                &document.virtual_result.code,
                child_target.start,
            ),
            vec![child_definition.clone()],
        );
        navigation_identities.insert(
            semantic_position(
                request_uri.as_str(),
                &document.virtual_result.code,
                other_target.start,
            ),
            vec![other_definition],
        );
        let mut names = vize_carton::FxHashSet::default();
        names.insert("title".into());
        let mut changes = HashMap::new();
        changes.insert(
            request_uri,
            vec![
                TextEdit {
                    range: child_range,
                    new_text: "renamedTitle".into(),
                },
                TextEdit {
                    range: other_range,
                    new_text: "renamedTitle".into(),
                },
            ],
        );
        let mut edit = WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        };
        let state = ServerState::new();
        let ctx = IdeContext::with_content(&state, &source_uri, 0, SOURCE.to_owned());
        let mut source_cache = corsa_support::ComponentPropSourceCache::default();

        retain_component_prop_edits(
            &ctx,
            &document,
            &mut edit,
            &names,
            &[child_definition],
            &navigation_identities,
            &mut source_cache,
        );

        let changes = edit.changes.expect("filtered changes");
        let edits = changes.values().next().expect("generated document edits");
        assert_eq!(edits.len(), 1, "Other.title must be rejected by identity");
        assert_eq!(edits[0].range, child_range);
    }

    fn navigation_target(
        result: &crate::ide::diagnostics::VirtualTsResult,
        component: &str,
    ) -> ByteRange<usize> {
        result
            .semantic_links
            .iter()
            .find(|link| {
                link.kind == VizeSemanticLinkKind::VueComponentPropNavigation
                    && result
                        .code
                        .get(link.source_range.clone())
                        .is_some_and(|source| source.contains(component))
                    && result.code.get(link.target_range.clone()) == Some("title")
            })
            .map(|link| link.target_range.clone())
            .unwrap_or_else(|| panic!("{component}.title navigation link"))
    }

    fn generated_range(code: &str, range: &ByteRange<usize>) -> tower_lsp::lsp_types::Range {
        let (start_line, start_character) = crate::ide::offset_to_position(code, range.start);
        let (end_line, end_character) = crate::ide::offset_to_position(code, range.end);
        tower_lsp::lsp_types::Range::new(
            Position::new(start_line, start_character),
            Position::new(end_line, end_character),
        )
    }

    fn semantic_position(
        request_uri: &str,
        code: &str,
        offset: usize,
    ) -> corsa_support::CanonicalSemanticPosition {
        let (line, character) = crate::ide::offset_to_position(code, offset);
        corsa_support::CanonicalSemanticPosition {
            request_uri: request_uri.into(),
            line,
            character,
        }
    }
}
