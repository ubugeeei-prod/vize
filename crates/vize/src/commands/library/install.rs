//! `vize lib pull` and `vize lib update`.

use std::path::Path;

use serde::Serialize;
use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::fs_ops::join_relative;
use super::fs_ops::project_relative_dir;
use super::lockfile::Lockfile;
use super::output::{declared_npm_packages, json, line, npm_hint, render_plan};
use super::plan::{ItemPlan, apply, conflict_summary, join_dir, plan_item};
use super::query::{ItemSpec, resolve_spec};
use super::registry::NpmDependency;
use super::resolve::Source;
use super::{LibContext, PullArgs, UpdateArgs};

/// Items of one kind requested at one version.
struct Request {
    kind: Source,
    version: Option<String>,
    names: Vec<String>,
}

fn group(
    requests: &mut Vec<Request>,
    kind: Source,
    version: Option<String>,
    name: String,
) -> LibResult<()> {
    match requests.iter_mut().find(|request| request.kind == kind) {
        Some(request) if request.version != version => Err(LibError::new(cstr!(
            "conflicting versions requested for {} items ({} vs {})",
            kind.as_str(),
            request.version.as_deref().unwrap_or("default"),
            version.as_deref().unwrap_or("default")
        ))),
        Some(request) => {
            if !request.names.contains(&name) {
                request.names.push(name);
            }
            Ok(())
        }
        None => {
            requests.push(Request {
                kind,
                version,
                names: vec![name],
            });
            Ok(())
        }
    }
}

/// Every item of one kind shares one directory so cross-item relative imports resolve.
fn target_dir(
    context: &LibContext,
    lockfile: &Lockfile,
    kind: &Source,
    flag: Option<&Path>,
    registry_default: &str,
) -> LibResult<String> {
    let requested = match flag {
        Some(dir) => Some(project_relative_dir(&context.root, dir)?),
        None => None,
    };
    let locked = lockfile
        .items
        .iter()
        .find(|item| item.kind == kind.as_str())
        .map(|item| item.dir.clone());
    match (requested, locked) {
        (Some(requested), Some(locked)) if requested != locked => Err(LibError::new(cstr!(
            "{} items are already pulled into {locked}; items of one kind must share a directory",
            kind.as_str()
        ))),
        (Some(dir), _) | (None, Some(dir)) => Ok(dir),
        (None, None) => {
            let configured = match kind {
                Source::Namespace(name) => context
                    .resolver
                    .namespace(name)
                    .and_then(|namespace| namespace.dir.as_deref()),
                _ => context.config.dir_for_kind(kind.as_str()),
            };
            project_relative_dir(
                &context.root,
                Path::new(configured.unwrap_or(registry_default)),
            )
        }
    }
}

fn plan_request(
    context: &mut LibContext,
    lockfile: &Lockfile,
    request: &Request,
    dir_flag: Option<&Path>,
    direct_names: &[String],
) -> LibResult<Vec<ItemPlan>> {
    let default_dir = context
        .resolver
        .registry(&request.kind, request.version.as_deref())?
        .manifest
        .default_target_directory
        .clone();
    let dir = target_dir(context, lockfile, &request.kind, dir_flag, &default_dir)?;
    let root = context.root.clone();
    let registry = context
        .resolver
        .registry(&request.kind, request.version.as_deref())?;
    let mut ordered: Vec<String> = Vec::new();
    for name in &request.names {
        let item = registry.item(name).ok_or_else(|| {
            LibError::new(cstr!(
                "{} does not publish {name}",
                registry.package_label()
            ))
        })?;
        for member in registry.closure(item)? {
            if !ordered.contains(&member.name) {
                ordered.push(member.name.clone());
            }
        }
    }
    let mut plans: Vec<ItemPlan> = Vec::with_capacity(ordered.len());
    for name in &ordered {
        let Some(item) = registry.item(name) else {
            continue;
        };
        let existing = lockfile.get(request.kind.as_str(), name);
        let direct = direct_names.contains(name) || existing.is_some_and(|locked| locked.direct);
        plans.push(plan_item(
            &root,
            &dir,
            request.kind.as_str(),
            registry,
            item,
            existing,
            direct,
        )?);
    }
    reject_foreign_owners(&root, lockfile, &plans)?;
    Ok(plans)
}

/// Refuse to write a file another source's or item's lock entry already owns.
fn reject_foreign_owners(root: &Path, lockfile: &Lockfile, plans: &[ItemPlan]) -> LibResult<()> {
    for plan in plans {
        let base = join_dir(root, &plan.dir)?;
        for file in &plan.files {
            let target = join_relative(&base, &file.path)?;
            for owner in &lockfile.items {
                if owner.kind == plan.kind && owner.name == plan.name {
                    continue;
                }
                let owner_base = join_dir(root, &owner.dir)?;
                for path in owner.files.keys() {
                    if join_relative(&owner_base, path)? == target {
                        return Err(LibError::new(cstr!(
                            "{} would overwrite {}, which belongs to pulled item {}:{}",
                            plan.name,
                            target.display(),
                            owner.kind,
                            owner.name
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallReport<'a> {
    command: &'a str,
    dry_run: bool,
    applied: bool,
    items: &'a [ItemPlan],
    conflicts: Option<String>,
    missing_npm_dependencies: Vec<NpmDependency>,
}

fn finish(
    context: &LibContext,
    command: &str,
    plans: &[ItemPlan],
    lockfile: &mut Lockfile,
    dry_run: bool,
    force: bool,
    force_flag: &str,
) -> LibResult<String> {
    let conflicts = conflict_summary(plans);
    let applied = !dry_run && (force || conflicts.is_none());
    let mut out = String::default();
    if !context.json {
        let verb = if dry_run { "would" } else { command };
        render_plan(&mut out, plans, verb);
    }
    if !dry_run {
        apply(
            plans,
            &context.root,
            lockfile,
            &context.lock_path,
            force,
            force_flag,
        )?;
    }
    let missing = npm_hint(plans, &declared_npm_packages(&context.root));
    if context.json {
        return json(&InstallReport {
            command,
            dry_run,
            applied,
            items: plans,
            conflicts,
            missing_npm_dependencies: missing,
        });
    }
    if dry_run && let Some(conflicts) = conflicts {
        line(
            &mut out,
            format_args!("conflicts (need {force_flag}):\n{conflicts}"),
        );
    }
    if !plans.iter().any(ItemPlan::changes_anything) {
        line(&mut out, format_args!("already up to date"));
    }
    for dependency in missing {
        line(
            &mut out,
            format_args!(
                "note: add {} {}@{} to package.json",
                if dependency.kind == "peer" {
                    "peer"
                } else {
                    "dependency"
                },
                dependency.name,
                dependency.range
            ),
        );
    }
    Ok(out)
}

pub fn pull(context: &mut LibContext, args: &PullArgs) -> LibResult<String> {
    let mut lockfile = context.lockfile()?;
    let mut requests: Vec<Request> = Vec::new();
    for raw in &args.items {
        let resolved = resolve_spec(context, &ItemSpec::parse(raw)?)?;
        group(
            &mut requests,
            resolved.source,
            resolved.version,
            resolved.name,
        )?;
    }
    let mut plans = Vec::new();
    for request in &requests {
        let names = request.names.clone();
        plans.extend(plan_request(
            context,
            &lockfile,
            request,
            args.dir.as_deref(),
            &names,
        )?);
    }
    finish(
        context,
        "pull",
        &plans,
        &mut lockfile,
        args.dry_run,
        args.overwrite,
        "--overwrite",
    )
}

pub fn update(context: &mut LibContext, args: &UpdateArgs) -> LibResult<String> {
    let mut lockfile = context.lockfile()?;
    if lockfile.items.is_empty() {
        return Err(LibError::new(
            "nothing pulled yet (no vize-lib.lock.json entries)",
        ));
    }
    let mut requests: Vec<Request> = Vec::new();
    let targets: Vec<(String, String)> = if args.items.is_empty() {
        lockfile
            .items
            .iter()
            .map(|item| (item.kind.clone(), item.name.clone()))
            .collect()
    } else {
        let mut targets = Vec::new();
        for query in &args.items {
            match lockfile.find(query).as_slice() {
                [item] => targets.push((item.kind.clone(), item.name.clone())),
                [] => return Err(LibError::new(cstr!("{query} is not pulled"))),
                _ => {
                    return Err(LibError::new(cstr!(
                        "{query} is ambiguous; prefix it with ui: or composable:"
                    )));
                }
            }
        }
        targets
    };
    for (kind, name) in targets {
        let kind = Source::parse(&kind)
            .ok_or_else(|| LibError::new(cstr!("unknown kind {kind} in lockfile")))?;
        group(&mut requests, kind, args.to.clone(), name)?;
    }
    let mut plans = Vec::new();
    for request in &requests {
        let direct: Vec<String> = lockfile
            .items
            .iter()
            .filter(|item| item.direct && item.kind == request.kind.as_str())
            .map(|item| item.name.clone())
            .collect();
        plans.extend(plan_request(context, &lockfile, request, None, &direct)?);
    }
    finish(
        context,
        "update",
        &plans,
        &mut lockfile,
        args.dry_run,
        args.force,
        "--force",
    )
}
