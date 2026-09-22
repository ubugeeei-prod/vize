//! The in-tree Vue Router provider (P4-10a).
//!
//! [`VueRouterProvider`] owns two project-level fact groups:
//!
//! - [`RouteTree`] (stratum 0) — one [`RouterTree`] per statically found
//!   `createRouter({ routes })` call: every route record with its full path,
//!   name, parent and statically resolved components, plus the reasons the
//!   tree is **open** (not statically complete), if any;
//! - [`RouteParams`] (stratum 1, reads `RouteTree`) — one [`NamedRoute`] per
//!   route name: the params a navigation by that name takes.
//!
//! [`typing`] is the consumer: the route-typing checks at
//! `router.push({ name, params })`, `$router.push(…)` and
//! `<RouterLink :to="{ name, params }">`.
//!
//! # Declared domain (tier `sound`)
//!
//! Facts are extracted only where they are statically certain: `createRouter`
//! imported from `vue-router`, `routes` an array literal (or a `const` /
//! relatively imported binding of one), records object literals with literal
//! `name`/`path`. Anything else — a spread of an unknown value, a computed
//! name, a non-literal path, any `addRoute` call in the analyzed modules —
//! is recorded as an [`OpenReason`], and a consumer never claims what an open
//! tree cannot prove.

pub mod path;
mod records;
mod resolve;
mod sites;
pub mod typing;

pub use path::{RouteParam, join_route_path, parse_route_params};
pub use typing::{RouteDiagnostic, RouteTyping};

use vize_carton::{CompactString, FxHashMap, Span};
use vize_davinci::fact::{Demand, FactGroup, FactProducer, FactTable, FactView};
use vize_davinci::pass::AnalysisId;

use super::{AmbientInput, ModuleId, ProjectSources, Provider, ids};

/// The Vue Router provider: `createRouter({ routes })` records as facts.
pub struct VueRouterProvider;

impl Provider for VueRouterProvider {
    const NAME: &'static str = "vue-router";
    const INPUTS: &'static [AmbientInput] = &[AmbientInput::ScriptModules];
    const OUTPUTS: Demand = Demand::NONE.with(RouteTree::ID).with(RouteParams::ID);
}

/// One route record, as statically extracted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteRecord {
    /// The record's literal `name`, if it has one.
    pub name: Option<CompactString>,
    /// The whole-file span of the `name` value (or of the record).
    pub name_span: Span,
    /// The full path (parent joined), or `None` when it or an ancestor's
    /// path is not a literal.
    pub path: Option<CompactString>,
    /// The module the record literal lives in.
    pub module: ModuleId,
    /// The record's parent, by index into [`RouterTree::records`].
    pub parent: Option<u32>,
    /// The `.vue` modules the record renders (`component`, `components`).
    pub components: Vec<ModuleId>,
}

/// Why a router's tree is not statically complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenReason {
    /// `routes` (or `children`) is not a statically known array.
    DynamicRoutes { module: ModuleId, span: Span },
    /// An array element or record property is not statically known.
    DynamicRecord { module: ModuleId, span: Span },
    /// A module calls `addRoute`, which can add names at runtime.
    AddRoute { module: ModuleId, span: Span },
}

/// One router: a `createRouter(...)` call and its extracted tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouterTree {
    /// The module holding the `createRouter` call.
    pub module: ModuleId,
    /// The whole-file span of the call.
    pub span: Span,
    /// Every record, parents before children, in declaration order.
    pub records: Vec<RouteRecord>,
    /// Export names the router instance is reachable under (`default`, …).
    pub exports: Vec<CompactString>,
    /// Why the tree is open; empty when it is statically complete.
    pub open: Vec<OpenReason>,
}

impl RouterTree {
    /// Whether every route of the router is statically known.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.open.is_empty()
    }

    /// Indices of `record` and every record below it.
    pub fn subtree(&self, record: u32) -> impl Iterator<Item = u32> + '_ {
        (record..self.records.len() as u32).filter(move |candidate| {
            let mut at = Some(*candidate);
            while let Some(index) = at {
                if index == record {
                    return true;
                }
                at = self.records[index as usize].parent;
            }
            false
        })
    }
}

/// A named route's signature — what a navigation by that name must match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedRoute {
    /// The router, by [`RouteTree`] key.
    pub router: u32,
    /// The record, by index into that router's records.
    pub record: u32,
    /// The module declaring the record.
    pub module: ModuleId,
    /// The whole-file span of the `name` literal.
    pub name_span: Span,
    /// The full path, when static.
    pub path: Option<CompactString>,
    /// The params the path declares, when the path is static.
    pub params: Option<Vec<RouteParam>>,
    /// How many records across all routers carry this name. Above one the
    /// name is ambiguous and no param claim is made.
    pub declarations: u32,
}

/// Stratum 0: every router's statically extracted tree, keyed by router
/// ordinal (discovery order).
pub struct RouteTree;

impl FactGroup for RouteTree {
    const ID: AnalysisId = ids::ROUTE_TREE;
    const NAME: &'static str = "route-tree";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = RouterTree;
}

impl FactProducer<ProjectSources> for RouteTree {
    fn produce(project: &ProjectSources, _: &FactView<'_>) -> FactTable<Self> {
        (0u32..).zip(records::collect(project)).collect()
    }
}

/// Stratum 1: the params of every route name, read off [`RouteTree`].
pub struct RouteParams;

impl FactGroup for RouteParams {
    const ID: AnalysisId = ids::ROUTE_PARAMS;
    const NAME: &'static str = "route-params";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE.with(RouteTree::ID);
    type Key = CompactString;
    type Value = NamedRoute;
}

impl FactProducer<ProjectSources> for RouteParams {
    fn produce(_: &ProjectSources, inputs: &FactView<'_>) -> FactTable<Self> {
        let Ok(trees) = inputs.get::<RouteTree>() else {
            return FactTable::default();
        };
        let mut named: FxHashMap<CompactString, NamedRoute> = FxHashMap::default();
        for (router, tree) in trees.iter() {
            for (record, route) in (0u32..).zip(&tree.records) {
                let Some(name) = &route.name else { continue };
                named
                    .entry(name.clone())
                    .and_modify(|existing| existing.declarations += 1)
                    .or_insert_with(|| NamedRoute {
                        router: *router,
                        record,
                        module: route.module,
                        name_span: route.name_span,
                        path: route.path.clone(),
                        params: route.path.as_deref().map(parse_route_params),
                        declarations: 1,
                    });
            }
        }
        named.into_iter().collect()
    }
}
