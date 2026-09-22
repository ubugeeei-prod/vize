//! The words of route-typing diagnostics. Text only: every decision is in
//! [`super`], every string is built here.

use vize_carton::line_index::LineIndex;
use vize_carton::{CompactString, cstr};

use super::super::sites::ValueKind;
use super::super::{NamedRoute, RouteParam, RouterTree};
use crate::providers::ProjectSources;

/// How many known route names a help line lists before eliding.
const LISTED_NAMES: usize = 8;

pub(super) fn unknown_route(name: &str) -> CompactString {
    cstr!("unknown route name `{name}`")
}

pub(super) fn unknown_route_help(
    project: &ProjectSources,
    name: &str,
    names: &[&str],
    reachable: &[(u32, &RouterTree)],
) -> Vec<CompactString> {
    let mut help = Vec::new();
    if let Some(closest) = closest(name, names) {
        help.push(cstr!("did you mean `{closest}`?"));
    }
    let mut listed = CompactString::new("known routes: ");
    for (index, known) in names.iter().take(LISTED_NAMES).enumerate() {
        if index > 0 {
            listed.push_str(", ");
        }
        listed.push_str(known);
    }
    if names.len() > LISTED_NAMES {
        listed.push_str(&cstr!(", … ({} more)", names.len() - LISTED_NAMES));
    }
    if names.is_empty() {
        listed.push_str("none");
    }
    let routers: Vec<CompactString> = reachable
        .iter()
        .map(|(_, tree)| location(project, tree.module, tree.span.start))
        .collect();
    listed.push_str(&cstr!(" (router at {})", routers.join(", ")));
    help.push(listed);
    help
}

pub(super) fn declared_at(
    project: &ProjectSources,
    name: &str,
    route: &NamedRoute,
    params: &[RouteParam],
) -> CompactString {
    let path = route.path.as_deref().unwrap_or("");
    let at = location(project, route.module, route.name_span.start);
    let mut text = cstr!("route `{name}` is `{path}` ({at})");
    if params.is_empty() {
        text.push_str(", which takes no params");
        return text;
    }
    text.push_str(", params: ");
    for (index, param) in params.iter().enumerate() {
        if index > 0 {
            text.push_str(", ");
        }
        text.push_str(&cstr!("{}: {}", param.name, param.accepted_type()));
    }
    text
}

pub(super) fn extra_param(name: &str, key: &str) -> CompactString {
    cstr!("route `{name}` has no param `{key}`")
}

/// The clause after "param `x` of route `y`" when `kind` is outside what
/// `param` accepts.
pub(super) fn type_problem(param: &RouteParam, kind: ValueKind) -> Option<CompactString> {
    let token = token(param);
    match kind {
        ValueKind::Array { .. } if !param.repeatable => {
            Some(cstr!("is an array, but `{token}` is not repeatable"))
        }
        ValueKind::Array { empty: true } if !param.optional => Some(cstr!(
            "is an empty array, but `{token}` needs at least one segment"
        )),
        ValueKind::Text { empty: true } if !param.optional => {
            Some(cstr!("is an empty string, but `{token}` is required"))
        }
        ValueKind::Nullish(value) if !param.optional => {
            Some(cstr!("is `{value}`, but `{token}` is required"))
        }
        ValueKind::Invalid(what) => {
            Some(cstr!("is {what}, but route params are strings or numbers"))
        }
        _ => None,
    }
}

pub(super) fn param_type(name: &str, param: &str, problem: &str) -> CompactString {
    cstr!("param `{param}` of route `{name}` {problem}")
}

pub(super) fn accepted_label(param: &RouteParam) -> CompactString {
    cstr!("expected {}", param.accepted_type())
}

pub(super) fn missing_params(name: &str, missing: &[&str]) -> CompactString {
    let noun = if missing.len() == 1 {
        "param"
    } else {
        "params"
    };
    cstr!(
        "navigation to `{name}` does not pass required {noun} {}",
        quoted(missing)
    )
}

pub(super) fn missing_label(missing: &[&str]) -> CompactString {
    let verb = if missing.len() == 1 { "is" } else { "are" };
    cstr!("{} {verb} not passed here", quoted(missing))
}

pub(super) fn inherit_help(missing: &[&str]) -> CompactString {
    cstr!(
        "Vue Router fills {} only from the current route and throws `Missing required param \"{}\"` when it has none",
        quoted(missing),
        missing[0]
    )
}

/// The param as the path spells it: `:id`, `:id?`, `:tags+`, `:rest*`.
fn token(param: &RouteParam) -> CompactString {
    let modifier = match (param.optional, param.repeatable) {
        (true, true) => "*",
        (true, false) => "?",
        (false, true) => "+",
        (false, false) => "",
    };
    cstr!(":{}{modifier}", param.name)
}

fn quoted(names: &[&str]) -> CompactString {
    let mut text = CompactString::default();
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            text.push_str(", ");
        }
        text.push_str(&cstr!("`{name}`"));
    }
    text
}

/// `path:line:column`, one-based, columns in UTF-16 units (the LSP's).
fn location(
    project: &ProjectSources,
    module: crate::providers::ModuleId,
    offset: u32,
) -> CompactString {
    let module = project.module(module);
    let (line, column) = LineIndex::new(module.source()).line_col(offset as usize);
    cstr!("{}:{}:{}", module.path(), line + 1, column + 1)
}

/// The known name nearest to `name` by edit distance, when it is close
/// enough to be a plausible typo (at most a third of the name, at least 2).
fn closest<'n>(name: &str, names: &[&'n str]) -> Option<&'n str> {
    let limit = (name.chars().count() / 3).max(2);
    names
        .iter()
        .map(|known| (edit_distance(name, known), *known))
        .filter(|(distance, _)| *distance <= limit)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, known)| known)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, left) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, right) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = if left == *right {
                diagonal
            } else {
                1 + diagonal.min(above).min(row[j])
            };
            diagonal = above;
        }
    }
    row[b.len()]
}
