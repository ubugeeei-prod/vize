//! Clean command - Remove Vize-generated cache artifacts.

use std::path::{Path, PathBuf};
use std::{fs, io::ErrorKind};

#[derive(clap::Args, Debug, Clone)]
pub struct CleanArgs {
    /// Project root whose node_modules/.vize cache should be removed
    #[arg(default_value = ".")]
    pub root: PathBuf,

    /// Print the cache path without deleting it
    #[arg(long)]
    pub dry_run: bool,

    /// Suppress status output
    #[arg(short, long)]
    pub quiet: bool,
}

pub fn run(args: CleanArgs) {
    let root = args.root.canonicalize().unwrap_or(args.root);
    let cache_dir = vize_cache_dir(&root);

    if args.dry_run {
        if !args.quiet {
            println!("{}", cache_dir.display());
        }
        return;
    }

    match fs::remove_dir_all(&cache_dir) {
        Ok(()) => {
            if !args.quiet {
                println!("Removed {}", cache_dir.display());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            if !args.quiet {
                println!("No Vize cache found at {}", cache_dir.display());
            }
        }
        Err(error) => {
            eprintln!("Failed to remove {}: {}", cache_dir.display(), error);
            std::process::exit(1);
        }
    }
}

fn vize_cache_dir(root: &Path) -> PathBuf {
    root.join("node_modules").join(".vize")
}

#[cfg(test)]
mod tests {
    use super::vize_cache_dir;
    use std::path::Path;

    #[test]
    fn vize_cache_dir_points_under_node_modules() {
        assert_eq!(
            vize_cache_dir(Path::new("/project")),
            Path::new("/project").join("node_modules").join(".vize")
        );
    }
}
