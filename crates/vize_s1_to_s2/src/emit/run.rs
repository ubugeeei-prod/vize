//! The S2 → DOM emit driver: option validation, [`EmitCx`] construction,
//! preamble assembly and the render-function body.
//!
//! Split out of `emit.rs` so the module root stays the capability index
//! and the type declarations; nothing here is new behaviour.

use alloc::vec::Vec as StdVec;
use vize_davinci::diagnostic::Severity;
use vize_s2::op::Namespace;

use crate::lower::Lowered;
use crate::pass::S2Facts;
use crate::pass::walk::PageWalk;

use super::buf::Buf;
use super::fragment::emit_root;
use super::helper::Helper;
use super::{
    DomEmit, DomEmitOptions, DomEmitSections, EmitCx, EmitError, UnsupportedReason, cache_slots,
    component, directive, filter, fragment, helper_preference, prefix, static_cache,
};

/// Emit a DOM render function from an already-lowered (and typically
/// transformed) S2 artifact under the shipped default options. `facts`
/// is the transform product compounds compile from.
pub fn emit_dom(lowered: &Lowered<'_>, facts: &S2Facts) -> Result<DomEmit, EmitError> {
    emit_dom_with_options(lowered, facts, &DomEmitOptions::DEFAULT)
}

/// [`emit_dom`] under explicit [`DomEmitOptions`].
pub fn emit_dom_with_options<'f>(
    lowered: &'f Lowered<'_>,
    facts: &'f S2Facts,
    options: &DomEmitOptions<'f>,
) -> Result<DomEmit, EmitError> {
    emit_dom_with_emit_budget(lowered, facts, options).map(|(emit, _)| emit)
}

pub(super) fn emit_dom_with_emit_budget<'f>(
    lowered: &'f Lowered<'_>,
    facts: &'f S2Facts,
    options: &DomEmitOptions<'f>,
) -> Result<(DomEmit, u32), EmitError> {
    if options.is_ts && !cfg!(feature = "typescript") {
        return Err(EmitError::unsupported(
            UnsupportedReason::TypeScriptLaneUnavailable,
        ));
    }
    if lowered
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(EmitError::Diagnostics);
    }
    // `static_cache = inline || !hoists.is_empty()`.
    let static_cache =
        options.inline || static_cache::enabled(&lowered.root, facts, &lowered.wrappers);
    let mut cx = EmitCx {
        buf: Buf::new(options.inline),
        source: lowered.source,
        facts,
        scopes: &lowered.scopes,
        wrappers: &lowered.wrappers,
        for_wrappers: &lowered.for_wrappers,
        walk: PageWalk::new(),
        scope_names: StdVec::new(),
        if_branch_key: 0,
        once_cache_index: 0,
        once_depth: 0,
        once_element_depth: 0,
        in_v_for: false,
        conditional_v_for_item: false,
        template_for_item_single_root: false,
        template_for_item_root_id: None,
        template_if_branch_root: false,
        template_if_for_branch_root: false,
        suppress_template_for_child_key: false,
        skip_memo: false,
        slot_param_depth: 0,
        hoist_static_vnodes: false,
        transition_slot_root: false,
        static_cache,
        parent_ns: Namespace::Html,
        prefix_identifiers: options.prefix_identifiers,
        is_ts: options.is_ts,
        cache_handlers: options.cache_handlers,
        hoisted_scope_id: options.hoisted_scope_id,
        scope_id: options.scope_id,
        skip_scope_id: false,
        cache_sites: StdVec::new(),
        used_unref: core::cell::Cell::new(u32::MAX),
        component_name: options.component_name,
        scope: prefix::PrefixScope::new(
            options.bindings,
            options.prefix_identifiers,
            options.is_ts,
            options.inline,
        ),
    };
    let filters = &facts.legacy.filters;
    if facts.legacy.filter_helper_precedes_components {
        cx.buf.prefer(Helper::ResolveFilter);
    }
    let mut helper_walk = PageWalk::new();
    let prefer_cx = helper_preference::PreferCx {
        facts,
        for_wrappers: &lowered.for_wrappers,
        bindings: options.bindings,
    };
    helper_preference::prefer_helpers(&mut cx.buf, &prefer_cx, &mut helper_walk, &lowered.root);
    fragment::prefer_root_fragment(&mut cx.buf, &lowered.root);
    cx.buf
        .push(options.mode.render_signature(options.bindings.is_some()));
    cx.buf.indent();
    cx.buf.newline();
    let assets_start = cx.buf.code.len();
    let names = component::collect_names(&lowered.root);
    let dirs = directive::collect_names(&lowered.root);
    let mut resolved_assets = false;
    if !names.is_empty() {
        resolved_assets |= component::emit_resolves(&mut cx, &names);
    }
    if !dirs.is_empty() {
        resolved_assets |= directive::emit_resolves(&mut cx, &dirs);
    }
    if !filters.is_empty() {
        filter::emit_resolves(&mut cx, filters);
        resolved_assets = true;
    }
    let mut assets_end = cx.buf.code.len();
    if resolved_assets {
        cx.buf.newline();
        assets_end = cx.buf.code.len();
    }
    cx.buf.push("return ");
    let return_expr_start = cx.buf.code.len();
    emit_root(&mut cx, &lowered.root)?;
    let return_expr_end = cx.buf.code.len();
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}");
    // `_unref` is a *transform* registration: it lists with the pre-walk's
    // preferred helpers, at the op whose expression needed it - ahead of a
    // structural helper that op registers later (`renderList`).
    let unref_visit = cx.used_unref.get();
    if unref_visit != u32::MAX {
        cx.buf.prefer_at_visit(Helper::Unref, unref_visit);
        cx.buf.use_helper(Helper::Unref);
    }
    cache_slots::renumber(&mut cx);
    let emit_visits = cx.walk.visits();
    let (preamble, imports_len) = cx.buf.preamble_with_imports_len(options);
    let code = cx.buf.code;
    Ok((
        DomEmit {
            preamble,
            code,
            sections: DomEmitSections {
                imports_len,
                assets_start,
                assets_end,
                return_expr_start,
                return_expr_end,
            },
        },
        emit_visits,
    ))
}
