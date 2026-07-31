//! Re-exports for names a plain (non-setup) `<script>` block declares.
//!
//! The plain-`<script>` body is moved inside `__setup()` so its diagnostics stay
//! anchored to user code, which puts every declaration out of reach of module
//! consumers. A bridge at module scope hands the bindings back:
//!
//! ```ignore
//! const __vize_plain_script_exports = __setup();
//! export const Mode = __vize_plain_script_exports.Mode;
//! ```
//!
//! That bridge carries only the *value* meaning. `enum` and `class` declare a
//! value **and** a type, so a consumer writing `const m: Mode = Mode.Split`
//! used to hit TS2749 ("refers to a value, but is being used as a type here").
//! Those kinds get a matching type alias so both meanings survive the move,
//! while `const`/`function` — value-only in the source — stay value-only here.

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Declaration, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, String as VizeString, append};

/// Which declaration spaces a plain-`<script>` export has to reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlainScriptExportKind {
    /// `const`/`let`/`function`: a value only. A type alias here would invent a
    /// type meaning the source never had.
    Value,
    /// `enum`/`const enum`: the type is the union of the member types, which
    /// `(typeof E)[keyof typeof E]` reconstructs from the enum object without
    /// widening string members to `string`.
    Enum,
    /// `class`: the type is the instance type. `InstanceType` accepts abstract
    /// constructors, so abstract and generic classes ride the same shape.
    Class,
}

impl PlainScriptExportKind {
    /// The type-side alias body, or `None` when the declaration is value-only.
    pub(super) fn type_alias_body(self, name: &str) -> Option<VizeString> {
        match self {
            Self::Value => None,
            Self::Enum => Some(vize_carton::cstr!("(typeof {name})[keyof typeof {name}]")),
            Self::Class => Some(vize_carton::cstr!("InstanceType<typeof {name}>")),
        }
    }
}

/// A name a plain `<script>` exports, plus the declaration spaces it occupies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlainScriptExport {
    pub(crate) name: CompactString,
    pub(crate) kind: PlainScriptExportKind,
    /// Whether the bridge declares the value side. Cleared for a name a hoisted
    /// namespace body captures: that name is declared earlier at module scope as
    /// an ambient alias instead (see `super::namespace_hoist`), and declaring it
    /// twice would be a duplicate identifier.
    pub(crate) bridged_value: bool,
}

pub(crate) fn collect_normal_script_named_value_exports(
    script: Option<&str>,
    has_script_setup: bool,
    has_plain_script_scope: bool,
) -> Vec<PlainScriptExport> {
    if has_script_setup || !has_plain_script_scope {
        return Vec::new();
    }
    script.map(collect_named_value_exports).unwrap_or_default()
}

pub(crate) fn push_setup_return_fields(
    exports: &[PlainScriptExport],
    fields: &mut Vec<CompactString>,
) {
    fields.extend(exports.iter().map(|export| export.name.clone()));
}

pub(crate) fn emit_setup_invocation_and_exports(
    ts: &mut VizeString,
    exports: &[PlainScriptExport],
) {
    if exports.iter().any(|export| export.bridged_value) {
        ts.push_str("const __vize_plain_script_exports = __setup();\n");
    } else {
        // Nothing left to read off the return object; binding it would only add
        // an unused module-scope const.
        ts.push_str("__setup();\n");
    }
    for export in exports {
        let name = &export.name;
        if export.bridged_value {
            append!(
                *ts,
                "export const {name} = __vize_plain_script_exports.{name};\n"
            );
        }
        // A value+type declaration loses its type meaning when the bridge only
        // re-exports the value. The alias restores it in the type space, which
        // is disjoint from the `const` above, so both names can coexist.
        if let Some(body) = export.kind.type_alias_body(name) {
            append!(*ts, "export type {name} = {body};\n");
        }
    }
    ts.push('\n');
}

fn collect_named_value_exports(script: &str) -> Vec<PlainScriptExport> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts().with_module(true)).parse();
    let parsed = if parsed.panicked || !parsed.diagnostics.is_empty() {
        Parser::new(&allocator, script, SourceType::tsx().with_module(true)).parse()
    } else {
        parsed
    };
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Vec::new();
    }

    let mut seen = FxHashSet::default();
    let mut exports = Vec::new();
    for statement in &parsed.program.body {
        collect_statement_exports(statement, &mut seen, &mut exports);
    }
    exports
}

fn collect_statement_exports(
    statement: &Statement<'_>,
    seen: &mut FxHashSet<CompactString>,
    exports: &mut Vec<PlainScriptExport>,
) {
    let Statement::ExportNamedDeclaration(export) = statement else {
        return;
    };
    if export.source.is_some() || export.export_kind.is_type() {
        return;
    }
    let Some(declaration) = export.declaration.as_ref() else {
        return;
    };
    collect_declaration_exports(declaration, seen, exports);
}

pub(super) fn collect_declaration_exports(
    declaration: &Declaration<'_>,
    seen: &mut FxHashSet<CompactString>,
    exports: &mut Vec<PlainScriptExport>,
) {
    match declaration {
        Declaration::VariableDeclaration(variable) => {
            for declarator in &variable.declarations {
                collect_binding_names(&declarator.id, seen, exports);
            }
        }
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                push_export(
                    id.name.as_str(),
                    PlainScriptExportKind::Value,
                    seen,
                    exports,
                );
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                push_export(
                    id.name.as_str(),
                    PlainScriptExportKind::Class,
                    seen,
                    exports,
                );
            }
        }
        Declaration::TSEnumDeclaration(enumeration) => {
            // `const enum` shares the shape: its members are still reachable
            // through the enum object in a type query.
            push_export(
                enumeration.id.name.as_str(),
                PlainScriptExportKind::Enum,
                seen,
                exports,
            );
        }
        _ => {}
    }
}

fn collect_binding_names(
    pattern: &BindingPattern<'_>,
    seen: &mut FxHashSet<CompactString>,
    exports: &mut Vec<PlainScriptExport>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            push_export(
                id.name.as_str(),
                PlainScriptExportKind::Value,
                seen,
                exports,
            );
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_binding_names(&property.value, seen, exports);
            }
            if let Some(rest) = &object.rest {
                collect_binding_names(&rest.argument, seen, exports);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_binding_names(element, seen, exports);
            }
            if let Some(rest) = &array.rest {
                collect_binding_names(&rest.argument, seen, exports);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_binding_names(&assignment.left, seen, exports);
        }
    }
}

fn push_export(
    name: &str,
    kind: PlainScriptExportKind,
    seen: &mut FxHashSet<CompactString>,
    exports: &mut Vec<PlainScriptExport>,
) {
    let name = CompactString::new(name);
    // Declaration merging (two `export enum Mode` blocks) surfaces one name
    // twice; a second `export const`/`export type` pair would be a duplicate
    // binding, so only the first occurrence is bridged.
    if seen.insert(name.clone()) {
        exports.push(PlainScriptExport {
            name,
            kind,
            bridged_value: true,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CompactString, PlainScriptExport, PlainScriptExportKind, collect_named_value_exports,
        emit_setup_invocation_and_exports,
    };

    fn export(name: &str, kind: PlainScriptExportKind) -> PlainScriptExport {
        PlainScriptExport {
            name: CompactString::new(name),
            kind,
            bridged_value: true,
        }
    }

    #[test]
    fn collect_named_value_exports_includes_ts_enums() {
        let exports = collect_named_value_exports(
            "export enum DiffDisplayMode { Hidden = 'hidden' }\nexport type Props = {}\n",
        );

        assert_eq!(
            exports,
            vec![export("DiffDisplayMode", PlainScriptExportKind::Enum)]
        );
    }

    #[test]
    fn declaration_kinds_are_classified_by_declaration_space() {
        let exports = collect_named_value_exports(
            "export const plain = 1;\nexport function helper() {}\nexport class Widget {}\nexport enum Mode { A = 'a' }\nexport const enum ConstMode { B = 'b' }\n",
        );

        assert_eq!(
            exports,
            vec![
                export("plain", PlainScriptExportKind::Value),
                export("helper", PlainScriptExportKind::Value),
                export("Widget", PlainScriptExportKind::Class),
                export("Mode", PlainScriptExportKind::Enum),
                export("ConstMode", PlainScriptExportKind::Enum),
            ]
        );
    }

    #[test]
    fn value_and_type_declarations_are_bridged_in_both_declaration_spaces() {
        let mut ts = vize_carton::String::default();
        emit_setup_invocation_and_exports(
            &mut ts,
            &[
                export("Mode", PlainScriptExportKind::Enum),
                export("Widget", PlainScriptExportKind::Class),
            ],
        );

        assert!(ts.contains("export const Mode = __vize_plain_script_exports.Mode;\n"));
        assert!(ts.contains("export type Mode = (typeof Mode)[keyof typeof Mode];\n"));
        assert!(ts.contains("export const Widget = __vize_plain_script_exports.Widget;\n"));
        assert!(ts.contains("export type Widget = InstanceType<typeof Widget>;\n"));
    }

    #[test]
    fn value_only_declarations_never_gain_a_type_export() {
        let mut ts = vize_carton::String::default();
        emit_setup_invocation_and_exports(
            &mut ts,
            &[
                export("plain", PlainScriptExportKind::Value),
                export("helper", PlainScriptExportKind::Value),
            ],
        );

        assert!(ts.contains("export const plain = __vize_plain_script_exports.plain;\n"));
        assert!(ts.contains("export const helper = __vize_plain_script_exports.helper;\n"));
        assert!(
            !ts.contains("export type "),
            "value-only exports must not invent a type meaning:\n{ts}"
        );
    }

    #[test]
    fn merged_enum_declarations_are_bridged_once() {
        let exports = collect_named_value_exports(
            "export enum Mode { A = 'a' }\nexport enum Mode { B = 'b' }\n",
        );
        assert_eq!(exports, vec![export("Mode", PlainScriptExportKind::Enum)]);

        let mut ts = vize_carton::String::default();
        emit_setup_invocation_and_exports(&mut ts, &exports);
        assert_eq!(ts.matches("export const Mode =").count(), 1);
        assert_eq!(ts.matches("export type Mode =").count(), 1);
    }

    #[test]
    fn no_exports_keeps_the_bare_setup_invocation() {
        let mut ts = vize_carton::String::default();
        emit_setup_invocation_and_exports(&mut ts, &[]);
        assert_eq!(ts.as_str(), "__setup();\n\n");
    }
}
