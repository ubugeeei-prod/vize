//! Code lens provider.
//!
//! Provides code lenses for:
//! - Script setup bindings (usage count)
//! - Component references
#![expect(
    clippy::disallowed_methods,
    reason = "tower-lsp command fields take std String values"
)]
//! - Event handler references

use crate::server::ServerState;

#[cfg(test)]
mod resident_tests;

mod legacy;

use tower_lsp::lsp_types::{CodeLens, Command, Position, Range, Url};

/// Code lens service.
pub struct CodeLensService;

impl CodeLensService {
    /// Read the shared, revision-bound authored facts. No request-time parsing.
    pub fn get_lenses(state: &ServerState, content: &str, uri: &Url) -> Vec<CodeLens> {
        use vize_croquis::binding_occurrences::OccurrenceBlock;
        let Some(packet) = state.binding_occurrence_facts(uri, content) else {
            return Vec::new();
        };
        if packet.is_legacy() {
            return legacy::CodeLensService::get_lenses(state, content, uri);
        }
        let Some(facts) = packet.authored() else {
            return Vec::new();
        };
        let mut bindings: Vec<_> = facts
            .bindings
            .values()
            .filter(|binding| binding.lens && binding.identity.block == OccurrenceBlock::Script)
            .collect();
        bindings.sort_unstable_by_key(|binding| (binding.lens_group, binding.identity.start));
        bindings
            .into_iter()
            .filter_map(|binding| {
                let count = facts.template_style_count(binding.identity);
                if count == 0 {
                    return None;
                }
                let line =
                    crate::utils::offset_to_position_str(content, binding.identity.start as usize)
                        .line;
                Some(CodeLens {
                    range: Range {
                        start: Position::new(line, 0),
                        end: Position::new(line, 0),
                    },
                    #[expect(
                        clippy::disallowed_macros,
                        reason = "the existing lsp_types command title is std String"
                    )]
                    command: Some(Command {
                        title: format!(
                            "{} template/style reference{}",
                            count,
                            if count == 1 { "" } else { "s" }
                        ),
                        command: "vize.findReferences".to_string(),
                        arguments: None,
                    }),
                    data: None,
                })
            })
            .collect()
    }
}
