//! Shared emitter context helpers kept out of `emit.rs` for the source budget.

use vize_davinci::id::NodeId;
use vize_s0::{String, ToCompactString};
use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::op::ForOp;

use super::js::RawJs;
use super::prefix::{self, ScopeMark, Site};
use super::{EmitCx, EmitError, UnsupportedReason as Reason};

/// Which text the shipped node held for an expression position.
#[derive(Clone, Copy)]
enum ContentShape {
    /// The quoted attribute value with its padding.
    Padded,
    /// The padded value, entity-decoded (bind values).
    Decoded,
    /// The trimmed source alone.
    Trimmed,
}

impl EmitCx<'_> {
    pub(super) fn scope_mark(&self) -> usize {
        self.scope_names.len()
    }

    pub(super) fn push_scope(&mut self, id: Option<NodeId>) -> usize {
        let mark = self.scope_mark();
        if let Some(facts) = id.and_then(|id| self.scopes.get(id)) {
            for binding in facts.bindings.iter() {
                self.scope_names.push(binding.name.clone());
            }
        }
        mark
    }

    pub(super) fn pop_scope(&mut self, mark: usize) {
        self.scope_names.truncate(mark);
    }

    /// `CodegenContext::is_slot_param`: the S2 scope facts' names plus the
    /// codegen's own `add_slot_params` list, which also carries the names a
    /// destructuring `v-for` alias or slot pattern binds.
    pub(super) fn is_scope_name(&self, source: &str) -> bool {
        self.scope_names.iter().any(|name| name.as_str() == source)
            || if self.prefix_identifiers {
                self.scope.is_slot_param(source)
            } else {
                self.scope.binds_in_pattern(source)
            }
    }

    pub(super) fn with_static_vnode_hoist<T>(
        &mut self,
        enabled: bool,
        write: impl FnOnce(&mut Self) -> Result<T, EmitError>,
    ) -> Result<T, EmitError> {
        let previous = self.hoist_static_vnodes;
        self.hoist_static_vnodes = previous || enabled;
        let result = write(self);
        self.hoist_static_vnodes = previous;
        result
    }

    pub(super) fn with_static_vnode_hoist_exact<T>(
        &mut self,
        enabled: bool,
        write: impl FnOnce(&mut Self) -> Result<T, EmitError>,
    ) -> Result<T, EmitError> {
        let previous = self.hoist_static_vnodes;
        self.hoist_static_vnodes = enabled;
        let result = write(self);
        self.hoist_static_vnodes = previous;
        result
    }

    pub(super) fn with_once_element<T>(
        &mut self,
        write: impl FnOnce(&mut Self) -> Result<T, EmitError>,
    ) -> Result<T, EmitError> {
        let previous = self.once_element_depth;
        if self.once_depth > 0 {
            self.once_element_depth = self.once_element_depth.saturating_add(1);
        }
        let result = write(self);
        self.once_element_depth = previous;
        result
    }

    // ---- `prefix_identifiers` (P2-11 installment 85) ----

    /// Whether the expression pipeline runs at all: the shipped
    /// `process_expression` returns early unless one of `prefix_identifiers`
    /// / `is_ts` is on, and every emit site that consults this mirrors it.
    /// Fold a prefixed result's `_unref` use into the emit and hand back
    /// its text.
    fn record_unref(&self, prefixed: prefix::Prefixed) -> String {
        if prefixed.used_unref && self.used_unref.get() == u32::MAX {
            self.used_unref.set(self.walk.visits());
        }
        prefixed.text
    }

    /// `is_constant_interpolation`: the interpolation reads a single
    /// binding the script cannot change. The name is only visible as
    /// itself where the transform leaves it bare — an inlined render
    /// function — so this is `false` everywhere else.
    pub(super) fn reads_constant_binding(&self, expr: &ExprRef<'_>) -> bool {
        let source = match expr {
            ExprRef::Js(js) => js.source,
            ExprRef::Opaque(opaque) => opaque.source,
            ExprRef::Foreign(_) | ExprRef::Filter(_) => return false,
        }
        .trim();
        self.scope.reads_constant_binding(source)
    }

    /// [`Self::reads_constant_binding`] over a name the caller already
    /// trimmed out of the op.
    pub(super) fn reads_constant_binding_name(&self, name: &str) -> bool {
        self.scope.reads_constant_binding(name)
    }

    /// Push a `_cache` slot number and record where its digits landed,
    /// so [`super::cache_slots::renumber`] can re-derive the numbering in
    /// printed order.
    pub(super) fn push_cache_index(&mut self, slot: u32) {
        let at = self.buf.code.len();
        self.push_cache_index_at(slot, at);
    }

    /// [`Self::push_cache_index`] for a construct whose slot number
    /// prints after its body (`withMemo`): the ordering key is where the
    /// construct began, which is where the shipped codegen took the slot.
    pub(super) fn push_cache_index_at(&mut self, slot: u32, order_key: usize) {
        self.cache_sites
            .push((self.buf.code.len(), order_key, slot));
        self.buf.push(slot.to_compact_string().as_str());
    }

    /// Push a captured slot body back into the buffer, re-registering the
    /// `_cache` sites it carries at their new offsets.
    pub(super) fn push_captured(&mut self, piece: &super::slots::SlotPiece) {
        let base = self.buf.code.len();
        self.buf.push(piece.as_str());
        for (offset, order_key, slot) in piece.sites().iter().copied() {
            self.cache_sites
                .push((base + offset, base + order_key, slot));
        }
    }

    pub(super) fn prefixing(&self) -> bool {
        self.prefix_identifiers || self.is_ts
    }

    /// `cache_handlers_in_current_scope`: handler caching is unsafe while
    /// template-scope params are in play, because a cached closure would
    /// capture the first scoped value.
    pub(super) fn caches_handlers(&self) -> bool {
        self.cache_handlers && !self.scope.has_slot_params()
    }

    /// `TransformContext::enter_v_for_scope` + `add_slot_params` for the
    /// callback params; pop with [`Self::leave_scope`]. The default lane
    /// records only the raw patterns (see [`PrefixScope`]).
    pub(super) fn enter_for_scope(&mut self, for_op: &ForOp<'_>) -> ScopeMark {
        let mark = self.scope.mark();
        let binding = &for_op.binding;
        let aliases = [
            Some(binding.value.source()),
            binding.key.as_ref().map(|expr| expr.source()),
            binding.index.as_ref().map(|expr| expr.source()),
        ];
        if self.prefix_identifiers {
            self.scope.push_for(aliases);
        } else {
            for alias in aliases.into_iter().flatten() {
                self.scope.push_pattern(alias);
            }
        }
        mark
    }

    /// `enter_v_slot_scope` + `add_slot_params` for a scoped slot.
    pub(super) fn enter_slot_scope(&mut self, params: Option<&str>) -> ScopeMark {
        let mark = self.scope.mark();
        if let Some(params) = params
            && !params.trim().is_empty()
        {
            if self.prefix_identifiers {
                self.scope.push_slot(params);
            } else {
                self.scope.push_pattern(params);
            }
        }
        mark
    }

    pub(super) fn leave_scope(&mut self, mark: ScopeMark) {
        self.scope.pop(mark);
    }

    /// The prefixed text of `expr` the way `site` consumes it.
    pub(super) fn prefixed_expr(
        &self,
        expr: &ExprRef<'_>,
        site: Site,
    ) -> Result<String, EmitError> {
        self.prefixed_expr_content(expr, site, ContentShape::Padded)
    }

    /// [`Self::prefixed_expr`] over the entity-decoded bind value.
    pub(super) fn prefixed_bind_expr(&self, expr: &ExprRef<'_>) -> Result<String, EmitError> {
        self.prefixed_expr_content(expr, Site::Expression, ContentShape::Decoded)
    }

    /// [`Self::prefixed_expr`] over the trimmed source alone (the
    /// transform trimmed it before the codegen saw it).
    pub(super) fn prefixed_trimmed_expr(
        &self,
        expr: &ExprRef<'_>,
        site: Site,
    ) -> Result<String, EmitError> {
        self.prefixed_expr_content(expr, site, ContentShape::Trimmed)
    }

    fn prefixed_expr_content(
        &self,
        expr: &ExprRef<'_>,
        site: Site,
        shape: ContentShape,
    ) -> Result<String, EmitError> {
        let (source, js) = match expr {
            ExprRef::Js(js) => (js.source, Some(*js)),
            ExprRef::Opaque(opaque) => (opaque.source, None),
            ExprRef::Foreign(_) | ExprRef::Filter(_) => {
                return Err(EmitError::unsupported_at(
                    Reason::PrefixExpressionKind,
                    expr.span(),
                ));
            }
        };
        let content = match shape {
            ContentShape::Padded => prefix::node_content(self.source, source, expr.span()),
            ContentShape::Decoded => prefix::node_content_decoded(self.source, source, expr.span()),
            ContentShape::Trimmed => prefix::Content {
                text: RawJs::Borrowed(source),
                offset: Some(0),
            },
        };
        prefix::prefix_expression(&self.scope, &content, js, site)
            .map(|prefixed| self.record_unref(prefixed))
            .map_err(|_| EmitError::unsupported_at(Reason::PrefixExpressionRejected, expr.span()))
    }

    /// The prefixed text of a retained expression the way `site` consumes it.
    pub(super) fn prefixed_js(&self, js: &JsExpr<'_>, site: Site) -> Result<String, EmitError> {
        let content = prefix::node_content(self.source, js.source, js.span);
        prefix::prefix_expression(&self.scope, &content, Some(js), site)
            .map(|prefixed| self.record_unref(prefixed))
            .map_err(|_| EmitError::unsupported_at(Reason::PrefixExpressionRejected, js.span))
    }

    /// [`Self::prefixed_js`] over the entity-decoded bind value.
    pub(super) fn prefixed_bind_js(&self, js: &JsExpr<'_>) -> Result<String, EmitError> {
        let content = prefix::node_content_decoded(self.source, js.source, js.span);
        prefix::prefix_expression(&self.scope, &content, Some(js), Site::Expression)
            .map(|prefixed| self.record_unref(prefixed))
            .map_err(|_| EmitError::unsupported_at(Reason::PrefixExpressionRejected, js.span))
    }

    pub(super) fn push_prefixed_js(
        &mut self,
        js: &JsExpr<'_>,
        site: Site,
    ) -> Result<(), EmitError> {
        let text = self.prefixed_js(js, site)?;
        self.buf.push(text.as_str());
        Ok(())
    }

    /// Prefixed text for a fact-derived string (no retained AST, no padding).
    pub(super) fn prefixed_text(&self, text: &str, site: Site) -> Result<String, EmitError> {
        let content = prefix::Content {
            text: RawJs::Borrowed(text),
            offset: None,
        };
        prefix::prefix_expression(&self.scope, &content, None, site)
            .map(|prefixed| self.record_unref(prefixed))
            .map_err(|_| EmitError::unsupported(Reason::PrefixExpressionRejected))
    }

    /// `process_inline_handler` + `generate_event_handler` over `expr`.
    pub(super) fn prefixed_handler(
        &self,
        expr: &ExprRef<'_>,
        for_caching: bool,
    ) -> Result<String, EmitError> {
        let (source, js) = match expr {
            ExprRef::Js(js) => (js.source, Some(*js)),
            ExprRef::Opaque(opaque) => (opaque.source, None),
            ExprRef::Foreign(_) | ExprRef::Filter(_) => {
                return Err(EmitError::unsupported_at(
                    Reason::PrefixExpressionKind,
                    expr.span(),
                ));
            }
        };
        let content = prefix::node_content(self.source, source, expr.span());
        prefix::prefix_handler(&self.scope, &content, js, for_caching)
            .map(|prefixed| self.record_unref(prefixed))
            .map_err(|_| EmitError::unsupported_at(Reason::PrefixExpressionRejected, expr.span()))
    }

    /// The handler text for synthesized handler source (`v-model` writes).
    pub(super) fn prefixed_handler_text(&self, text: &str) -> Result<String, EmitError> {
        let content = prefix::Content {
            text: RawJs::Borrowed(text),
            offset: None,
        };
        prefix::prefix_handler(&self.scope, &content, None, false)
            .map(|prefixed| self.record_unref(prefixed))
            .map_err(|_| EmitError::unsupported(Reason::PrefixExpressionRejected))
    }

    /// `emit_dynamic_directive_arg` under `prefix_identifiers`.
    pub(super) fn prefixed_dynamic_arg(&self, js: &JsExpr<'_>) -> String {
        prefix::prefix_dynamic_arg(&self.scope, js)
    }

    pub(super) fn push_prefixed_expr(
        &mut self,
        expr: &ExprRef<'_>,
        site: Site,
    ) -> Result<(), EmitError> {
        let text = self.prefixed_expr(expr, site)?;
        self.buf.push(text.as_str());
        Ok(())
    }
}

impl<'facts> EmitCx<'facts> {
    pub(super) fn scope_id_here(&self) -> Option<&'facts str> {
        (!self.skip_scope_id).then_some(self.scope_id).flatten()
    }
}
