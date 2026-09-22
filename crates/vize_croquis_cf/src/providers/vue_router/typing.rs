//! Route typing — the consumer of the Vue Router provider's groups.
//!
//! Every navigation site ([`super::sites`]) is checked against the route
//! facts, and every error carries the witness chain that proves it:
//!
//! | code                                | severity | claim, and its witness                                              |
//! | ----------------------------------- | -------- | ------------------------------------------------------------------- |
//! | `ecosystem/vue-router-unknown-route` | error    | no record of any router the site can reach has the name — one `route-tree` link per such router, each proven only while its tree is closed |
//! | `ecosystem/vue-router-extra-param`   | error    | the route's static path declares no such param — its `route-params` link |
//! | `ecosystem/vue-router-param-type`    | error    | the value's syntax is outside what the param accepts — its `route-params` link |
//! | `ecosystem/vue-router-missing-param` | warning  | a required param is not passed, so the navigation depends on the current route carrying it (Vue Router fills missing params from it) |
//!
//! The missing-param finding is a warning on purpose: Vue Router inherits
//! required params from the current location, so the navigation throws
//! only when that location lacks them — a claim this rule cannot prove.

mod messages;

use vize_carton::{CompactString, FxHashSet};
use vize_davinci::diagnostic::{
    Advisory, Diagnostic, DiagnosticPart, Domain, PartKind, RuleContract, Severity, Stage, Tier,
    WitnessChain, WitnessLink,
};
use vize_davinci::fact::{Demand, FactConsumer, FactError, FactGroup, FactManager, FactView};
use vize_davinci::witness::{WitnessCheck, WitnessChecks, WitnessGroup};

use super::sites::{self, NavSite, Params, Target};
use super::{NamedRoute, RouteParams, RouteTree, RouterTree};
use crate::providers::{ModuleId, PROJECT_FACTS, ProjectSources};

/// Unknown route name.
pub const UNKNOWN_ROUTE: &str = "ecosystem/vue-router-unknown-route";
/// A param the route does not declare.
pub const EXTRA_PARAM: &str = "ecosystem/vue-router-extra-param";
/// A param value the param cannot take.
pub const PARAM_TYPE: &str = "ecosystem/vue-router-param-type";
/// A required param left to inheritance from the current route.
pub const MISSING_PARAM: &str = "ecosystem/vue-router-missing-param";

/// The rule's contract: `sound` over its declared domain.
pub static CONTRACT: RuleContract = RuleContract::new(
    Tier::Sound,
    Domain::new(
        "named navigations with a literal name and a params object literal of static keys, \
         checked against routers whose `createRouter({ routes })` records are static",
    ),
    Severity::Error,
);

/// The route-typing consumer: reads the route tree and the route params.
pub struct RouteTyping;

impl FactConsumer for RouteTyping {
    const NAME: &'static str = "vue-router/route-typing";
    const DEMAND: Demand = Demand::NONE.with(RouteTree::ID).with(RouteParams::ID);
}

/// The groups a route-typing witness may cite.
pub const WITNESS_CHECKS: WitnessChecks = WitnessChecks::new(&[
    WitnessCheck::of::<RouteTree>(),
    WitnessCheck::of::<RouteParams>(),
]);

impl WitnessGroup for RouteTree {
    fn fact_span(_: &u32, tree: &RouterTree) -> vize_carton::Span {
        tree.span
    }

    /// A router's tree proves "no such name" only while it is closed.
    fn verdict(tree: &RouterTree) -> vize_davinci::diagnostic::Verdict {
        if tree.is_closed() {
            vize_davinci::diagnostic::Verdict::Proven
        } else {
            vize_davinci::diagnostic::Verdict::Unknown
        }
    }
}

impl WitnessGroup for RouteParams {
    fn fact_span(_: &CompactString, route: &NamedRoute) -> vize_carton::Span {
        route.name_span
    }

    /// A route's params are proven when its path is static and the name is
    /// declared exactly once.
    fn verdict(route: &NamedRoute) -> vize_davinci::diagnostic::Verdict {
        if route.params.is_some() && route.declarations == 1 {
            vize_davinci::diagnostic::Verdict::Proven
        } else {
            vize_davinci::diagnostic::Verdict::Unknown
        }
    }
}

/// One route-typing finding, in the module it points into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDiagnostic {
    /// The module the diagnostic's spans are offsets into.
    pub module: ModuleId,
    /// The rule code (`ecosystem/vue-router-…`).
    pub code: &'static str,
    /// The diagnostic on the unified channel.
    pub diagnostic: Diagnostic,
}

/// Compute the provider's facts for `project` and check every navigation.
///
/// # Errors
///
/// A [`FactError`] only if the project registry lost a route group — a
/// build without the `provider-vue-router` feature.
pub fn run(project: &ProjectSources) -> Result<Vec<RouteDiagnostic>, FactError> {
    let mut manager = FactManager::new(&PROJECT_FACTS);
    let view = manager.prepare::<RouteTyping>(project)?;
    check(project, &view)
}

/// Check every navigation of `project` against the facts in `view`.
///
/// # Errors
///
/// As [`FactView::get`] for the two demanded groups.
pub fn check(
    project: &ProjectSources,
    view: &FactView<'_>,
) -> Result<Vec<RouteDiagnostic>, FactError> {
    let trees = view.get::<RouteTree>()?;
    let named = view.get::<RouteParams>()?;
    let routers: Vec<(u32, &RouterTree)> = trees.iter().map(|(key, tree)| (*key, tree)).collect();
    if routers.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for site in sites::collect(project, &routers) {
        let reachable: Vec<(u32, &RouterTree)> = routers
            .iter()
            .filter(|(key, _)| site.target == Target::App || site.target == Target::Router(*key))
            .copied()
            .collect();
        let declared = reachable.iter().any(|(_, tree)| {
            tree.records
                .iter()
                .any(|record| record.name.as_deref() == Some(&*site.name))
        });
        if !declared {
            unknown_route(project, &site, &reachable, &mut out);
            continue;
        }
        if let Some(route) = named.get(&site.name)
            && reachable.iter().any(|(key, _)| *key == route.router)
            && let (Some(params), 1) = (&route.params, route.declarations)
        {
            check_params(project, &site, route, params, &mut out);
        }
    }
    Ok(out)
}

fn unknown_route(
    project: &ProjectSources,
    site: &NavSite,
    reachable: &[(u32, &RouterTree)],
    out: &mut Vec<RouteDiagnostic>,
) {
    if reachable.is_empty() || reachable.iter().any(|(_, tree)| !tree.is_closed()) {
        return;
    }
    let links = reachable
        .iter()
        .map(|(key, tree)| WitnessLink::of::<RouteTree>(key, tree.span))
        .collect();
    let Some(chain) = WitnessChain::from_links(links) else {
        return;
    };
    let mut names: Vec<&str> = Vec::new();
    let mut seen = FxHashSet::default();
    for (_, tree) in reachable {
        for record in &tree.records {
            if let Some(name) = record.name.as_deref()
                && seen.insert(name)
            {
                names.push(name);
            }
        }
    }
    let message = messages::unknown_route(&site.name);
    let mut diagnostic =
        Diagnostic::proven(Stage::Semantic, site.name_span, message, chain).with_part(
            DiagnosticPart::new(PartKind::Primary, site.name_span, "no route has this name"),
        );
    for help in messages::unknown_route_help(project, &site.name, &names, reachable) {
        diagnostic =
            diagnostic.with_part(DiagnosticPart::new(PartKind::Help, site.name_span, help));
    }
    out.push(RouteDiagnostic {
        module: site.module,
        code: UNKNOWN_ROUTE,
        diagnostic,
    });
}

fn check_params(
    project: &ProjectSources,
    site: &NavSite,
    route: &NamedRoute,
    params: &[super::RouteParam],
    out: &mut Vec<RouteDiagnostic>,
) {
    let chain = || WitnessChain::new(WitnessLink::of::<RouteParams>(&site.name, route.name_span));
    let declared_at = messages::declared_at(project, &site.name, route, params);
    let entries = match &site.params {
        Params::Unknown => return,
        Params::Absent => &[][..],
        Params::Known { entries, .. } => entries.as_slice(),
    };
    for entry in entries {
        let Some(param) = params.iter().find(|param| param.name == entry.key) else {
            let message = messages::extra_param(&site.name, &entry.key);
            let diagnostic = Diagnostic::proven(Stage::Semantic, entry.key_span, message, chain())
                .with_part(DiagnosticPart::new(
                    PartKind::Primary,
                    entry.key_span,
                    "not a param of this route",
                ))
                .with_part(DiagnosticPart::new(
                    PartKind::Help,
                    entry.key_span,
                    declared_at.clone(),
                ));
            out.push(RouteDiagnostic {
                module: site.module,
                code: EXTRA_PARAM,
                diagnostic,
            });
            continue;
        };
        let Some(problem) = messages::type_problem(param, entry.value) else {
            continue;
        };
        let message = messages::param_type(&site.name, &param.name, &problem);
        let label = messages::accepted_label(param);
        let diagnostic = Diagnostic::proven(Stage::Semantic, entry.value_span, message, chain())
            .with_part(DiagnosticPart::new(
                PartKind::Primary,
                entry.value_span,
                label,
            ))
            .with_part(DiagnosticPart::new(
                PartKind::Help,
                entry.value_span,
                declared_at.clone(),
            ));
        out.push(RouteDiagnostic {
            module: site.module,
            code: PARAM_TYPE,
            diagnostic,
        });
    }
    let missing: Vec<&str> = params
        .iter()
        .filter(|param| !param.optional && !entries.iter().any(|entry| entry.key == param.name))
        .map(|param| param.name.as_str())
        .collect();
    if missing.is_empty() {
        return;
    }
    let span = match &site.params {
        Params::Known { span, .. } => *span,
        Params::Absent | Params::Unknown => site.name_span,
    };
    let message = messages::missing_params(&site.name, &missing);
    let diagnostic = Diagnostic::new(Advisory::Warning, Stage::Semantic, span, message)
        .with_witness(chain())
        .with_part(DiagnosticPart::new(
            PartKind::Primary,
            span,
            messages::missing_label(&missing),
        ))
        .with_part(DiagnosticPart::new(
            PartKind::Help,
            span,
            messages::inherit_help(&missing),
        ))
        .with_part(DiagnosticPart::new(PartKind::Help, span, declared_at));
    out.push(RouteDiagnostic {
        module: site.module,
        code: MISSING_PARAM,
        diagnostic,
    });
}
