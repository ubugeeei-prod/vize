//! One emit-resolver declaration per static component.
//!
//! The alias is an instance factory: a vue-tsc library export is invoked as
//! itself, and every other component goes through `$EmitPropsFactory`. Neither
//! arm reads that use's props. A `v-else-if` chain closes the previous branch
//! before the next one, so the declaration is written in the template function
//! and each use only aliases it. The per-use emit-props call stays put.

use std::cell::RefCell;

use vize_carton::{FxHashMap, String, cstr};

struct InlineResolver {
    name: String,
}

struct State {
    /// `(component, parent scope, guard block)` of a declaration already in that block.
    inline: FxHashMap<String, InlineResolver>,
    /// Static component key → template-function declaration name.
    hoisted: FxHashMap<String, String>,
    pending: Vec<String>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        inline: FxHashMap::default(),
        hoisted: FxHashMap::default(),
        pending: Vec::new(),
    });
}

pub(super) struct ResolverShare<'a> {
    pub(super) component_type_name: &'a str,
    pub(super) component_ref: &'a str,
    pub(super) factory: &'a str,
    pub(super) dynamic: bool,
    pub(super) parent_scope: Option<u32>,
    pub(super) block: Option<u64>,
}

pub(crate) fn reset() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.inline.clear();
        state.hoisted.clear();
        state.pending.clear();
    });
}

pub(crate) fn flush(ts: &mut String) {
    STATE.with(|state| {
        let pending = std::mem::take(&mut state.borrow_mut().pending);
        for line in &pending {
            ts.push_str(line.as_str());
        }
    });
}

/// `Some` when this use should alias a declaration that is still in scope.
pub(super) fn alias_target(share: &ResolverShare<'_>) -> Option<String> {
    let key = component_key(share);
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(inline) = state
            .inline
            .get(&inline_key(&key, share.parent_scope, share.block))
        {
            return Some(inline.name.clone());
        }
        if share.dynamic {
            return None;
        }
        if share.block.is_some()
            && let Some(inline) = state
                .inline
                .get(&inline_key(&key, share.parent_scope, None))
        {
            return Some(inline.name.clone());
        }
        if let Some(name) = state.hoisted.get(&key) {
            return Some(name.clone());
        }
        if share.block.is_none() {
            return None;
        }
        let name = fresh_name(share.component_type_name, &state.hoisted);
        state.pending.push(declaration(
            name.as_str(),
            share.component_ref,
            share.factory,
        ));
        state.hoisted.insert(key, name.clone());
        Some(name)
    })
}

pub(super) fn record_inline(share: &ResolverShare<'_>, name: &str) {
    let key = inline_key(&component_key(share), share.parent_scope, share.block);
    STATE.with(|state| {
        state.borrow_mut().inline.insert(
            key,
            InlineResolver {
                name: String::from(name),
            },
        );
    });
}

fn component_key(share: &ResolverShare<'_>) -> String {
    cstr!(
        "{}\0{}\0{}",
        share.component_type_name,
        share.component_ref,
        share.factory
    )
}

fn inline_key(key: &str, parent_scope: Option<u32>, block: Option<u64>) -> String {
    cstr!("{key}\0{parent_scope:?}\0{block:?}")
}

fn fresh_name(type_name: &str, hoisted: &FxHashMap<String, String>) -> String {
    let base = cstr!("__{type_name}_emit_resolver");
    if hoisted.values().all(|name| name != &base) {
        return base;
    }
    let mut nth = 2u32;
    loop {
        let name = cstr!("{base}_{nth}");
        if hoisted.values().all(|existing| existing != &name) {
            return name;
        }
        nth += 1;
    }
}

/// Same condition as the per-guard `__library` alias, spelled inline so the
/// template-function declaration does not name a block-local alias.
fn declaration(name: &str, component_ref: &str, factory: &str) -> String {
    cstr!(
        "  type {name} = (typeof {component_ref} extends {{ __vizeCheck: any }} | {{ readonly __vizeComponentMarker: true }} ? false : typeof {component_ref} extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false) extends true ? (component: typeof {component_ref}) => typeof {component_ref} : {factory}<typeof {component_ref}>;\n"
    )
}
