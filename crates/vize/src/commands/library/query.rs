//! `vize lib list | search | info` and item spec resolution.

use serde::Serialize;
use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::output::{json, line};
use super::registry::{LoadedRegistry, NpmDependency, RegistryFile};
use super::resolve::RegistryKind;
use super::{KindArg, LibContext};

/// Parsed `[kind:]name[@version]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSpec {
    pub kind: Option<RegistryKind>,
    pub name: String,
    pub version: Option<String>,
}

impl ItemSpec {
    pub fn parse(raw: &str) -> LibResult<Self> {
        let raw = raw.trim();
        let (kind, rest) = match raw.split_once(':') {
            Some((prefix, rest)) => {
                let kind = RegistryKind::parse(prefix).ok_or_else(|| {
                    LibError::new(cstr!("unknown kind {prefix:?} (use ui: or composable:)"))
                })?;
                (Some(kind), rest)
            }
            None => (None, raw),
        };
        let (name, version) = match rest.rsplit_once('@') {
            Some((name, version)) if !name.is_empty() => (name, Some(version)),
            _ => (rest, None),
        };
        if name.is_empty() || version.is_some_and(str::is_empty) {
            return Err(LibError::new(cstr!("invalid item {raw:?}")));
        }
        Ok(Self {
            kind,
            name: name.into(),
            version: version.map(String::from),
        })
    }
}

/// A spec resolved to exactly one registry item.
#[derive(Debug, Clone)]
pub struct ResolvedSpec {
    pub kind: RegistryKind,
    pub name: String,
    pub version: Option<String>,
}

/// Resolve a spec against every candidate registry; errors when ambiguous.
///
/// Registries available without the network (`--registry`, installed
/// packages) are consulted first, so a name found locally never triggers an
/// `npm pack` of the other package.
pub fn resolve_spec(context: &mut LibContext, spec: &ItemSpec) -> LibResult<ResolvedSpec> {
    let kinds = spec
        .kind
        .map_or_else(|| RegistryKind::ALL.to_vec(), |kind| vec![kind]);
    let mut found: Vec<(RegistryKind, String)> = Vec::new();
    for kind in &kinds {
        let Some(registry) = context.resolver.local_registry(*kind) else {
            continue;
        };
        let version_matches = spec
            .version
            .as_deref()
            .is_none_or(|version| registry.manifest.package.version == version);
        if let Some(item) = registry.find(&spec.name).filter(|_| version_matches) {
            found.push((*kind, item.name.clone()));
        }
    }
    let mut failures: Vec<String> = Vec::new();
    if found.is_empty() {
        for kind in kinds {
            match context.resolver.registry(kind, spec.version.as_deref()) {
                Ok(registry) => {
                    if let Some(item) = registry.find(&spec.name) {
                        found.push((kind, item.name.clone()));
                    }
                }
                Err(error) => failures.push(error.message().into()),
            }
        }
    }
    match found.as_slice() {
        [(kind, name)] => Ok(ResolvedSpec {
            kind: *kind,
            name: name.clone(),
            version: spec.version.clone(),
        }),
        [] if !failures.is_empty() && spec.kind.is_some() => {
            Err(LibError::new(failures.join("; ")))
        }
        [] => {
            let mut message = cstr!("no registry item matches {:?}", spec.name);
            if !failures.is_empty() {
                message.push_str(&cstr!(" ({})", failures.join("; ")));
            }
            Err(LibError::new(message))
        }
        _ => {
            let candidates: Vec<String> = found
                .iter()
                .map(|(kind, name)| cstr!("{}:{name}", kind.as_str()))
                .collect();
            Err(LibError::new(cstr!(
                "{:?} is ambiguous; use one of {}",
                spec.name,
                candidates.join(", ")
            )))
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemSummary<'a> {
    kind: &'a str,
    name: &'a str,
    title: &'a str,
    description: &'a str,
    aliases: &'a [String],
    package: &'a str,
    version: &'a str,
}

fn registries(context: &mut LibContext, filter: Option<KindArg>) -> LibResult<Vec<RegistryKind>> {
    let mut available = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    let explicit = context.resolver.has_explicit();
    for kind in KindArg::kinds(filter) {
        match context.resolver.registry(kind, None) {
            Ok(_) => available.push(kind),
            Err(error) if filter.is_none() && explicit => drop(error),
            Err(error) => failures.push(error.message().into()),
        }
    }
    if available.is_empty() {
        return Err(LibError::new(if failures.is_empty() {
            String::from("no registry available")
        } else {
            String::from(failures.join("; "))
        }));
    }
    Ok(available)
}

fn render_items(
    context: &mut LibContext,
    kinds: &[RegistryKind],
    select: impl Fn(&LoadedRegistry) -> Vec<usize>,
) -> LibResult<String> {
    let mut summaries = Vec::new();
    let mut out = String::default();
    for kind in kinds {
        let registry = context.resolver.registry(*kind, None)?;
        let indices = select(registry);
        if !context.json {
            line(
                &mut out,
                format_args!("{} ({})", registry.package_label(), registry.origin),
            );
        }
        for index in indices {
            let Some(item) = registry.manifest.items.get(index) else {
                continue;
            };
            if context.json {
                summaries.push(serde_json::to_value(ItemSummary {
                    kind: &item.kind,
                    name: &item.name,
                    title: &item.title,
                    description: &item.description,
                    aliases: &item.aliases,
                    package: &registry.manifest.package.name,
                    version: &registry.manifest.package.version,
                }));
            } else {
                line(
                    &mut out,
                    format_args!("  {}:{:<28} {}", item.kind, item.name, item.description),
                );
            }
        }
    }
    if context.json {
        let items: Result<Vec<_>, _> = summaries.into_iter().collect();
        let items =
            items.map_err(|error| LibError::new(cstr!("failed to serialize items: {error}")))?;
        return json(&serde_json::json!({ "items": items }));
    }
    Ok(out)
}

pub fn list(context: &mut LibContext, filter: Option<KindArg>) -> LibResult<String> {
    let kinds = registries(context, filter)?;
    render_items(context, &kinds, |registry| {
        (0..registry.manifest.items.len()).collect()
    })
}

pub fn search(context: &mut LibContext, query: &str, filter: Option<KindArg>) -> LibResult<String> {
    let kinds = registries(context, filter)?;
    render_items(context, &kinds, |registry| {
        let matches = registry.search(query);
        registry
            .manifest
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| matches.iter().any(|found| found.name == item.name))
            .map(|(index, _)| index)
            .collect()
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemInfo<'a> {
    package: &'a str,
    version: &'a str,
    kind: &'a str,
    name: &'a str,
    title: &'a str,
    description: &'a str,
    aliases: &'a [String],
    package_subpath: &'a str,
    entry: &'a str,
    content_hash: &'a str,
    files: &'a [RegistryFile],
    registry_dependencies: &'a [String],
    dependencies: &'a [NpmDependency],
}

pub fn info(context: &mut LibContext, raw: &str) -> LibResult<String> {
    let spec = ItemSpec::parse(raw)?;
    let resolved = resolve_spec(context, &spec)?;
    let as_json = context.json;
    let registry = context
        .resolver
        .registry(resolved.kind, resolved.version.as_deref())?;
    let item = registry
        .item(&resolved.name)
        .ok_or_else(|| LibError::new(cstr!("unknown item {}", resolved.name)))?;
    if as_json {
        return json(&ItemInfo {
            package: &registry.manifest.package.name,
            version: &registry.manifest.package.version,
            kind: &item.kind,
            name: &item.name,
            title: &item.title,
            description: &item.description,
            aliases: &item.aliases,
            package_subpath: &item.package_subpath,
            entry: &item.entry,
            content_hash: &item.content_hash,
            files: &item.files,
            registry_dependencies: &item.registry_dependencies,
            dependencies: &item.dependencies,
        });
    }
    let mut out = String::default();
    line(
        &mut out,
        format_args!("{}:{} - {}", item.kind, item.name, item.title),
    );
    line(&mut out, format_args!("  {}", item.description));
    line(
        &mut out,
        format_args!("  package   {}", registry.package_label()),
    );
    line(
        &mut out,
        format_args!("  subpath   {}", item.package_subpath),
    );
    if !item.aliases.is_empty() {
        line(
            &mut out,
            format_args!("  aliases   {}", item.aliases.join(", ")),
        );
    }
    let requires = if item.registry_dependencies.is_empty() {
        String::from("(none)")
    } else {
        String::from(item.registry_dependencies.join(", "))
    };
    line(&mut out, format_args!("  requires  {requires}"));
    for dependency in &item.dependencies {
        line(
            &mut out,
            format_args!(
                "  npm       {}@{} ({})",
                dependency.name, dependency.range, dependency.kind
            ),
        );
    }
    line(&mut out, format_args!("  files"));
    for file in &item.files {
        line(
            &mut out,
            format_args!("    {} ({}, {} bytes)", file.path, file.role, file.size),
        );
    }
    Ok(out)
}
