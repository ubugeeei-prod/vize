//! `vize lib` - shadcn-style source distribution for `@vizejs/ui` and
//! `@vizejs/composable`.
//!
//! Items are copied as source into the project from the versioned registry
//! each package ships in its npm tarball (`registry/registry.json`). A
//! lockfile records the package version and per-file digests at pull time so
//! later `status`, `diff`, and `update` runs can separate local edits from
//! upstream changes.

mod error;
mod fetch;
mod fs_ops;
mod init;
mod inspect;
mod install;
mod lockfile;
mod outdated;
mod output;
mod plan;
mod query;
mod registry;
mod remove;
mod resolve;
#[cfg(test)]
mod tests;
mod validate;

use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use clap::{Args, Subcommand};
use vize_s0::String;

use self::error::{LibError, LibResult};
use self::fs_ops::ensure_project_path;
use self::lockfile::{DEFAULT_LOCKFILE, Lockfile};
use self::resolve::{Namespace, Origin, Resolver, Source};

#[derive(Args, Debug, Clone)]
pub struct LibArgs {
    /// Project root (config lookup, lockfile, and target directories are relative to it)
    #[arg(long, global = true, default_value = ".")]
    pub root: PathBuf,

    /// Registry to use instead of auto-discovery: a registry.json, its directory,
    /// or an unpacked package directory. Repeat for ui and composable.
    #[arg(long, global = true, value_name = "PATH")]
    pub registry: Vec<PathBuf>,

    /// Never run `npm pack`; only use --registry or installed packages
    #[arg(long, global = true)]
    pub offline: bool,

    /// Print machine-readable JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// npm executable used for `npm pack` / `npm view` (default: $VIZE_LIB_NPM, then npm)
    #[arg(long, global = true, hide = true, value_name = "PROGRAM")]
    pub npm: Option<PathBuf>,

    /// curl executable used for https registries (default: $VIZE_LIB_CURL, then curl)
    #[arg(long, global = true, hide = true, value_name = "PROGRAM")]
    pub curl: Option<PathBuf>,

    #[command(subcommand)]
    pub command: LibCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LibCommand {
    /// Write the `lib` config section for this project (run with --dry-run first)
    Init(InitArgs),
    /// List pullable items
    List(KindFilter),
    /// Search items by name, title, description, or alias
    Search {
        /// Words that must all match
        query: String,
        #[command(flatten)]
        filter: KindFilter,
    },
    /// Show one item: files, dependencies, and version
    Info {
        /// Item name or alias: `name`, `ui:name`, `@ns/name`, optionally `@version` suffixed
        name: String,
    },
    /// Copy items (and their registry dependencies) into the project
    #[command(visible_alias = "add")]
    Pull(PullArgs),
    /// Show pulled items with newer registry versions
    Outdated,
    /// Compare pulled items with the lockfile and the available registry
    Status,
    /// Show local changes against a registry version of an item
    Diff {
        /// Pulled item name
        name: String,
        /// Registry version to compare against (default: installed or latest)
        #[arg(long)]
        to: Option<String>,
    },
    /// Update pulled items without clobbering local edits
    Update(UpdateArgs),
    /// Delete pulled items and orphaned dependencies
    Remove(RemoveArgs),
}

#[derive(Args, Debug, Clone, Default)]
pub struct KindFilter {
    /// Only show one source: `ui`, `composable`, or a configured `@namespace`
    #[arg(long, value_parser = parse_source)]
    pub kind: Option<Source>,
}

fn parse_source(value: &str) -> Result<Source, String> {
    Source::parse(value)
        .ok_or_else(|| vize_s0::cstr!("{value:?} is not ui, composable, or an @namespace"))
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Directory for @vizejs/ui items (default: detected, e.g. src/components/vize)
    #[arg(long)]
    pub ui_dir: Option<String>,
    /// Directory for @vizejs/composable items (default: detected, e.g. src/composables/vize)
    #[arg(long)]
    pub composable_dir: Option<String>,
    /// Print the detected setup and the config change without writing
    #[arg(long)]
    pub dry_run: bool,
    /// Replace an existing `lib` section in vize.config.json
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct PullArgs {
    /// Items to pull: `name`, `name@version`, `ui:name`, `composable:name@version`, `@ns/name`
    #[arg(required = true)]
    pub items: Vec<String>,
    /// Target directory (default: vize.config `lib` section, then the registry default)
    #[arg(long, short = 'p', visible_alias = "path")]
    pub dir: Option<PathBuf>,
    /// Print the plan without writing files
    #[arg(long)]
    pub dry_run: bool,
    /// Overwrite files that differ from the registry and were not pulled pristine
    #[arg(long, short = 'o')]
    pub overwrite: bool,
    /// Accepted for shadcn compatibility; `vize lib` never prompts
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateArgs {
    /// Items to update (default: every pulled item)
    pub items: Vec<String>,
    /// Target package version (default: installed or latest)
    #[arg(long)]
    pub to: Option<String>,
    /// Overwrite locally modified files that changed upstream
    #[arg(long)]
    pub force: bool,
    /// Print the plan without writing files
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub struct RemoveArgs {
    /// Items to remove
    #[arg(required = true)]
    pub items: Vec<String>,
    /// Delete locally modified files and ignore dependents
    #[arg(long)]
    pub force: bool,
    /// Print what would be removed without deleting
    #[arg(long)]
    pub dry_run: bool,
}

/// Shared state for one `vize lib` invocation.
pub struct LibContext {
    pub root: PathBuf,
    pub config_path: Option<PathBuf>,
    pub lock_path: PathBuf,
    pub config: crate::config::LibConfig,
    pub resolver: Resolver,
    pub json: bool,
}

impl LibContext {
    pub fn new(args: &LibArgs) -> LibResult<Self> {
        let root = args
            .root
            .canonicalize()
            .map_err(|error| LibError::io("open project root", &args.root, &error))?;
        let loaded = crate::config::load_lib_config_with_source(Some(&root));
        let config = loaded.config;
        let config_dir = loaded
            .source_path
            .as_deref()
            .and_then(Path::parent)
            .map_or_else(|| root.clone(), Path::to_path_buf);
        let mut namespaces = BTreeMap::new();
        for (name, entry) in &config.registries {
            if !resolve::is_namespace(name) {
                return Err(LibError::new(vize_s0::cstr!(
                    "lib.registries key {name:?} must be an @namespace"
                )));
            }
            namespaces.insert(
                name.clone(),
                Namespace {
                    origin: Origin::parse(entry.source(), &config_dir)?,
                    dir: entry.dir().map(String::from),
                },
            );
        }
        let lock_path = root.join(config.lockfile.as_deref().unwrap_or(DEFAULT_LOCKFILE));
        ensure_project_path(&root, &lock_path)?;
        let registry_paths: Vec<PathBuf> = args
            .registry
            .iter()
            .map(|path| absolutize(&root, path))
            .collect();
        let mut resolver = Resolver::new(&root, &registry_paths, namespaces, args.offline)?;
        if let Some(npm) = &args.npm {
            resolver.tools_mut().npm = npm.as_os_str().to_owned();
        }
        if let Some(curl) = &args.curl {
            resolver.tools_mut().curl = curl.as_os_str().to_owned();
        }
        Ok(Self {
            root,
            config_path: loaded.source_path,
            lock_path,
            config,
            resolver,
            json: args.json,
        })
    }

    pub fn lockfile(&self) -> LibResult<Lockfile> {
        Lockfile::read(&self.root, &self.lock_path)
    }
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| root.join(path))
}

/// Execute a parsed `vize lib` command and return its rendered output.
pub fn execute(args: &LibArgs) -> LibResult<String> {
    let mut context = LibContext::new(args)?;
    match &args.command {
        LibCommand::Init(init) => init::init(&context, init),
        LibCommand::List(filter) => query::list(&mut context, filter.kind.as_ref()),
        LibCommand::Search { query, filter } => {
            query::search(&mut context, query, filter.kind.as_ref())
        }
        LibCommand::Outdated => outdated::outdated(&mut context),
        LibCommand::Info { name } => query::info(&mut context, name),
        LibCommand::Pull(pull) => install::pull(&mut context, pull),
        LibCommand::Status => inspect::status(&mut context),
        LibCommand::Diff { name, to } => inspect::diff(&mut context, name, to.as_deref()),
        LibCommand::Update(update) => install::update(&mut context, update),
        LibCommand::Remove(remove) => remove::remove(&mut context, remove),
    }
}

pub fn run(args: LibArgs) {
    match execute(&args) {
        Ok(rendered) => print!("{rendered}"),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
