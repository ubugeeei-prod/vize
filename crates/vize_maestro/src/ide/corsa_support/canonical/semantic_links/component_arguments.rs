//! Authored property-key roles retained from resolved native endpoints.

use std::ops::Range as OffsetRange;

use oxc_ast::ast::{
    ComputedMemberExpression, Expression, IdentifierReference, StaticMemberExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, Location, OneOf, WorkspaceEdit,
};
use vize_canon::{LspLocation, LspPosition, LspRange};
use vize_l0::{FxHashMap, String};

use super::{CanonicalVirtualDocument, ComponentPropNavigationMatches};
use crate::ide::IdeContext;

impl ComponentPropNavigationMatches {
    /// Retain the authored geometry of each definition-verified property key.
    /// The same Vue shorthand token also projects a value binding; only the
    /// producer-owned navigation endpoint establishes its public-key role.
    pub(crate) fn authored_arguments(
        &self,
        ctx: &IdeContext<'_>,
        document: &CanonicalVirtualDocument,
    ) -> Option<Vec<Location>> {
        self.positions
            .iter()
            .map(|position| {
                let (_, result) = super::virtual_result(document, &position.request_uri)?;
                let offset = crate::ide::position_to_offset(
                    &result.code,
                    position.line,
                    position.character,
                )?;
                let link = result.semantic_links.iter().find(|link| {
                    link.kind
                        == vize_canon::virtual_ts::VizeSemanticLinkKind::VueComponentPropNavigation
                        && link.target_range.start == offset
                })?;
                let (end_line, end_character) =
                    crate::ide::offset_to_position(&result.code, link.target_range.end);
                super::super::map_canonical_corsa_location(
                    ctx,
                    document,
                    &LspLocation {
                        uri: position.request_uri.to_string(),
                        range: LspRange {
                            start: LspPosition {
                                line: position.line,
                                character: position.character,
                            },
                            end: LspPosition {
                                line: end_line,
                                character: end_character,
                            },
                        },
                    },
                )
            })
            .collect()
    }
}

impl CanonicalVirtualDocument {
    /// Preserve positive value provenance before native key/value edits at a
    /// shorthand's one authored token are deduplicated. Contextual object/type
    /// keys do not become values just because they lack a navigation link.
    /// Only scope-checked native edits at definition-verified public arguments
    /// are considered; this discovers no new rename locations or symbols.
    pub(crate) fn selected_shorthand_value_arguments(
        &self,
        ctx: &IdeContext<'_>,
        edit: &WorkspaceEdit,
        property_arguments: &[Location],
        cache: &mut FxHashMap<String, Option<Vec<OffsetRange<usize>>>>,
    ) -> Option<Vec<Location>> {
        if property_arguments.is_empty() {
            return Some(Vec::new());
        }
        let mut values = Vec::new();
        let mut valid = true;
        let mut selected = |uri: &tower_lsp::lsp_types::Url, range| {
            let location = LspLocation {
                uri: uri.to_string(),
                range: super::tower_range(range),
            };
            let Some(authored) = super::super::map_canonical_corsa_location(ctx, self, &location)
            else {
                return;
            };
            let Some(argument) = property_arguments.iter().find(|argument| {
                argument.uri == authored.uri
                    && argument.range.start <= authored.range.start
                    && authored.range.start <= authored.range.end
                    && authored.range.end <= argument.range.end
            }) else {
                return;
            };
            let Some((request_uri, result)) = super::virtual_result(self, &location.uri) else {
                valid = false;
                return;
            };
            let Some(start) = crate::ide::position_to_offset(
                &result.code,
                location.range.start.line,
                location.range.start.character,
            ) else {
                valid = false;
                return;
            };
            let Some(end) = crate::ide::position_to_offset(
                &result.code,
                location.range.end.line,
                location.range.end.character,
            ) else {
                valid = false;
                return;
            };
            if result.semantic_links.iter().any(|link| {
                link.kind
                    == vize_canon::virtual_ts::VizeSemanticLinkKind::VueComponentPropNavigation
                    && start < link.target_range.end
                    && link.target_range.start < end
            }) {
                return;
            }
            let ranges = cache.entry(request_uri.clone()).or_insert_with(|| {
                value_expression_ranges(&result.code, request_uri.ends_with(".tsx"))
            });
            let Some(ranges) = ranges else {
                valid = false;
                return;
            };
            if ranges
                .iter()
                .any(|value| value.start <= start && end <= value.end && start < end)
            {
                values.push(argument.clone());
            }
        };
        if let Some(changes) = &edit.changes {
            for (uri, edits) in changes {
                for edit in edits {
                    selected(uri, edit.range);
                }
            }
        }
        if let Some(changes) = &edit.document_changes {
            let mut document_edit = |edit: &tower_lsp::lsp_types::TextDocumentEdit| {
                for entry in &edit.edits {
                    let range = match entry {
                        OneOf::Left(edit) => edit.range,
                        OneOf::Right(edit) => edit.text_edit.range,
                    };
                    selected(&edit.text_document.uri, range);
                }
            };
            match changes {
                DocumentChanges::Edits(edits) => {
                    for edit in edits {
                        document_edit(edit);
                    }
                }
                DocumentChanges::Operations(operations) => {
                    for operation in operations {
                        if let DocumentChangeOperation::Edit(edit) = operation {
                            document_edit(edit);
                        }
                    }
                }
            }
        }
        valid.then_some(values)
    }
}

fn value_expression_ranges(source: &str, tsx: bool) -> Option<Vec<OffsetRange<usize>>> {
    let allocator = oxc_allocator::Allocator::default();
    let source_type = if tsx {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let mut values = ValueExpressions::default();
    values.visit_program(&parsed.program);
    Some(values.ranges)
}

#[derive(Default)]
struct ValueExpressions {
    ranges: Vec<OffsetRange<usize>>,
}

impl<'a> Visit<'a> for ValueExpressions {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.ranges
            .push(identifier.span.start as usize..identifier.span.end as usize);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        self.ranges
            .push(member.property.span.start as usize..member.property.span.end as usize);
        walk::walk_static_member_expression(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if let Expression::StringLiteral(literal) = &member.expression {
            self.ranges
                .push(literal.span.start as usize..literal.span.end as usize);
        }
        walk::walk_computed_member_expression(self, member);
    }
}

#[cfg(test)]
#[path = "component_arguments_tests.rs"]
mod tests;
