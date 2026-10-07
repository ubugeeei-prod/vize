//! Vue's ordered raw-new versus processed-earlier branch-key comparison.
//!
//! Prefixing already visits each key expression. Keep its identifier-reference
//! observation even when a reference is local and needs no textual rewrite.

use alloc::vec::Vec as StdVec;

use vize_l0::id::NodeId;
use vize_l0::{String, ToCompactString};
use vize_l2::expr::{ExprRef, JsExpr};
use vize_l2::op::{BindingOp, IfBranch, IfOp, Op};

use crate::emit::create_slots_walk::is_slot_if;
use crate::emit::js::{RawJs, escape_js_string};
use crate::emit::prefix::{self, Content, Site};
use crate::emit::{EmitCx, EmitError, UnsupportedReason as Reason};
use crate::lower::{BranchKey, BranchKeyKind, IfFacts};

#[derive(Default)]
pub(super) struct BranchKeys {
    // Compound earlier expressions can never equal the new SimpleExpression.
    simple: StdVec<(bool, String)>,
    // Erased conditional-slot wrappers have no retained expression AST.
    unknown_dynamic: bool,
}

impl BranchKeys {
    pub(super) fn emit(
        &mut self,
        cx: &EmitCx<'_>,
        facts: Option<&IfFacts>,
        index: usize,
        branch: &IfBranch<'_>,
        allocated: u32,
    ) -> Result<String, EmitError> {
        let key = facts
            .and_then(|facts| facts.branches.get(index))
            .and_then(Option::as_ref);
        let Some(key) = key else {
            return Ok(allocated.to_compact_string());
        };
        match &key.kind {
            BranchKeyKind::Static(None) => Ok(allocated.to_compact_string()),
            BranchKeyKind::Static(Some(value)) => {
                self.check(cx, true, value.as_str())?;
                if cx.prefix_identifiers {
                    self.simple.push((true, value.clone()));
                }
                let mut out = String::from("\"");
                out.push_str(escape_js_string(value.as_str()).as_str());
                out.push('"');
                Ok(out)
            }
            BranchKeyKind::Dynamic { source, .. } if source.is_empty() => {
                Ok(allocated.to_compact_string())
            }
            BranchKeyKind::Dynamic { source, bind_index } => {
                self.check(cx, false, source.as_str())?;
                if !cx.prefixing() {
                    return Ok(source.clone());
                }
                self.prefix(
                    cx,
                    key,
                    retained_key(branch, *bind_index),
                    bind_index.is_none(),
                )
            }
        }
    }

    fn check(&self, cx: &EmitCx<'_>, attribute: bool, raw: &str) -> Result<(), EmitError> {
        if cx.prefix_identifiers
            && (self
                .simple
                .iter()
                .any(|(kind, text)| *kind == attribute && text.as_str() == raw)
                || (!attribute && self.unknown_dynamic))
        {
            return Err(EmitError::Diagnostics);
        }
        Ok(())
    }

    fn prefix(
        &mut self,
        cx: &EmitCx<'_>,
        key: &BranchKey,
        js: Option<&JsExpr<'_>>,
        erased_wrapper: bool,
    ) -> Result<String, EmitError> {
        let BranchKeyKind::Dynamic { source, .. } = &key.kind else {
            return Err(EmitError::Diagnostics);
        };
        // Normal wrapper keys already used a prefix parse here. Carrier keys
        // reuse their retained AST in that same call instead of reparsing it.
        let content = Content {
            text: RawJs::Borrowed(source.as_str()),
            offset: js.filter(|js| js.source == source.as_str()).map(|_| 0),
        };
        let prefixed = prefix::prefix_expression(&cx.scope, &content, js, Site::Raw)
            .map_err(|_| EmitError::unsupported_at(Reason::PrefixExpressionRejected, key.span))?;
        if erased_wrapper {
            // The wrapper disappeared before Vue's expression transform; its
            // comparison key stays raw even though codegen prefixes its value.
            self.simple.push((false, source.clone()));
        } else if prefix::is_simple_identifier(source.as_str())
            || !prefixed.has_identifier_references
        {
            self.simple.push((false, prefixed.text.clone()));
        }
        let text = cx.record_unref(prefixed);
        Ok(prefix::consume(&cx.scope, text, Site::Expression))
    }

    fn observe_slot(
        &mut self,
        cx: &EmitCx<'_>,
        key: &BranchKey,
        branch: &IfBranch<'_>,
    ) -> Result<(), EmitError> {
        let Some(raw) = key.collision_text() else {
            return Ok(());
        };
        match &key.kind {
            BranchKeyKind::Static(_) => {
                self.check(cx, true, raw)?;
                self.simple.push((true, String::from(raw)));
            }
            BranchKeyKind::Dynamic { bind_index, .. } => {
                self.check(cx, false, raw)?;
                if let Some(js) = retained_key(branch, *bind_index)
                    && js.source == raw
                    && !cx.is_ts
                    && prefix::retained_expression_is_js(js)
                {
                    // The named-slot key is not emitted. Its retained AST can
                    // still publish the existing rewrite observation, no parse.
                    let _ = self.prefix(cx, key, Some(js), false)?;
                } else {
                    // A mixed slot/wrapper chain cannot prove processed key
                    // shape without another parse. Refuse potential collisions.
                    if self.simple.iter().any(|(attribute, _)| !attribute) {
                        return Err(EmitError::Diagnostics);
                    }
                    self.unknown_dynamic = true;
                }
            }
        }
        Ok(())
    }
}

fn retained_key<'a>(branch: &IfBranch<'a>, index: Option<usize>) -> Option<&'a JsExpr<'a>> {
    let [op] = branch.region.ops.as_slice() else {
        return None;
    };
    let bindings = match op {
        Op::Element(element) => &element.bindings,
        Op::Component(component) => &component.bindings,
        Op::Slot(slot) => &slot.bindings,
        _ => return None,
    };
    let BindingOp::Bind(bind) = bindings.get(index?)? else {
        return None;
    };
    match bind.value.as_ref()? {
        ExprRef::Js(js) => Some(*js),
        _ => None,
    }
}

pub(in crate::emit) struct SlotKeys {
    enabled: bool,
    keys: BranchKeys,
}

impl SlotKeys {
    pub(in crate::emit) fn new(cx: &EmitCx<'_>, id: Option<NodeId>, if_op: &IfOp<'_>) -> Self {
        Self {
            // Non-slot wrappers already passed through normal branch emission
            // before createSlots captures their compatibility entry.
            enabled: cx.prefix_identifiers && is_slot_if(cx, id, if_op),
            keys: BranchKeys::default(),
        }
    }

    pub(in crate::emit) fn observe(
        &mut self,
        cx: &EmitCx<'_>,
        id: Option<NodeId>,
        index: usize,
        branch: &IfBranch<'_>,
    ) -> Result<(), EmitError> {
        if self.enabled
            && let Some(key) = id
                .and_then(|id| cx.facts.if_facts.get(id))
                .and_then(|facts| facts.branches.get(index))
                .and_then(Option::as_ref)
        {
            self.keys.observe_slot(cx, key, branch)?;
        }
        Ok(())
    }
}
