//! The `visitType` walk behind [`super::count`].
use super::super::super::checker_memo::CheckerMemo;
use corsa::api::{NodeHandle, SymbolHandle, TypeHandle, TypeResponse};
use std::{future::Future, pin::Pin};
use vize_l0::{FxHashMap, FxHashSet};

// TypeScript 7 TypeFlags, from microsoft/typescript-go internal/checker/types.go.
const STRING: u32 = 1 << 5;
const NUMBER: u32 = 1 << 6;
const TYPE_PARAMETER: u32 = 1 << 19;
const OBJECT: u32 = 1 << 20;
const INDEX: u32 = 1 << 21;
const TEMPLATE_LITERAL: u32 = 1 << 22;
const STRING_MAPPING: u32 = 1 << 23;
const INDEXED_ACCESS: u32 = 1 << 25;
const CONDITIONAL: u32 = 1 << 26;
const UNION: u32 = 1 << 27;
const INTERSECTION: u32 = 1 << 28;
// ObjectFlags.
const CLASS: u32 = 1 << 0;
const INTERFACE: u32 = 1 << 1;
const REFERENCE: u32 = 1 << 2;
const TUPLE: u32 = 1 << 3;
const MAPPED: u32 = 1 << 5;
// SymbolFlags.TypeParameter: a declared type parameter, not a class's `this`.
const SYMBOL_TYPE_PARAMETER: u32 = 1 << 18;

/// Upstream stops descending into a type seen this many times in one root
/// (a recursive type).
const REPEAT_LIMIT: u32 = 9;
/// Checker requests one owner may spend before its counts are given up.
const REQUEST_BUDGET: usize = 400;

mod objects;

type Boxed<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub(super) struct Walk<'m, 'a> {
    memo: &'m CheckerMemo<'a>,
    own: Vec<TypeHandle>,
    /// The own parameters' symbols, aligned with `own`. Upstream counts by
    /// declaration, and one parameter can surface as several type objects
    /// (a conditional type's check type is one).
    own_symbols: Vec<Option<SymbolHandle>>,
    /// The own parameters' names, aligned with `own`.
    names: Vec<vize_l0::String>,
    pub(super) counts: Vec<u32>,
    defaults: Vec<Option<TypeResponse>>,
    from_class: bool,
    budget: usize,
    pub(super) uncertain: bool,
    /// A type parameter's symbol, and whether it is a declared type
    /// parameter (not a class's `this`).
    symbols: FxHashMap<TypeHandle, Option<(SymbolHandle, bool)>>,
    // Per root, as upstream's `collectTypeParameterUsageCounts`.
    usages: FxHashMap<TypeHandle, u32>,
    visited_members: FxHashSet<TypeHandle>,
    visited_constraints: FxHashSet<TypeHandle>,
    visited_default: bool,
}

impl<'m, 'a> Walk<'m, 'a> {
    pub(super) fn new(
        memo: &'m CheckerMemo<'a>,
        own: Vec<TypeHandle>,
        names: Vec<vize_l0::String>,
        defaults: Vec<Option<TypeResponse>>,
        from_class: bool,
    ) -> Self {
        let counts = vec![0; own.len()];
        let own_symbols = vec![None; own.len()];
        Self {
            memo,
            own,
            own_symbols,
            names,
            counts,
            defaults,
            from_class,
            budget: REQUEST_BUDGET,
            uncertain: false,
            symbols: FxHashMap::default(),
            usages: FxHashMap::default(),
            visited_members: FxHashSet::default(),
            visited_constraints: FxHashSet::default(),
            visited_default: false,
        }
    }

    pub(super) fn reset_root(&mut self) {
        self.usages.clear();
        self.visited_members.clear();
        self.visited_constraints.clear();
        self.visited_default = false;
    }

    /// Spend one request; out of budget, the counts are given up.
    pub(super) fn spend(&mut self) -> bool {
        if self.budget == 0 {
            self.uncertain = true;
        }
        self.budget = self.budget.saturating_sub(1);
        !self.uncertain
    }

    /// Nothing left to learn: given up, or every own parameter is used
    /// often enough that none can be reported.
    fn done(&self) -> bool {
        self.uncertain || self.counts.iter().all(|count| *count > 2)
    }

    /// `incrementTypeUsages(type) > 9`.
    fn seen_too_often(&mut self, ty: &TypeHandle) -> bool {
        let seen = self.usages.entry(ty.clone()).or_insert(0);
        *seen += 1;
        *seen > REPEAT_LIMIT
    }

    fn give_up<T>(&mut self, answer: corsa::Result<T>) -> Option<T> {
        match answer {
            Ok(value) => Some(value),
            Err(_) => {
                self.uncertain = true;
                None
            }
        }
    }

    pub(super) fn visit(
        &mut self,
        ty: TypeResponse,
        assume_multiple: bool,
        return_type: bool,
    ) -> Boxed<'_, ()> {
        Box::pin(async move {
            if self.done() || self.seen_too_often(&ty.id) {
                return;
            }
            let flags = ty.flags;
            let object_flags = ty.object_flags.unwrap_or(0);
            if flags & TYPE_PARAMETER != 0 {
                self.type_parameter(ty.id, assume_multiple).await;
                return;
            }
            let client = self.memo.client();
            let snapshot = self.memo.snapshot().handle.clone();
            let project = self.memo.project_handle();
            if !self.spend() {
                return;
            }
            let aliased = client
                .get_alias_type_arguments_of_type(snapshot.clone(), project.clone(), ty.id.clone())
                .await;
            let Some(aliased) = self.give_up(aliased) else {
                return;
            };
            if !aliased.is_empty() {
                // Upstream does not descend into the alias, so it assumes
                // every argument is used several times.
                for argument in aliased {
                    self.visit(argument, true, false).await;
                }
                return;
            }
            if flags & (UNION | INTERSECTION) != 0 {
                if !self.spend() {
                    return;
                }
                for part in self.memo.constituents(&ty).await {
                    self.visit(part, assume_multiple, false).await;
                }
                return;
            }
            if flags & INDEXED_ACCESS != 0 {
                let object = client
                    .get_object_type_of_type_in_project(
                        snapshot.clone(),
                        project.clone(),
                        ty.id.clone(),
                    )
                    .await;
                let index = client
                    .get_index_type_of_type_in_project(snapshot, project, ty.id.clone())
                    .await;
                for part in [object, index] {
                    if !self.spend() {
                        return;
                    }
                    match self.give_up(part) {
                        Some(Some(part)) => self.visit(part, assume_multiple, false).await,
                        Some(None) => self.uncertain = true,
                        None => return,
                    }
                }
                return;
            }
            if flags & OBJECT != 0 && object_flags & REFERENCE != 0 {
                self.reference(&ty, assume_multiple, return_type).await;
                return;
            }
            if flags & TEMPLATE_LITERAL != 0 {
                if !self.spend() {
                    return;
                }
                let parts = self.memo.constituents(&ty).await;
                if parts.is_empty() {
                    self.uncertain = true;
                }
                for part in parts {
                    self.visit(part, assume_multiple, false).await;
                }
                return;
            }
            if flags & CONDITIONAL != 0 {
                let check = client
                    .get_check_type_of_type_in_project(
                        snapshot.clone(),
                        project.clone(),
                        ty.id.clone(),
                    )
                    .await;
                let extends = client
                    .get_extends_type_of_type_in_project(snapshot, project, ty.id.clone())
                    .await;
                for part in [check, extends] {
                    if !self.spend() {
                        return;
                    }
                    if let Some(Some(part)) = self.give_up(part) {
                        self.visit(part, assume_multiple, false).await;
                    }
                }
                return;
            }
            if flags & OBJECT != 0 {
                self.object(&ty, object_flags).await;
                return;
            }
            if flags & (INDEX | STRING_MAPPING) != 0 {
                if !self.spend() {
                    return;
                }
                let target = client
                    .get_target_of_type_in_project(snapshot, project, ty.id.clone())
                    .await;
                match self.give_up(target) {
                    Some(Some(target)) => self.visit(target, assume_multiple, false).await,
                    Some(None) => self.uncertain = true,
                    None => {}
                }
            }
        })
    }

    /// `isTypeParameter(type)`: count an own parameter, then visit its
    /// constraint once and the first default once.
    async fn type_parameter(&mut self, ty: TypeHandle, assume_multiple: bool) {
        let symbol = self.symbol(&ty).await;
        let own = self.own.iter().position(|own| *own == ty).or_else(|| {
            let (symbol, _) = symbol.as_ref()?;
            self.own_symbols
                .iter()
                .position(|own| own.as_ref() == Some(symbol))
        });
        if let Some(index) = own
            && let Some(count) = self.counts.get_mut(index)
        {
            *count += if assume_multiple { 2 } else { 1 };
        }
        let declared = symbol.as_ref().is_some_and(|(_, declared)| *declared);
        if self.visited_constraints.insert(ty.clone()) && declared {
            if !self.spend() {
                return;
            }
            let constraint = self
                .memo
                .client()
                .get_constraint_of_type_parameter(
                    self.memo.snapshot().handle.clone(),
                    self.memo.project_handle(),
                    ty.clone(),
                )
                .await;
            if let Some(Some(constraint)) = self.give_up(constraint) {
                // The checker can answer a copy of a parameter (a
                // conditional type's check type) with the parameter itself.
                // That is the same declaration, which upstream counts once
                // per visit and whose constraint it reads from the syntax.
                let same = constraint.flags & TYPE_PARAMETER != 0
                    && self.symbol(&constraint.id).await.map(|(symbol, _)| symbol)
                        == symbol.map(|(symbol, _)| symbol);
                if !same {
                    self.visit(constraint, false, false).await;
                }
            }
        }
        if let Some(index) = own
            && !self.visited_default
            && let Some(Some(default)) = self.defaults.get(index).cloned()
        {
            self.visited_default = true;
            self.visit(default, false, false).await;
        }
    }

    /// A type parameter's symbol and whether it is declared as one: a
    /// class's `this` type is a type parameter too, whose class declaration
    /// has no constraint for upstream to visit.
    pub(super) async fn symbol(&mut self, ty: &TypeHandle) -> Option<(SymbolHandle, bool)> {
        if let Some(found) = self.symbols.get(ty) {
            return found.clone();
        }
        if !self.spend() {
            return None;
        }
        let symbol = self
            .memo
            .client()
            .get_symbol_of_type_in_project(
                self.memo.snapshot().handle.clone(),
                self.memo.project_handle(),
                ty.clone(),
            )
            .await;
        let found = self
            .give_up(symbol)
            .flatten()
            .map(|symbol| (symbol.id, symbol.flags & SYMBOL_TYPE_PARAMETER != 0));
        self.symbols.insert(ty.clone(), found.clone());
        found
    }

    /// Resolves the own parameters' symbols, and whether upstream counts
    /// each under its own declaration. Upstream keys a parameter's uses by
    /// its symbol's first declaration: a parameter that is not that
    /// declaration (a class after a merged interface) is never reported, and
    /// is left uncounted here. `locations` are the parameters' own nodes.
    pub(super) async fn resolve_own(&mut self, locations: &[Option<NodeHandle>]) -> Vec<bool> {
        let mut counted = Vec::with_capacity(self.own.len());
        for index in 0..self.own.len() {
            let Some(ty) = self.own.get(index).cloned() else {
                continue;
            };
            if !self.spend() {
                return counted;
            }
            let symbol = self
                .memo
                .client()
                .get_symbol_of_type_in_project(
                    self.memo.snapshot().handle.clone(),
                    self.memo.project_handle(),
                    ty,
                )
                .await;
            let symbol = self.give_up(symbol).flatten();
            let location = locations.get(index).and_then(Option::as_ref);
            counted.push(symbol.as_ref().is_some_and(|symbol| {
                match (symbol.declarations.first(), location) {
                    (Some(first), Some(own)) => match (node_index(first), node_index(own)) {
                        // A parameter without constraint or default spans
                        // just its name, so its range locates the name, the
                        // node right after the declaration.
                        (Some(first), Some(own)) => {
                            own == first || Some(own) == first.checked_add(1)
                        }
                        _ => false,
                    },
                    _ => symbol.declarations.len() <= 1,
                }
            }));
            if let Some(slot) = self.own_symbols.get_mut(index) {
                *slot = symbol.map(|symbol| symbol.id);
            }
        }
        counted
    }

    /// `isTypeReference(type)`: generic type arguments. Readonly arrays and
    /// tuples use an argument once; a mutable one does too unless returned;
    /// any other generic counts it twice (and every one does in a class).
    async fn reference(&mut self, ty: &TypeResponse, assume_multiple: bool, return_type: bool) {
        let client = self.memo.client();
        let snapshot = self.memo.snapshot().handle.clone();
        let project = self.memo.project_handle();
        if !self.spend() {
            return;
        }
        let arguments = client
            .get_type_arguments(snapshot.clone(), project.clone(), ty.id.clone())
            .await;
        let Some(arguments) = self.give_up(arguments) else {
            return;
        };
        if arguments.is_empty() {
            return;
        }
        let mut multiple = self.from_class || assume_multiple;
        if !multiple {
            if !self.spend() {
                return;
            }
            let target = client
                .get_target_of_type_in_project(snapshot.clone(), project.clone(), ty.id.clone())
                .await;
            let Some(target) = self.give_up(target).flatten() else {
                self.uncertain = true;
                return;
            };
            multiple = if target.object_flags.unwrap_or(0) & TUPLE != 0 {
                return_type && target.readonly != Some(true)
            } else {
                if !self.spend() {
                    return;
                }
                let array = client
                    .is_array_type(snapshot.clone(), project.clone(), target.id.clone())
                    .await;
                let Some(array) = self.give_up(array) else {
                    return;
                };
                if array {
                    return_type && self.symbol_name(&ty.id).await.as_deref() == Some("Array")
                } else {
                    true
                }
            };
        }
        for argument in arguments {
            self.visit(argument, multiple, return_type).await;
        }
    }

    async fn symbol_name(&mut self, ty: &TypeHandle) -> Option<vize_l0::String> {
        if !self.spend() {
            return None;
        }
        let symbol = self
            .memo
            .client()
            .get_symbol_of_type_in_project(
                self.memo.snapshot().handle.clone(),
                self.memo.project_handle(),
                ty.clone(),
            )
            .await;
        self.give_up(symbol)
            .flatten()
            .map(|symbol| vize_l0::String::from(symbol.name.as_str()))
    }
}

/// The node index a handle names (`<index>.<kind>.<path>`). The checker's
/// handles and those built from its encoded syntax tree share the index.
fn node_index(handle: &NodeHandle) -> Option<u32> {
    handle.as_str().split('.').next()?.parse().ok()
}
