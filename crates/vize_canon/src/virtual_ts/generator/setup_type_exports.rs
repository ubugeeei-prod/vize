use std::borrow::Cow;

use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::Croquis;

use super::script_blocks::ScriptBlockScopes;
use crate::virtual_ts::VizeMapping;

fn skip_trivia(source: &str, mut at: usize) -> usize {
    let bytes = source.as_bytes();
    loop {
        while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        if source.get(at..).is_some_and(|rest| rest.starts_with("//")) {
            at = source
                .get(at..)
                .and_then(|rest| rest.find('\n'))
                .map_or(source.len(), |end| at + end + 1);
        } else if source.get(at..).is_some_and(|rest| rest.starts_with("/*")) {
            at = source
                .get(at + 2..)
                .and_then(|rest| rest.find("*/"))
                .map_or(source.len(), |end| at + end + 4);
        } else {
            return at;
        }
    }
}

fn keyword_end(source: &str, at: usize, keyword: &str) -> Option<usize> {
    let rest = source.get(at..)?.strip_prefix(keyword)?;
    (!rest
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')))
    .then_some(at + keyword.len())
}

fn exported_type_is_generic(source: &str, name: &str) -> bool {
    let Some(mut at) = keyword_end(source, 0, "export") else {
        return false;
    };
    at = skip_trivia(source, at);
    if let Some(end) = keyword_end(source, at, "declare") {
        at = skip_trivia(source, end);
    }
    let Some(end) =
        keyword_end(source, at, "type").or_else(|| keyword_end(source, at, "interface"))
    else {
        return false;
    };
    at = skip_trivia(source, end);
    let Some(rest) = source.get(at..).and_then(|rest| rest.strip_prefix(name)) else {
        return false;
    };
    rest.get(skip_trivia(rest, 0)..)
        .is_some_and(|rest| rest.starts_with('<'))
}

struct SetupTypeExport {
    name: String,
    artifact: String,
}

/// Preserves public types whose `typeof` dependencies keep them in setup scope.
pub(super) struct SetupTypeExportsPlan {
    diagnostic_starts: Vec<u32>,
    classic_export_starts: Vec<u32>,
    exports: Vec<SetupTypeExport>,
}

impl SetupTypeExportsPlan {
    pub(super) fn new(
        summary: &Croquis,
        script: Option<&str>,
        script_blocks: &ScriptBlockScopes,
        has_script_setup: bool,
    ) -> Self {
        let diagnostic_starts = if has_script_setup {
            script
                .into_iter()
                .flat_map(|script| {
                    summary
                        .type_exports
                        .iter()
                        .filter(move |item| item.hoisted)
                        .filter_map(move |declaration| {
                            let source =
                                script.get(declaration.start as usize..declaration.end as usize)?;
                            (source.starts_with("export") && !script_blocks.owns(declaration.start))
                                .then_some(declaration.start)
                        })
                })
                .collect()
        } else {
            Vec::new()
        };
        let mut classic_export_starts = Vec::new();
        let mut exports = Vec::new();
        let mut captured_names: FxHashSet<String> = FxHashSet::default();

        if let Some(script) = script {
            for declaration in summary.type_exports.iter().filter(|item| !item.hoisted) {
                let start = declaration.start as usize;
                let end = declaration.end as usize;
                let Some(source) = script.get(start..end) else {
                    continue;
                };
                if !source.starts_with("export") {
                    continue;
                }
                // A classic `<script>` declaration is emitted verbatim at module
                // scope, `export` modifier and all, next to the module-scope
                // values it depends on. Capturing it as well would re-declare
                // the same authored name — TS2300 on code `vue-tsc` accepts.
                if script_blocks.owns(declaration.start) {
                    continue;
                }
                if !has_script_setup {
                    classic_export_starts.push(declaration.start);
                }

                let name: String = declaration.name.as_str().into();
                // Generic declarations cannot ride the non-generic
                // `undefined as unknown as T` capture path: it drops their type
                // parameters, and a value-dependent generic cannot be
                // reconstructed at module scope. Keep the authored modifier so
                // TypeScript reports TS1184 inside `__setup`, matching vue-tsc,
                // and skip artifact capture.
                if exported_type_is_generic(source, &name) {
                    continue;
                }
                // Legal declaration merging (e.g. two `export interface
                // Options`) surfaces the same name twice. Capture a single
                // artifact per exported name so the generated module has no
                // duplicate bindings or aliases.
                if !captured_names.insert(name.clone()) {
                    continue;
                }
                exports.push(SetupTypeExport {
                    artifact: cstr!("__vize_exported_type_{name}"),
                    name,
                });
            }
        }
        classic_export_starts.sort_unstable();

        Self {
            diagnostic_starts,
            classic_export_starts,
            exports,
        }
    }

    /// Whether `name` is captured as an explicit public re-export at module
    /// scope. Used to distinguish a public `export type Props` from a private
    /// setup-scoped `type Props`, which is never re-exported.
    pub(super) fn exports_public_type(&self, name: &str) -> bool {
        self.exports
            .iter()
            .any(|export| export.name.as_str() == name)
    }

    /// A lone classic script is projected through `__setup`, so remove only
    /// its legal module modifier and restore the public declaration through the
    /// captured artifact. Actual `<script setup>` modifiers remain for TS1184.
    pub(super) fn strip_classic_modifiers(&self, line: &mut Cow<'_, str>, line_start: u32) {
        let line_end = line_start.saturating_add(line.len() as u32);
        let first = self
            .classic_export_starts
            .partition_point(|start| *start < line_start);
        let last = self
            .classic_export_starts
            .partition_point(|start| *start < line_end);
        if first == last {
            return;
        }

        let original = line.as_ref();
        let mut output = String::with_capacity(original.len());
        let mut copied_until = 0usize;
        for start in self
            .classic_export_starts
            .get(first..last)
            .unwrap_or_default()
        {
            let column = (*start - line_start) as usize;
            if original.get(column..column + "export".len()) == Some("export")
                && let Some(kept) = original.get(copied_until..column)
            {
                output.push_str(kept);
                copied_until = column + "export".len();
            }
        }
        output.push_str(original.get(copied_until..).unwrap_or_default());
        *line = Cow::Owned(output.into());
    }

    /// A hoisted setup type is legal in the generated module, but its authored
    /// `export` modifier is not legal in `<script setup>`. Keep one anonymous,
    /// collision-free modifier in setup scope and map only that token back to
    /// the authored modifier so TypeScript reports TS1184 at the correct site.
    pub(super) fn emit_diagnostic_anchors(
        &self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        for &start in &self.diagnostic_starts {
            let generated_start = ts.len() + 2;
            ts.push_str("  export default class {}\n");
            mappings.push(VizeMapping::new(
                generated_start..generated_start + "export".len(),
                source_offset(start as usize)..source_offset(start as usize + "export".len()),
            ));
        }
    }

    pub(super) fn emit_setup_artifacts(&self, ts: &mut String, fields: &mut Vec<String>) {
        for exported in &self.exports {
            let name = &exported.name;
            let artifact = &exported.artifact;
            append!(
                *ts,
                "\n  const {artifact} = undefined as unknown as {name};\n"
            );
            fields.push(artifact.clone());
        }
    }

    pub(super) fn emit_module_exports(&self, ts: &mut String) {
        for exported in &self.exports {
            let name = &exported.name;
            let artifact = &exported.artifact;
            append!(
                *ts,
                "export type {name} = Awaited<ReturnType<typeof __setup>>[\"{artifact}\"];\n"
            );
        }
        if !self.exports.is_empty() {
            ts.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{ScriptBlockScopes, SetupTypeExportsPlan, exported_type_is_generic};
    use vize_croquis::{Croquis, TypeExport, TypeExportKind};

    #[test]
    fn only_explicit_non_hoisted_exports_are_planned() {
        let script = "type Local = typeof value;\nexport type Public = typeof value;";
        let public_start = script.find("export").unwrap() as u32;
        let mut summary = Croquis::new();
        summary.type_exports.push(TypeExport {
            name: "Local".into(),
            kind: TypeExportKind::Type,
            start: 0,
            end: 26,
            hoisted: false,
        });
        summary.type_exports.push(TypeExport {
            name: "Public".into(),
            kind: TypeExportKind::Type,
            start: public_start,
            end: script.len() as u32,
            hoisted: false,
        });

        let plan =
            SetupTypeExportsPlan::new(&summary, Some(script), &ScriptBlockScopes::default(), true);
        let mut fields = Vec::new();
        let mut setup = vize_carton::String::default();
        plan.emit_setup_artifacts(&mut setup, &mut fields);
        assert!(setup.contains("unknown as Public"));
        assert_eq!(fields, ["__vize_exported_type_Public"]);
    }

    #[test]
    fn generic_exports_are_not_widened_by_a_non_generic_capture() {
        let source = "export /* public */ type Box<T extends typeof value> = { value: T }";
        assert!(exported_type_is_generic(source, "Box"));

        let mut summary = Croquis::new();
        summary.type_exports.push(TypeExport {
            name: "Box".into(),
            kind: TypeExportKind::Type,
            start: 0,
            end: source.len() as u32,
            hoisted: false,
        });
        let plan =
            SetupTypeExportsPlan::new(&summary, Some(source), &ScriptBlockScopes::default(), true);
        // It is never captured through the non-generic artifact path, which
        // would drop its type parameters. The authored modifier remains in the
        // setup scope so TypeScript reports TS1184.
        let mut fields = Vec::new();
        let mut setup = vize_carton::String::default();
        plan.emit_setup_artifacts(&mut setup, &mut fields);
        assert!(fields.is_empty());
        assert!(setup.is_empty());
    }

    #[test]
    fn hoisted_setup_exports_keep_a_mapped_modifier_diagnostic_anchor() {
        let source = "export interface Public {}";
        let mut summary = Croquis::new();
        summary.type_exports.push(TypeExport {
            name: "Public".into(),
            kind: TypeExportKind::Interface,
            start: 0,
            end: source.len() as u32,
            hoisted: true,
        });

        let plan =
            SetupTypeExportsPlan::new(&summary, Some(source), &ScriptBlockScopes::default(), true);
        let mut setup = vize_carton::String::default();
        let mut mappings = Vec::new();
        plan.emit_diagnostic_anchors(&mut setup, &mut mappings, &|offset| offset + 10);
        assert_eq!(setup, "  export default class {}\n");
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].gen_range, 2..8);
        assert_eq!(mappings[0].src_range, 10..16);

        let plain =
            SetupTypeExportsPlan::new(&summary, Some(source), &ScriptBlockScopes::default(), false);
        setup.clear();
        mappings.clear();
        plain.emit_diagnostic_anchors(&mut setup, &mut mappings, &|offset| offset + 10);
        assert!(setup.is_empty());
        assert!(mappings.is_empty());
    }

    #[test]
    fn classic_script_exports_use_the_legal_capture_bridge() {
        let source = "export type Public = typeof value;";
        let mut summary = Croquis::new();
        summary.type_exports.push(TypeExport {
            name: "Public".into(),
            kind: TypeExportKind::Type,
            start: 0,
            end: source.len() as u32,
            hoisted: false,
        });

        let plan =
            SetupTypeExportsPlan::new(&summary, Some(source), &ScriptBlockScopes::default(), false);
        let mut line = Cow::Borrowed(source);
        plan.strip_classic_modifiers(&mut line, 0);
        assert_eq!(line, " type Public = typeof value;");

        let mut fields = Vec::new();
        let mut setup = vize_carton::String::default();
        plan.emit_setup_artifacts(&mut setup, &mut fields);
        assert_eq!(fields, ["__vize_exported_type_Public"]);
        assert!(setup.contains("unknown as Public"));

        let mut module = vize_carton::String::default();
        plan.emit_module_exports(&mut module);
        assert!(module.contains("export type Public = Awaited<ReturnType<typeof __setup>>"));

        let setup_plan =
            SetupTypeExportsPlan::new(&summary, Some(source), &ScriptBlockScopes::default(), true);
        let mut setup_line = Cow::Borrowed(source);
        setup_plan.strip_classic_modifiers(&mut setup_line, 0);
        assert_eq!(setup_line, source);
    }

    #[test]
    fn merged_interface_exports_capture_one_artifact_per_name() {
        let script = "export interface Options { a: typeof value } export interface Options { b: typeof value }";
        let first_start = script.find("export").unwrap() as u32;
        let second_start = script.rfind("export").unwrap() as u32;
        let mut summary = Croquis::new();
        for start in [first_start, second_start] {
            summary.type_exports.push(TypeExport {
                name: "Options".into(),
                kind: TypeExportKind::Interface,
                start,
                end: script.len() as u32,
                hoisted: false,
            });
        }

        let plan =
            SetupTypeExportsPlan::new(&summary, Some(script), &ScriptBlockScopes::default(), true);

        // Only one artifact/alias is emitted for the merged name.
        let mut fields = Vec::new();
        let mut setup = vize_carton::String::default();
        plan.emit_setup_artifacts(&mut setup, &mut fields);
        assert_eq!(fields, ["__vize_exported_type_Options"]);
        assert_eq!(setup.matches("unknown as Options").count(), 1);

        let mut module = vize_carton::String::default();
        plan.emit_module_exports(&mut module);
        assert_eq!(module.matches("export type Options =").count(), 1);
    }
}
