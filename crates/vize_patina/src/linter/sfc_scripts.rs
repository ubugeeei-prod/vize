//! One parse per SFC script block, shared by every rule that reads it.

use std::cell::OnceCell;

use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use vize_atelier_sfc::{SfcDescriptor, SfcScriptBlock};
use vize_l0::profile;

/// The lint language of a script block: JSX except under `lang="ts"` (#9222).
pub(crate) fn script_block_source_type(lang: Option<&str>) -> SourceType {
    crate::rules::script::script_source_type().with_jsx(matches!(
        lang,
        None | Some("js" | "javascript" | "jsx" | "tsx")
    ))
}

/// The descriptor's script blocks, each parsed at most once, on first read.
pub(crate) struct SfcScripts<'a> {
    allocator: &'a Allocator,
    script: Option<Block<'a>>,
    script_setup: Option<Block<'a>>,
}

struct Block<'a> {
    source: &'a str,
    source_type: SourceType,
    parsed: OnceCell<ParserReturn<'a>>,
}

impl<'a> Block<'a> {
    fn new(block: &'a SfcScriptBlock<'_>) -> Self {
        Self {
            source: block.content.as_ref(),
            source_type: script_block_source_type(block.lang.as_deref()),
            parsed: OnceCell::new(),
        }
    }
}

/// The shared parses, as a lint context holds them.
pub(crate) trait ScriptParses {
    /// The `<script>` block's parse.
    fn script(&self) -> Option<&ParserReturn<'_>>;
    /// The `<script setup>` block's parse.
    fn script_setup(&self) -> Option<&ParserReturn<'_>>;
}

impl ScriptParses for SfcScripts<'_> {
    fn script(&self) -> Option<&ParserReturn<'_>> {
        SfcScripts::script(self)
    }

    fn script_setup(&self) -> Option<&ParserReturn<'_>> {
        SfcScripts::script_setup(self)
    }
}

impl<'a> SfcScripts<'a> {
    pub(crate) fn new(allocator: &'a Allocator, descriptor: &'a SfcDescriptor<'_>) -> Self {
        Self {
            allocator,
            script: descriptor.script.as_ref().map(Block::new),
            script_setup: descriptor.script_setup.as_ref().map(Block::new),
        }
    }

    /// The `<script>` block's parse.
    pub(crate) fn script(&self) -> Option<&ParserReturn<'a>> {
        self.script.as_ref().map(|block| self.parse(block))
    }

    /// The `<script setup>` block's parse.
    pub(crate) fn script_setup(&self) -> Option<&ParserReturn<'a>> {
        self.script_setup.as_ref().map(|block| self.parse(block))
    }

    fn parse<'s>(&'s self, block: &'s Block<'a>) -> &'s ParserReturn<'a> {
        block.parsed.get_or_init(|| {
            profile!(
                "patina.script_rule.parse",
                Parser::new(self.allocator, block.source, block.source_type).parse()
            )
        })
    }
}
