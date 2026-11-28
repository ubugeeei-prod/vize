//! One parse per SFC script block, shared by every rule and the Croquis
//! analysis that read it.

use std::cell::OnceCell;
use std::ops::Deref;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::ParserReturn;
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;
use vize_atelier_sfc::SfcDescriptor;
use vize_atelier_sfc::croquis::SfcScriptParses;

/// The descriptor's script parses, in the lint language.
pub(crate) type SfcScripts<'a> = SfcScriptParses<'a, 'a>;

/// Lint reads JSX in every block except `lang="ts"` (#9222).
fn lint_language(lang: Option<&str>) -> SourceType {
    crate::rules::script::script_source_type().with_jsx(matches!(
        lang,
        None | Some("js" | "javascript" | "jsx" | "tsx")
    ))
}

/// Each script block of `descriptor`, parsed in its lint language on first read.
pub(crate) fn lint_scripts<'a>(
    allocator: &'a Allocator,
    descriptor: &'a SfcDescriptor<'_>,
) -> SfcScripts<'a> {
    SfcScriptParses::new(allocator, descriptor, lint_language, None)
}

/// The shared parses, as a lint context holds them.
pub(crate) trait ScriptParses {
    /// The `<script>` block's parse.
    fn script(&self) -> Option<&ParserReturn<'_>>;
    /// The `<script setup>` block's parse.
    fn script_setup(&self) -> Option<&ParserReturn<'_>>;
    /// The node-building semantic pass over `program`, when `program` is one
    /// of these blocks' parses and the holder shares one per block.
    fn semantic_of(&self, program: &Program<'_>) -> Option<&SemanticBuilderReturn<'_>> {
        let _ = program;
        None
    }
}

impl ScriptParses for SfcScriptParses<'_, '_> {
    fn script(&self) -> Option<&ParserReturn<'_>> {
        SfcScriptParses::script(self)
    }

    fn script_setup(&self) -> Option<&ParserReturn<'_>> {
        SfcScriptParses::script_setup(self)
    }
}

/// A descriptor's script parses plus one `SemanticBuilder` pass per block,
/// run when a rule first asks for it, so the rules reading a block share it.
pub(crate) struct ScriptSemantics<'s, 'a> {
    parses: &'s SfcScripts<'a>,
    script: OnceCell<SemanticBuilderReturn<'s>>,
    script_setup: OnceCell<SemanticBuilderReturn<'s>>,
}

impl<'s, 'a> ScriptSemantics<'s, 'a> {
    pub(crate) fn new(parses: &'s SfcScripts<'a>) -> Self {
        Self {
            parses,
            script: OnceCell::new(),
            script_setup: OnceCell::new(),
        }
    }

    /// The parses these passes read.
    pub(crate) fn parses(&self) -> &'s SfcScripts<'a> {
        self.parses
    }
}

impl ScriptParses for ScriptSemantics<'_, '_> {
    fn script(&self) -> Option<&ParserReturn<'_>> {
        self.parses.script()
    }

    fn script_setup(&self) -> Option<&ParserReturn<'_>> {
        self.parses.script_setup()
    }

    fn semantic_of(&self, program: &Program<'_>) -> Option<&SemanticBuilderReturn<'_>> {
        // Only an already-read parse can be `program`; peeking never parses.
        let (cell, parsed) = [
            (&self.script_setup, self.parses.parsed_script_setup()),
            (&self.script, self.parses.parsed_script()),
        ]
        .into_iter()
        .find(|(_, parsed)| {
            parsed.is_some_and(|parsed| std::ptr::addr_eq(&parsed.program, program))
        })?;
        let parsed = parsed?;
        Some(cell.get_or_init(|| build(&parsed.program, true)))
    }
}

/// The `SemanticBuilder` pass a rule ran itself before blocks shared one.
pub(crate) fn build<'a>(program: &'a Program<'a>, nodes: bool) -> SemanticBuilderReturn<'a> {
    SemanticBuilder::new()
        .with_build_nodes(nodes)
        .build(program)
}

/// A block's shared semantic pass, or the rule's own when none is shared.
pub(crate) enum BlockSemantic<'s, 'a> {
    Shared(&'s SemanticBuilderReturn<'a>),
    Own(Box<SemanticBuilderReturn<'a>>),
}

impl<'s, 'a> BlockSemantic<'s, 'a> {
    /// `shared`, else a pass of its own over `program`, building nodes when
    /// `nodes`. Node ids, scopes, symbols and diagnostics do not depend on
    /// `nodes`, so a shared pass serves rules that skip them.
    pub(crate) fn of(
        shared: Option<&'s SemanticBuilderReturn<'a>>,
        program: &'a Program<'a>,
        nodes: bool,
    ) -> Self {
        shared.map_or_else(|| Self::Own(Box::new(build(program, nodes))), Self::Shared)
    }
}

impl<'a> Deref for BlockSemantic<'_, 'a> {
    type Target = SemanticBuilderReturn<'a>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Shared(built) => built,
            Self::Own(built) => built,
        }
    }
}

#[cfg(test)]
mod tests;
