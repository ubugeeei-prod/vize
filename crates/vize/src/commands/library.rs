//! `vize lib` - shadcn-style source distribution for `@vizejs/ui` and
//! `@vizejs/composable`.
//!
//! Items are copied as source into the project from the versioned registry
//! each package ships in its npm tarball (`registry/registry.json`). A
//! lockfile records the package version and per-file digests at pull time so
//! later `status`, `diff`, and `update` runs can separate local edits from
//! upstream changes.

mod error;
mod fs_ops;
mod inspect;
mod install;
mod lockfile;
mod output;
mod plan;
mod query;
mod registry;
mod remove;
mod resolve;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand, ValueEnum};
use vize_s0::String;

use self::error::{LibError, LibResult};
use self::lockfile::{DEFAULT_LOCKFILE, Lockfile};
use self::resolve::{RegistryKind, Resolver};

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

    /// npm executable used for `npm pack` (default: $VIZE_LIB_NPM, then npm)
    #[arg(long, global = true, hide = true, value_name = "PROGRAM")]
    pub npm: Option<PathBuf>,

    #[command(subcommand)]
    pub command: LibCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LibCommand {
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
        /// Item name or alias, optionally `ui:` / `composable:` prefixed and `@version` suffixed
        name: String,
    },
    /// Copy items (and their registry dependencies) into the project
    Pull(PullArgs),
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
    /// Only show one registry kind
    #[arg(long, value_enum)]
    pub kind: Option<KindArg>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindArg {
    Ui,
    Composable,
}

impl KindArg {
    fn kinds(filter: Option<Self>) -> Vec<RegistryKind> {
        match filter {
            Some(Self::Ui) => vec![RegistryKind::Ui],
            Some(Self::Composable) => vec![RegistryKind::Composable],
            None => RegistryKind::ALL.to_vec(),
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct PullArgs {
    /// Items to pull: `name`, `name@version`, `ui:name`, `composable:name@version`
    #[arg(required = true)]
    pub items: Vec<String>,
    /// Target directory (default: vize.config `lib` section, then the registry default)
    #[arg(long)]
    pub dir: Option<PathBuf>,
    /// Print the plan without writing files
    #[arg(long)]
    pub dry_run: bool,
    /// Overwrite files that differ from the registry and were not pulled pristine
    #[arg(long)]
    pub overwrite: bool,
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
        let config = crate::config::load_lib_config_with_source(Some(&root)).config;
        let lock_path = root.join(config.lockfile.as_deref().unwrap_or(DEFAULT_LOCKFILE));
        let registry_paths: Vec<PathBuf> = args
            .registry
            .iter()
            .map(|path| absolutize(&root, path))
            .collect();
        let mut resolver = Resolver::new(&root, &registry_paths, args.offline)?;
        if let Some(npm) = &args.npm {
            resolver.set_npm_program(npm.as_os_str().to_owned());
        }
        Ok(Self {
            root,
            lock_path,
            config,
            resolver,
            json: args.json,
        })
    }

    pub fn lockfile(&self) -> LibResult<Lockfile> {
        Lockfile::read(&self.lock_path)
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
        LibCommand::List(filter) => query::list(&mut context, filter.kind),
        LibCommand::Search { query, filter } => query::search(&mut context, query, filter.kind),
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
