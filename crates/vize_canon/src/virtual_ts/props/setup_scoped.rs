use oxc_allocator::Allocator;
use oxc_ast::ast::{TSTypeName, TSTypeReference};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, String, append, cstr};
use vize_croquis::Croquis;

use super::{append_model_props_type_literal, extract_generic_names};

/// Props type emission mode for `defineProps<T>()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropsTypeEmission {
    /// Emit `export type Props = ...` in module scope before `__setup()`.
    Module,
    /// Keep the concrete props type inside `__setup()` and export it through
    /// `ReturnType<typeof __setup>`. This is needed when `T` references a
    /// setup-scope value via `typeof`.
    DeferredToSetup,
}

/// Emit the setup-local props type artifact used when the `defineProps<T>()`
/// type argument can only resolve inside `__setup()`.
pub(crate) fn generate_setup_scoped_props_artifact(ts: &mut String, summary: &Croquis) {
    let Some(type_args) = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref())
    else {
        return;
    };
    let inner_type = type_args
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(type_args.as_str());
    let models = summary.macros.models();

    ts.push_str("\n  // Setup-scoped props type artifact\n");
    if models.is_empty() {
        append!(*ts, "  type __VizeSetupProps = {inner_type};\n");
    } else {
        append!(*ts, "  type __VizeSetupProps = {inner_type} & ");
        append_model_props_type_literal(ts, summary, models);
        ts.push_str(";\n");
    }
    ts.push_str("  const __vize_setup_props = undefined as unknown as __VizeSetupProps;\n");
}

pub(super) fn props_type_ref(
    generic_param: Option<&str>,
    props_type_ref_override: Option<&str>,
) -> String {
    props_type_ref_override
        .map(String::from)
        .unwrap_or_else(|| {
            generic_param
                .map(|g| {
                    let names = extract_generic_names(g);
                    cstr!("Props<{names}>")
                })
                .unwrap_or_else(|| "Props".into())
        })
}

pub(super) fn unused_generic_comment(
    generic_param: Option<&str>,
    props_source: &str,
) -> &'static str {
    let Some(generic) = generic_param else {
        return "";
    };
    let names = extract_generic_names(generic);
    let type_references = collect_type_reference_names(props_source);
    let uses_every_generic = type_references.is_some_and(|type_references| {
        names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .all(|name| {
                type_references
                    .iter()
                    .any(|type_reference| type_reference.as_str() == name)
            })
    });
    if uses_every_generic {
        ""
    } else {
        "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
    }
}

#[derive(Default)]
struct TypeReferenceNames {
    names: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for TypeReferenceNames {
    fn visit_ts_type_reference(&mut self, ty: &TSTypeReference<'a>) {
        record_type_name_root(&ty.type_name, &mut self.names);
        walk::walk_ts_type_reference(self, ty);
    }
}

fn collect_type_reference_names(props_source: &str) -> Option<FxHashSet<CompactString>> {
    let source = cstr!("type __VizeProps = {props_source};");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source.as_str(), SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }

    let mut references = TypeReferenceNames::default();
    references.visit_program(&parsed.program);
    Some(references.names)
}

fn record_type_name_root(name: &TSTypeName<'_>, names: &mut FxHashSet<CompactString>) {
    match name {
        TSTypeName::IdentifierReference(identifier) => {
            names.insert(CompactString::new(identifier.name.as_str()));
        }
        TSTypeName::QualifiedName(qualified) => record_type_name_root(&qualified.left, names),
        TSTypeName::ThisExpression(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::unused_generic_comment;

    #[test]
    fn suppresses_only_props_aliases_that_omit_sfc_generics() {
        assert_eq!(
            unused_generic_comment(Some("T"), "ImportedProps"),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
        assert_eq!(unused_generic_comment(Some("T"), "LocalProps<T>"), "");
        assert_eq!(
            unused_generic_comment(Some("T, U"), "LocalProps<T>"),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
        assert_eq!(unused_generic_comment(Some("T, U"), "LocalProps<T, U>"), "");
        assert_eq!(
            unused_generic_comment(Some("T"), r#"{ T: string; kind: "T" }"#),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
        assert_eq!(unused_generic_comment(Some("T"), "{ value: T }"), "");
        assert_eq!(
            unused_generic_comment(Some("T"), "SomeTTProps"),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
    }
}
