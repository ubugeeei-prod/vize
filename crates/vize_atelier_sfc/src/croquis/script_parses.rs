//! One parse per SFC script block, shared by every analysis that reads it.

use std::cell::OnceCell;

use oxc_allocator::Allocator;
use oxc_parser::ParserReturn;
use oxc_span::SourceType;
use vize_carton::profile;
use vize_croquis::script_parser::parse_original_for_analysis;

use crate::{SfcDescriptor, SfcScriptBlock};

/// A descriptor's script blocks, each parsed at most once, on first read, in
/// the language its consumer reads it as.
pub struct SfcScriptParses<'a> {
    descriptor: &'a SfcDescriptor<'a>,
    allocator: &'a Allocator,
    script: Option<Block<'a>>,
    script_setup: Option<Block<'a>>,
}

struct Block<'a> {
    source: &'a str,
    source_type: SourceType,
    parsed: OnceCell<ParserReturn<'a>>,
}

impl<'a> SfcScriptParses<'a> {
    /// Parse each block of `descriptor` as `language(lang)` when first read.
    pub fn new(
        allocator: &'a Allocator,
        descriptor: &'a SfcDescriptor<'a>,
        language: fn(Option<&str>) -> SourceType,
    ) -> Self {
        let block = |block: &'a SfcScriptBlock<'_>| Block {
            source: block.content.as_ref(),
            source_type: language(block.lang.as_deref()),
            parsed: OnceCell::new(),
        };
        Self {
            descriptor,
            allocator,
            script: descriptor.script.as_ref().map(block),
            script_setup: descriptor.script_setup.as_ref().map(block),
        }
    }

    /// The language Vue compiles a script block as: JSX only under
    /// `lang="jsx"` or `lang="tsx"`.
    pub fn compile_language(lang: Option<&str>) -> SourceType {
        let path = if matches!(lang.map(str::trim), Some("tsx" | "jsx")) {
            "script.tsx"
        } else {
            "script.ts"
        };
        SourceType::from_path(path).unwrap_or_default()
    }

    /// The descriptor these blocks belong to.
    pub fn descriptor(&self) -> &'a SfcDescriptor<'a> {
        self.descriptor
    }

    /// The `<script>` block's authored parse.
    pub fn script(&self) -> Option<&ParserReturn<'a>> {
        self.script.as_ref().map(|block| self.parse(block))
    }

    /// The `<script setup>` block's authored parse.
    pub fn script_setup(&self) -> Option<&ParserReturn<'a>> {
        self.script_setup.as_ref().map(|block| self.parse(block))
    }

    fn parse<'s>(&'s self, block: &'s Block<'a>) -> &'s ParserReturn<'a> {
        block.parsed.get_or_init(|| {
            profile!(
                "atelier.sfc.script_parse",
                parse_original_for_analysis(self.allocator, block.source, block.source_type)
            )
        })
    }
}
