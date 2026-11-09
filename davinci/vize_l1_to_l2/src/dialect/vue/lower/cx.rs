// The lowering context: id minting, span recovery, and the three fact
// channels. Module documentation remains on the compatibility owner.

use alloc::vec::Vec;

use vize_l0::diag::{Advisory, Diagnostic, Stage};
use vize_l0::id::NodeId;
use vize_l0::side_table::SideTable;
use vize_l0::{Allocator, SourceBlock, Span, String};
use vize_l1::markup::{Element, ElementClose, Token};
use vize_l2::expr::ExprRef;
use vize_l2::provenance::{ProvenanceRecord, ProvenanceSink};
use vize_l2::scope::{ScopeFacts, ScopeTag};

use super::text::{Lane, TextLane, TextSource, TextWhitespace};

pub(crate) struct Cx<'a, P> {
    pub allocator: &'a Allocator,
    block: SourceBlock<'a>,
    pub(super) frozen_openings: super::frozen::FrozenOpenings<'a>,
    /// The complete authored source that L0 spans are measured against.
    pub source: &'a str,
    next_op: u32,
    exhausted: bool,
    next_scope: u32,
    /// How many `<pre>` ancestors the walk is inside; condensing is
    /// suppressed for those subtrees (`lower::text`).
    condense_depth: u32,
    /// The direct owner's effective text rule, including inherited `<pre>`.
    pub(crate) text_whitespace: TextWhitespace,
    /// Authored start of the first visible text child of an HTML pre/textarea.
    pub(crate) ignore_newline_at: Option<u32>,
    /// How many `v-pre` ancestors the walk is inside; interpolations are
    /// inert authored text for those subtrees.
    v_pre_depth: u32,
    /// Whether ordinary comments lower to `ui.comment`.
    preserve_comments: bool,
    /// Whether lint/editor consumers need ASTs for expressions folded into
    /// compound text ops. Compile-only paths reparse those parts in the
    /// emitter and must not pay for an earlier duplicate parse.
    pub(crate) retain_consumed_expressions: bool,
    pub diagnostics: Vec<Diagnostic>,
    /// Decisions, kept only when the consumer chose a recording sink.
    pub provenance: P,
    pub scopes: SideTable<ScopeFacts>,
    pub texts: SideTable<super::text::TextParts>,
    pub consumed_expressions: Vec<ExprRef<'a>>,
    pub for_facts: SideTable<super::forop::ForParts>,
    pub if_facts: SideTable<super::if_keys::IfFacts>,
    pub wrappers: SideTable<vize_l2::analysis::fragment_facts::BranchFragments>,
    pub for_wrappers: SideTable<vize_l2::analysis::fragment_facts::LoopFragment>,
    pub caps: super::caps::LegacyCaps,
    pub foreign_dialect: Option<super::foreign::ForeignDialect<'a>>,
    /// Op families this lowering built, set where the op is minted —
    /// see `lower::features`.
    pub features: vize_l2::artifact::LoweringFeatures,
    custom_element_patterns: Vec<String>,
    custom_element_predicate: Option<fn(&str) -> bool>,
}

impl<'a, P: ProvenanceSink> Cx<'a, P> {
    pub(crate) fn with_source_block_and_comment_policy(
        allocator: &'a Allocator,
        block: SourceBlock<'a>,
        caps: super::caps::LegacyCaps,
        policy: super::LowerPolicy,
        custom_element_patterns: &[String],
        custom_element_predicate: Option<fn(&str) -> bool>,
    ) -> Self
    where
        P: Default,
    {
        Self {
            allocator,
            block,
            frozen_openings: super::frozen::FrozenOpenings::Absent,
            source: block.root_source(),
            next_op: 0,
            exhausted: false,
            next_scope: 0,
            condense_depth: 0,
            text_whitespace: TextWhitespace::Normal,
            ignore_newline_at: None,
            v_pre_depth: 0,
            preserve_comments: policy.preserve_comments,
            retain_consumed_expressions: policy.retain_consumed_expressions,
            diagnostics: Vec::new(),
            provenance: P::default(),
            scopes: SideTable::new(),
            texts: SideTable::new(),
            consumed_expressions: Vec::new(),
            for_facts: SideTable::new(),
            if_facts: SideTable::new(),
            wrappers: SideTable::new(),
            for_wrappers: SideTable::new(),
            caps,
            foreign_dialect: None,
            features: vize_l2::artifact::LoweringFeatures::EMPTY,
            custom_element_patterns: custom_element_patterns.to_vec(),
            custom_element_predicate,
        }
    }

    pub(super) fn source_block(&self) -> SourceBlock<'a> {
        self.block
    }

    /// Record that this lowering built an op of `family`.
    ///
    /// Called where the op is minted, never from a rule name: a decision
    /// that *failed* still leaves an op behind (a malformed `v-for` keeps
    /// its `ui.for` under the escape), and the planner must see it.
    pub(crate) fn observe(&mut self, family: vize_l2::artifact::OpFamily) {
        self.features = self.features.observing(family);
    }

    /// The text rules of the child list being lowered.
    pub(crate) fn text_lane(&self) -> Lane<'a> {
        Lane {
            src: TextSource::new(self.allocator, self.block),
            rules: TextLane {
                preserve_comments: self.preserve_comments,
                whitespace: self.text_whitespace,
                ignore_newline_at: self.ignore_newline_at,
                in_pre: self.condense_depth > 0,
                frozen: self.v_pre_suppressed(),
            },
        }
    }

    /// Whether the walk is inside a `v-pre` subtree.
    pub(crate) fn v_pre_suppressed(&self) -> bool {
        self.v_pre_depth > 0
    }

    pub(crate) fn preserve_comments(&self) -> bool {
        self.preserve_comments
    }

    /// Enter/leave a condense-suppressing `<pre>` around its children.
    pub(crate) fn push_condense_suppression(&mut self) {
        self.condense_depth = self.condense_depth.saturating_add(1);
    }

    pub(crate) fn pop_condense_suppression(&mut self) {
        self.condense_depth = self.condense_depth.saturating_sub(1);
    }

    /// Enter/leave a `v-pre` subtree around its children.
    pub(crate) fn push_v_pre_suppression(&mut self) {
        self.v_pre_depth = self.v_pre_depth.saturating_add(1);
    }

    pub(crate) fn pop_v_pre_suppression(&mut self) {
        self.v_pre_depth = self.v_pre_depth.saturating_sub(1);
    }

    /// Attach a merged run's recorded parts to its compound op, when the
    /// op has an id (the `attach_scope` exhaustion rule).
    pub(crate) fn attach_texts(
        &mut self,
        node: Option<NodeId>,
        parts: super::text::TextParts,
        span: Span,
        source: &str,
    ) {
        self.observe(vize_l2::artifact::OpFamily::TextCompound);
        if let Some(id) = node {
            parts.assert_compound_laws(id, span, source);
            self.record_text(
                "lower.text-fact",
                node,
                || vize_l0::cstr!("parts={}", parts.parts.len()),
                || {
                    let dynamic = parts.dynamic_count();
                    vize_l0::cstr!(
                        "fact static={} dynamic={dynamic}",
                        parts.parts.len() - dynamic
                    )
                },
                span,
            );
            self.texts.insert(id, parts);
        }
    }

    /// Attach captured wrapper keys to their `ui.if` op, when the op has
    /// an id (the `attach_scope` exhaustion rule).
    pub(crate) fn attach_wrappers(
        &mut self,
        node: Option<NodeId>,
        keys: vize_l2::analysis::fragment_facts::BranchFragments,
    ) {
        if let Some(id) = node {
            self.wrappers.insert(id, keys);
        }
    }

    /// Attach a `<template v-for>` unwrap fact to its `ui.for` op.
    pub(crate) fn attach_for_wrapper(
        &mut self,
        node: Option<NodeId>,
        wrapper: vize_l2::analysis::fragment_facts::LoopFragment,
    ) {
        if let Some(id) = node {
            self.for_wrappers.insert(id, wrapper);
        }
    }

    /// The next page-order op id, or `None` after exhaustion (reported
    /// once, as a diagnostic).
    pub(crate) fn mint_op(&mut self) -> Option<NodeId> {
        if self.exhausted {
            return None;
        }
        match NodeId::from_index(self.next_op) {
            Some(id) => {
                self.next_op += 1;
                Some(id)
            }
            None => {
                self.exhausted = true;
                self.error(
                    Span::new(0, 0),
                    String::from(
                        "the artifact exhausted its node ids; facts past this point are not keyed",
                    ),
                );
                None
            }
        }
    }

    pub(crate) fn op_count(&self) -> u32 {
        self.next_op
    }

    pub(crate) fn mint_scope(&mut self) -> ScopeTag {
        let tag = ScopeTag::from_index(self.next_scope);
        self.next_scope = self.next_scope.saturating_add(1);
        tag
    }

    /// Attach scope facts to an op, when the op has an id (after id
    /// exhaustion the tree stays total and only the keying stops).
    pub(crate) fn attach_scope(&mut self, node: Option<NodeId>, facts: ScopeFacts) {
        if let Some(id) = node {
            self.scopes.insert(id, facts);
        }
    }

    /// Byte offset of `slice` inside the authored source. Every L1 string
    /// is a slice of the one source (`vize_l1`'s P1-10 contract), so
    /// this is pointer arithmetic, never a search.
    pub(crate) fn offset(&self, slice: &str) -> u32 {
        super::text::block_offset(self.block, slice)
    }

    pub(crate) fn span_of(&self, slice: &str) -> Span {
        super::text::block_span(self.block, slice)
    }

    /// The span of a token's own text (`leading` excluded; a `Missing`
    /// token is zero-width at the offset where its syntax belongs).
    pub(crate) fn token_span(&self, token: &Token<'a>) -> Span {
        self.span_of(token.text)
    }

    /// A zero-width **source** slice at `offset` (snapped into range and
    /// onto a char boundary), for positions whose syntax was never
    /// authored — an expression hole still needs a slice the span
    /// machinery can locate.
    pub(crate) fn hole_at(&self, offset: u32) -> &'a str {
        self.block.zero_width_at(offset)
    }

    pub(crate) fn error(&mut self, span: Span, message: String) {
        self.diagnostics
            .push(crate::exemptions::lowering(span, &message));
    }

    /// Render an L1 typed hole into the unified channel. The tokenizer
    /// never reports a missing end tag (end-tag matching is tree
    /// construction, not lexing), so the `ElementClose::Missing` hole
    /// becomes a diagnostic exactly here — the P2-7 hand-off's second
    /// half. One chokepoint per lowered element; the exact relief
    /// `MissingEndTag` message, pointing at the open tag.
    pub(crate) fn report_missing_close(&mut self, element: &Element<'a>) {
        if matches!(element.close, ElementClose::Missing) {
            let span = self.token_span(&element.open.lt_name);
            self.diagnostics.push(Diagnostic::legacy_error(
                &crate::exemptions::MISSING_END_TAG,
                Stage::Surface,
                span,
                String::from("Element is missing end tag."),
            ));
        }
    }

    pub(crate) fn info(&mut self, span: Span, message: String) {
        self.diagnostics.push(Diagnostic::new(
            Advisory::Info,
            Stage::Semantic,
            span,
            message,
        ));
    }

    /// One provenance record, in decision order (the survival law:
    /// `node: None` + empty `after` is a decision that produced nothing).
    /// `after` is formatted only when the sink keeps records.
    pub(crate) fn record(
        &mut self,
        rule: &'static str,
        node: Option<NodeId>,
        before: &str,
        after: impl FnOnce() -> String,
        span: Span,
    ) {
        self.record_text(rule, node, || String::from(before), after, span);
    }

    /// [`Self::record`] for a decision whose `before` is not authored text.
    fn record_text(
        &mut self,
        rule: &'static str,
        node: Option<NodeId>,
        before: impl FnOnce() -> String,
        after: impl FnOnce() -> String,
        span: Span,
    ) {
        if let Some(records) = self.provenance.kept() {
            let after = after();
            // The span excludes `after` formatting: it counts rule/before
            // ownership and record-vector allocation traffic.
            #[cfg(feature = "davinci-benchmark-profile")]
            let _guard = super::benchmark::record_guard();
            records.push(ProvenanceRecord {
                rule: String::from(rule),
                node,
                before: before(),
                after,
                span,
            });
        }
    }
}

pub(crate) use span::{attr_slice, attr_span, element_span};
