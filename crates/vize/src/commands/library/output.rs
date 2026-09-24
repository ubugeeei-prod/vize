//! Rendering helpers shared by `vize lib` subcommands.

use std::fmt::Write as _;

use serde::Serialize;
use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::plan::{FileAction, ItemPlan};
use super::registry::NpmDependency;

/// Pretty JSON with a trailing newline.
pub fn json<T: Serialize>(value: &T) -> LibResult<String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| LibError::new(cstr!("failed to serialize output: {error}")))?;
    bytes.push(b'\n');
    String::from_utf8(bytes).map_err(|error| LibError::new(cstr!("invalid UTF-8 output: {error}")))
}

/// Append one formatted line.
pub fn line(out: &mut String, args: std::fmt::Arguments<'_>) {
    let _ = out.write_fmt(args);
    out.push('\n');
}

fn action_marker(action: FileAction) -> &'static str {
    match action {
        FileAction::Create => "+",
        FileAction::Update => "~",
        FileAction::Unchanged => "=",
        FileAction::KeepLocal => "*",
        FileAction::Delete => "-",
        FileAction::Conflict | FileAction::ConflictDelete => "!",
    }
}

/// Human-readable plan: one header per item, one line per changed file.
pub fn render_plan(out: &mut String, plans: &[ItemPlan], verb: &str) {
    for plan in plans {
        let version = match plan.from_version.as_deref() {
            Some(from) if from != plan.to_version => cstr!("{from} -> {}", plan.to_version),
            _ => plan.to_version.clone(),
        };
        let role = if plan.direct { "" } else { " (dependency)" };
        line(
            out,
            format_args!(
                "{verb} {}:{} {version} into {}{role}",
                plan.kind, plan.name, plan.dir
            ),
        );
        for file in &plan.files {
            if file.action == FileAction::Unchanged {
                continue;
            }
            line(
                out,
                format_args!(
                    "  {} {} ({})",
                    action_marker(file.action),
                    file.path,
                    file.action.as_str()
                ),
            );
        }
    }
}

/// Unique npm dependencies of `plans`, with the ones `package.json` lacks.
pub fn npm_hint(plans: &[ItemPlan], declared: &[String]) -> Vec<NpmDependency> {
    let mut missing: Vec<NpmDependency> = Vec::new();
    for dependency in plans.iter().flat_map(|plan| &plan.npm_dependencies) {
        if declared.contains(&dependency.name)
            || missing.iter().any(|seen| seen.name == dependency.name)
        {
            continue;
        }
        missing.push(dependency.clone());
    }
    missing
}

/// Dependency names declared by the project's `package.json` (all sections).
pub fn declared_npm_packages(root: &std::path::Path) -> Vec<String> {
    let Ok(bytes) = std::fs::read(root.join("package.json")) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Vec::new();
    };
    [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ]
    .into_iter()
    .filter_map(|section| value.get(section).and_then(serde_json::Value::as_object))
    .flat_map(|section| section.keys().map(|name| String::from(name.as_str())))
    .collect()
}
