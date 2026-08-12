//! Declaration-scope boundary between a classic `<script>` and `<script setup>`.
//!
//! Vue compiles the two blocks into two different scopes: classic `<script>`
//! declarations stay at module scope, `<script setup>` declarations become
//! locals of the component's `setup()`. So setup *sees* module scope, a
//! setup-local declaration *shadows* a same-named module-scope one, and the
//! classic block cannot see setup values at all.
//!
//! The generator has to reproduce that split. Collapsing both blocks into one
//! synthetic scope emits an authored name twice (TS2300) or splits one merged
//! name across an exported and a local declaration (TS2395) — neither of which
//! `vue-tsc` reports.

use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::{Croquis, ScopeKind, TypeExport};

use super::spans::merge_overlapping_spans;

/// Where each authored declaration of a two-block SFC lives once the virtual
/// module is generated.
#[derive(Default)]
pub(super) struct ScriptBlockScopes {
    /// Byte ranges of the merged script that the classic `<script>` block owns.
    /// Empty unless the SFC has both blocks: a lone `<script>` is emitted
    /// *inside* `__setup()` like setup code, so it has no module-scope region.
    classic_spans: Vec<(u32, u32)>,
    /// Names the classic block already declares at module scope.
    classic_names: FxHashSet<CompactString>,
}

impl ScriptBlockScopes {
    pub(super) fn collect(summary: &Croquis, has_script_setup: bool) -> Self {
        if !has_script_setup {
            return Self::default();
        }
        let classic_spans: Vec<(u32, u32)> = summary
            .scopes
            .iter()
            .filter(|scope| matches!(scope.kind, ScopeKind::NonScriptSetup))
            .map(|scope| (scope.span.start, scope.span.end))
            .collect();
        if classic_spans.is_empty() {
            return Self::default();
        }

        let mut scopes = Self {
            classic_spans,
            classic_names: FxHashSet::default(),
        };
        for export in &summary.type_exports {
            if scopes.owns(export.start) {
                scopes.classic_names.insert(export.name.clone());
            }
        }
        for (name, (start, _)) in &summary.binding_spans {
            if scopes.owns(*start) {
                scopes.classic_names.insert(name.clone());
            }
        }
        scopes
    }

    /// Whether the declaration starting at `start` was authored in the classic
    /// `<script>` block.
    pub(super) fn owns(&self, start: u32) -> bool {
        self.classic_spans
            .iter()
            .any(|&(span_start, span_end)| start >= span_start && start < span_end)
    }

    /// Whether a `<script setup>` type declaration only shadows a classic-block
    /// name and must therefore stay inside `__setup()`.
    ///
    /// An *exported* setup declaration is excluded: Vue lifts it to module
    /// scope, where colliding with the classic declaration is a genuine
    /// duplicate that has to keep both authored ranges.
    fn shadows_classic_name(&self, export: &TypeExport, script: Option<&str>) -> bool {
        if self.classic_spans.is_empty()
            || self.owns(export.start)
            || !self.classic_names.contains(&export.name)
        {
            return false;
        }
        !script
            .and_then(|script| script.get(export.start as usize..export.end as usize))
            .is_some_and(|source| source.starts_with("export"))
    }

    /// Module-scope spans contributed by the classic block plus every type
    /// declaration that genuinely belongs at module scope.
    pub(super) fn module_spans(
        &self,
        summary: &Croquis,
        script: Option<&str>,
        extra: Vec<(u32, u32)>,
    ) -> Vec<(u32, u32)> {
        let mut spans = extra;
        spans.extend(self.classic_spans.iter().copied());
        for export in &summary.type_exports {
            // Non-hoisted types reference setup-scope values via `typeof` and
            // must stay inside `__setup` so TS can resolve them.
            if export.hoisted && !self.shadows_classic_name(export, script) {
                spans.push((export.start, export.end));
            }
        }
        merge_overlapping_spans(spans)
    }

    /// Name lookup for the hoisted type declarations that may need SFC generic
    /// parameters spliced back in.
    pub(super) fn hoisted_type_spans<'a>(
        &self,
        summary: &'a Croquis,
        script: Option<&str>,
    ) -> FxHashMap<(u32, u32), &'a str> {
        summary
            .type_exports
            .iter()
            .filter(|export| export.hoisted && !self.shadows_classic_name(export, script))
            .map(|export| ((export.start, export.end), export.name.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptBlockScopes;
    use vize_croquis::{Croquis, NonScriptSetupScopeData, TypeExport, TypeExportKind};

    /// A merged script exactly as `analyze_scripts` builds it: the classic
    /// block's bytes, a separator newline, then the setup block's bytes.
    const CLASSIC: &str = "export type Kind = string;\n";
    const SETUP_LOCAL: &str = "type Kind = string;\n";
    const SETUP_EXPORTED: &str = "export type Kind = string;\n";

    fn merged(setup: &str) -> (String, u32) {
        (format!("{CLASSIC}\n{setup}"), CLASSIC.len() as u32)
    }

    fn summary_of(setup: &str) -> (Croquis, String, u32) {
        let (script, classic_len) = merged(setup);
        let setup_start = classic_len + 1;
        let mut summary = Croquis::new();
        summary.scopes.enter_non_script_setup_scope(
            NonScriptSetupScopeData {
                is_ts: true,
                has_define_component: false,
            },
            0,
            classic_len,
        );
        summary.scopes.exit_scope();
        for (start, end) in [
            (0, CLASSIC.len() as u32 - 1),
            (setup_start, setup_start + setup.len() as u32 - 1),
        ] {
            summary.type_exports.push(TypeExport {
                name: "Kind".into(),
                kind: TypeExportKind::Type,
                start,
                end,
                hoisted: true,
            });
        }
        (summary, script, classic_len)
    }

    #[test]
    fn classic_declarations_are_owned_by_the_module_scope_region() {
        let (summary, _, classic_len) = summary_of(SETUP_LOCAL);
        let scopes = ScriptBlockScopes::collect(&summary, true);

        assert!(scopes.owns(0));
        assert!(!scopes.owns(classic_len + 1));
    }

    #[test]
    fn a_setup_local_type_shadowing_a_classic_name_stays_in_setup() {
        let (summary, script, classic_len) = summary_of(SETUP_LOCAL);
        let scopes = ScriptBlockScopes::collect(&summary, true);

        // Only the classic declaration is emitted at module scope; the
        // setup-local one keeps Vue's shadowing instead of colliding there.
        assert_eq!(
            scopes.module_spans(&summary, Some(script.as_str()), Vec::new()),
            [(0, classic_len)]
        );
        assert_eq!(
            scopes
                .hoisted_type_spans(&summary, Some(script.as_str()))
                .into_iter()
                .collect::<Vec<_>>(),
            [((0, 26), "Kind")]
        );
    }

    #[test]
    fn an_exported_setup_type_still_reaches_module_scope() {
        let (summary, script, classic_len) = summary_of(SETUP_EXPORTED);
        let scopes = ScriptBlockScopes::collect(&summary, true);

        // Vue lifts an exported setup declaration to module scope too, so both
        // authored ranges must stay there for the genuine duplicate report.
        assert_eq!(
            scopes.module_spans(&summary, Some(script.as_str()), Vec::new()),
            [(0, classic_len), (classic_len + 1, script.len() as u32 - 1)]
        );
    }

    #[test]
    fn a_single_script_block_keeps_every_type_at_module_scope() {
        let (summary, script, classic_len) = summary_of(SETUP_LOCAL);
        let scopes = ScriptBlockScopes::collect(&summary, false);

        assert!(!scopes.owns(0));
        assert_eq!(
            scopes.module_spans(&summary, Some(script.as_str()), Vec::new()),
            [(0, 26), (classic_len + 1, script.len() as u32 - 1)]
        );
    }
}
