//! The exact text a fixture's `expected.txt` holds: the provider's facts,
//! then every diagnostic with its parts and witness links.

use std::fmt::Write;

use vize_carton::Span;
use vize_carton::line_index::LineIndex;
use vize_croquis_cf::providers::vue_router::typing::RouteDiagnostic;
use vize_croquis_cf::providers::vue_router::{OpenReason, RouteParams, RouteTree};
use vize_croquis_cf::providers::{ModuleId, ProjectSources};
use vize_davinci::diagnostic::{PartKind, Severity};
use vize_davinci::fact::{FactGroup, FactView};

fn at(project: &ProjectSources, module: ModuleId, span: Span) -> String {
    let source = project.module(module);
    let index = LineIndex::new(source.source());
    let (line, column) = index.line_col(span.start as usize);
    let (end_line, end_column) = index.line_col(span.end as usize);
    format!(
        "{}:{}:{}-{}:{}",
        source.path(),
        line + 1,
        column + 1,
        end_line + 1,
        end_column + 1
    )
}

pub fn fixture(
    project: &ProjectSources,
    view: &FactView<'_>,
    diagnostics: &[RouteDiagnostic],
) -> String {
    let mut out = String::new();
    let trees = view.get::<RouteTree>().unwrap();
    writeln!(out, "== route-tree ({} routers)", trees.len()).unwrap();
    for (key, tree) in trees.iter() {
        let state = if tree.is_closed() { "closed" } else { "open" };
        writeln!(
            out,
            "router #{key} {} exports=[{}] {state}",
            at(project, tree.module, tree.span),
            tree.exports.join(", ")
        )
        .unwrap();
        for record in &tree.records {
            let parent = record
                .parent
                .map_or_else(|| "-".to_owned(), |parent| format!("#{parent}"));
            let components: Vec<_> = record
                .components
                .iter()
                .map(|id| project.module(*id).path())
                .collect();
            writeln!(
                out,
                "  {} path={} parent={parent} components=[{}]",
                record.name.as_deref().unwrap_or("(unnamed)"),
                record.path.as_deref().unwrap_or("(dynamic)"),
                components.join(", ")
            )
            .unwrap();
        }
        for reason in &tree.open {
            let (kind, module, span) = match reason {
                OpenReason::DynamicRoutes { module, span } => ("dynamic-routes", module, span),
                OpenReason::DynamicRecord { module, span } => ("dynamic-record", module, span),
                OpenReason::AddRoute { module, span } => ("add-route", module, span),
            };
            writeln!(out, "  open: {kind} {}", at(project, *module, *span)).unwrap();
        }
    }
    let params = view.get::<RouteParams>().unwrap();
    writeln!(out, "== route-params ({} names)", params.len()).unwrap();
    for (name, route) in params.iter() {
        let signature = match &route.params {
            None => "(dynamic path)".to_owned(),
            Some(params) => params
                .iter()
                .map(|param| format!("{}: {}", param.name, param.accepted_type()))
                .collect::<Vec<_>>()
                .join(", "),
        };
        writeln!(
            out,
            "{name} router=#{} declarations={} params=[{signature}]",
            route.router, route.declarations
        )
        .unwrap();
    }
    writeln!(out, "== diagnostics ({})", diagnostics.len()).unwrap();
    for finding in diagnostics {
        let diagnostic = &finding.diagnostic;
        let severity = match diagnostic.severity() {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
        };
        writeln!(
            out,
            "{severity}[{}] {}",
            finding.code,
            at(project, finding.module, diagnostic.span)
        )
        .unwrap();
        writeln!(out, "  {}", diagnostic.message).unwrap();
        for part in &diagnostic.parts {
            let kind = match part.kind {
                PartKind::Primary => "primary",
                PartKind::Secondary => "secondary",
                PartKind::Help => "help",
                PartKind::Suggestion => "suggestion",
            };
            writeln!(out, "  {kind}: {}", part.message).unwrap();
        }
        for link in diagnostic
            .witness_chain()
            .into_iter()
            .flat_map(|chain| chain.links())
        {
            let group = if link.group == RouteTree::ID {
                RouteTree::NAME
            } else {
                RouteParams::NAME
            };
            writeln!(
                out,
                "  witness: {group} {:?} @ {}..{}",
                link.key, link.span.start, link.span.end
            )
            .unwrap();
        }
    }
    out
}
