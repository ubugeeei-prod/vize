//! Names Nuxt auto-imports into templates, read once per Nuxt project.
//!
//! The linter is rebuilt on every diagnostics pass, so the generated
//! declarations are read on first use and kept until a watched file event
//! says Nuxt rewrote them or a project boundary moved.

use std::{path::Path, sync::Arc};

use parking_lot::Mutex;
use tower_lsp::lsp_types::Url;
use vize_carton::nuxt::{
    NUXT_CONFIG_NAMES,
    auto_imports::{self, NuxtRoots},
};
use vize_l0::String;

use super::ServerState;

/// What the Nuxt project that owns a document declares for its templates.
#[derive(Clone)]
pub(crate) enum NuxtTemplateNames {
    Generated(Arc<[String]>),
    /// Nuxt has not written its declarations yet.
    NotGenerated,
}

#[derive(Default)]
pub(super) struct NuxtTemplateGlobals {
    projects: Mutex<Projects>,
}

#[derive(Default)]
struct Projects {
    roots: NuxtRoots,
    /// Indexed like `roots`; `None` until that project is first read.
    names: Vec<Option<NuxtTemplateNames>>,
    /// Log lines the server has not sent to the client yet.
    notices: Vec<String>,
}

impl ServerState {
    /// The template names of the Nuxt project at or above `directory`, or
    /// `None` when no Nuxt project owns it.
    pub(crate) fn nuxt_template_names(&self, directory: &Path) -> Option<NuxtTemplateNames> {
        let mut projects = self.nuxt_template_globals.projects.lock();
        let Projects {
            roots,
            names,
            notices,
        } = &mut *projects;
        let index = roots.nearest(directory)?;
        if names.len() <= index {
            names.resize(index + 1, None);
        }
        let entry = names.get_mut(index)?;
        if let Some(cached) = entry {
            return Some(cached.clone());
        }
        let root = roots.dir(index)?;
        let read = match auto_imports::template_globals(root) {
            Ok(generated) => NuxtTemplateNames::Generated(generated.into()),
            Err(declarations) => {
                notices.push(auto_imports::missing_declarations_notice(
                    root,
                    &declarations,
                ));
                NuxtTemplateNames::NotGenerated
            }
        };
        Some(entry.insert(read).clone())
    }

    /// Forget every Nuxt project when a watched change may regenerate its
    /// declarations or move a project boundary. Returns whether a Nuxt project
    /// was known, so open documents need their diagnostics again.
    pub(crate) fn invalidate_nuxt_template_names<'a>(
        &self,
        changes: impl IntoIterator<Item = &'a Url>,
    ) -> bool {
        if !changes.into_iter().any(moves_nuxt_template_names) {
            return false;
        }
        let mut projects = self.nuxt_template_globals.projects.lock();
        let known = !projects.names.is_empty();
        projects.roots = NuxtRoots::default();
        projects.names.clear();
        known
    }

    /// Log lines about skipped Nuxt projects that are still unsent.
    pub(crate) fn take_nuxt_notices(&self) -> Vec<String> {
        std::mem::take(&mut self.nuxt_template_globals.projects.lock().notices)
    }
}

/// Nuxt's generated `types/imports.d.ts`, a Nuxt config (which can move
/// `buildDir`), or a file that decides which project owns a directory.
fn moves_nuxt_template_names(uri: &Url) -> bool {
    let Ok(path) = uri.to_file_path() else {
        return false;
    };
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    // An install rewrites every manifest under node_modules, and none of them
    // owns an authored document.
    if path
        .components()
        .any(|component| component.as_os_str() == "node_modules")
    {
        return false;
    }
    match name {
        "imports.d.ts" => path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|parent| parent == "types"),
        "package.json" => true,
        name => NUXT_CONFIG_NAMES.contains(&name) || name.starts_with("vite.config."),
    }
}

#[cfg(test)]
mod tests;
