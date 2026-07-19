use std::borrow::Cow;

use vize_carton::{String, append, cstr};
use vize_croquis::Croquis;

fn skip_trivia(source: &str, mut at: usize) -> usize {
    let bytes = source.as_bytes();
    loop {
        while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        if source.get(at..).is_some_and(|rest| rest.starts_with("//")) {
            at = source[at..]
                .find('\n')
                .map_or(source.len(), |end| at + end + 1);
        } else if source.get(at..).is_some_and(|rest| rest.starts_with("/*")) {
            at = source[at + 2..]
                .find("*/")
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
    export_starts: Vec<u32>,
    exports: Vec<SetupTypeExport>,
}

impl SetupTypeExportsPlan {
    pub(super) fn new(summary: &Croquis, script: Option<&str>) -> Self {
        let mut export_starts = Vec::new();
        let mut exports = Vec::new();

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

                let name: String = declaration.name.as_str().into();
                if exported_type_is_generic(source, &name) {
                    continue;
                }
                export_starts.push(declaration.start);
                exports.push(SetupTypeExport {
                    artifact: cstr!("__vize_exported_type_{name}"),
                    name,
                });
            }
        }

        export_starts.sort_unstable();
        Self {
            export_starts,
            exports,
        }
    }

    pub(super) fn strip_modifiers(&self, line: &mut Cow<'_, str>, line_start: u32) {
        let line_end = line_start.saturating_add(line.len() as u32);
        let first = self
            .export_starts
            .partition_point(|start| *start < line_start);
        let last = self
            .export_starts
            .partition_point(|start| *start < line_end);
        if first == last {
            return;
        }

        let original = line.as_ref();
        let mut output = String::with_capacity(original.len());
        let mut copied_until = 0usize;
        for start in &self.export_starts[first..last] {
            let column = (*start - line_start) as usize;
            if original.get(column..column + "export".len()) == Some("export") {
                output.push_str(&original[copied_until..column]);
                copied_until = column + "export".len();
            }
        }
        output.push_str(&original[copied_until..]);
        *line = Cow::Owned(output.into());
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
    use super::{SetupTypeExportsPlan, exported_type_is_generic};
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

        let plan = SetupTypeExportsPlan::new(&summary, Some(script));
        let mut line = std::borrow::Cow::Borrowed("export type Public = typeof value;");
        plan.strip_modifiers(&mut line, public_start);
        assert_eq!(line, " type Public = typeof value;");

        let mut fields = Vec::new();
        let mut setup = vize_carton::String::default();
        plan.emit_setup_artifacts(&mut setup, &mut fields);
        assert!(setup.contains("unknown as Public"));
        assert_eq!(fields, ["__vize_exported_type_Public"]);
    }

    #[test]
    fn every_export_modifier_on_a_shared_line_is_removed_by_exact_span() {
        let script = "const value = 1; export type Public = typeof value; export interface Shape { value: typeof value }";
        let public_start = script.find("export").unwrap() as u32;
        let shape_start = script.rfind("export").unwrap() as u32;
        let mut summary = Croquis::new();
        for (name, kind, start) in [
            ("Public", TypeExportKind::Type, public_start),
            ("Shape", TypeExportKind::Interface, shape_start),
        ] {
            summary.type_exports.push(TypeExport {
                name: name.into(),
                kind,
                start,
                end: script.len() as u32,
                hoisted: false,
            });
        }

        let plan = SetupTypeExportsPlan::new(&summary, Some(script));
        let mut line = std::borrow::Cow::Borrowed(script);
        plan.strip_modifiers(&mut line, 0);
        assert_eq!(
            line,
            "const value = 1;  type Public = typeof value;  interface Shape { value: typeof value }"
        );
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
        let plan = SetupTypeExportsPlan::new(&summary, Some(source));
        let mut line = std::borrow::Cow::Borrowed(source);
        plan.strip_modifiers(&mut line, 0);
        assert_eq!(line, source);

        let mut fields = Vec::new();
        let mut setup = vize_carton::String::default();
        plan.emit_setup_artifacts(&mut setup, &mut fields);
        assert!(fields.is_empty());
        assert!(setup.is_empty());
    }
}
