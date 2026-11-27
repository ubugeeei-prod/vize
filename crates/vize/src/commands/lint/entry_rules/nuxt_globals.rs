//! Nuxt auto-import names for `vue/no-undefined-refs`, read once per project.

use std::path::PathBuf;

use vize_carton::nuxt::auto_imports;
use vize_l0::String;
use vize_patina::Linter;

const NO_UNDEFINED_REFS: &str = "vue/no-undefined-refs";

/// What each Nuxt project's generated declarations name, read on first use.
pub(super) struct NuxtTemplateGlobals<'a> {
    roots: &'a [PathBuf],
    /// `Some(None)` once a project is known to have no generated declarations.
    names: Vec<Option<Option<Vec<String>>>>,
}

impl<'a> NuxtTemplateGlobals<'a> {
    pub(super) fn new(roots: &'a [PathBuf]) -> Self {
        Self {
            roots,
            names: vec![None; roots.len()],
        }
    }

    /// Teach `linter` the names Nuxt auto-imports into the templates of the
    /// project at `root`, next to the `configured` ones.
    ///
    /// Until `nuxi prepare` has run nothing names them, and every auto-import
    /// would read as undefined. The rule is turned off for that project
    /// instead; `disabled` is the set the linter was built with.
    pub(super) fn apply(
        &mut self,
        linter: Linter,
        root: usize,
        configured: Option<&[String]>,
        mut disabled: Vec<String>,
    ) -> Linter {
        // A linter that never runs the rule needs no generated file read.
        if !linter.is_rule_enabled(NO_UNDEFINED_REFS)
            || !linter.registry().has_rule(NO_UNDEFINED_REFS)
        {
            return linter;
        }
        match self.names(root) {
            Some(names) => linter.with_undefined_ref_globals(
                configured
                    .unwrap_or_default()
                    .iter()
                    .chain(names)
                    .cloned()
                    .collect(),
            ),
            None => {
                disabled.push(NO_UNDEFINED_REFS.into());
                linter.with_disabled_rules(disabled)
            }
        }
    }

    fn names(&mut self, root: usize) -> Option<&[String]> {
        let directory = self.roots.get(root)?;
        self.names
            .get_mut(root)?
            .get_or_insert_with(|| {
                auto_imports::template_globals(directory)
                    .inspect_err(|declarations| {
                        eprintln!(
                            "[vize] {}",
                            auto_imports::missing_declarations_notice(directory, declarations)
                        );
                    })
                    .ok()
            })
            .as_deref()
    }
}
