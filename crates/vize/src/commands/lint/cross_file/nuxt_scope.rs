//! Host-only, once-invocation Nuxt scope admission.
//!
//! Source facts alone never establish complete coverage. This intentionally
//! narrow supported runtime needs bounded physical inventories and explicit
//! absence of opaque loader inputs before the ID consumer may use them.

mod inventory;
#[cfg(all(test, not(windows)))]
mod tests;

use std::path::Path;
use vize_croquis_cf::{CrossFileAnalyzer, NuxtRenderDirs};
use vize_l0::profiler::global_profiler;

use crate::commands::project_nuxt::host_authority::{
    authority::RuntimeAuthority,
    io::{self, ProbeIo},
    rc::Environment,
};

pub(super) fn admitted(
    root: &Path,
    configured: &NuxtRenderDirs,
    analyzer: &CrossFileAnalyzer,
) -> bool {
    let environment = Environment::capture();
    admit_with_environment(root, configured, analyzer, &environment).0
}

fn admit_with_environment(
    root: &Path,
    configured: &NuxtRenderDirs,
    analyzer: &CrossFileAnalyzer,
    environment: &Environment,
) -> (bool, io::IoCounts) {
    let mut io = ProbeIo::default();
    let accepted = (|| {
        if cfg!(windows)
            || !configured.pages_enabled
            || !configured.static_render_config
            || environment.opaque
            || !analyzer.has_closed_nuxt_render_facts()
        {
            return None;
        }
        let runtime = RuntimeAuthority::read(root, &mut io)?;
        inventory::complete(root, configured, analyzer.registry(), &mut io)?;
        environment.absent(root, &mut io)?;
        // Every manifest cache entry is scoped to this attempt. No completeness
        // witness survives into another invocation or untracked input revision.
        if runtime.witness_bytes() != io.counts.manifest_bytes {
            return None;
        }
        Some(())
    })()
    .is_some();
    global_profiler().record_counter("cli.lint.nuxt_scope.proven", u64::from(accepted));
    (accepted, io.counts.clone())
}
