//! Runtime props the analyzer cannot list, typed from the checker's view.
//!
//! `defineProps(sharedProps)` with an imported object names its props only
//! through the object's type. The batch resolver reads the props from the
//! imported module's source when it can, but a package that ships only
//! declarations (`export declare const sharedProps: { ... }`, what every
//! published Vue component library emits) has no initializer to read, and an
//! unresolvable import has no source at all. The checker still knows
//! `typeof sharedProps`, so the component contract and the template are typed
//! from that type instead: Vue derives both from the same object.

use vize_carton::{String, cstr};
use vize_croquis::{BindingType, Croquis, ScopeKind};

/// The imported binding `defineProps(binding)` reads its runtime props from,
/// when the analyzer listed none of them. Only an import qualifies: it is
/// visible both at module scope (the exported `Props`) and in the template.
pub(super) fn opaque_runtime_props_binding(summary: &Croquis) -> Option<&str> {
    if !summary.macros.props().is_empty() {
        return None;
    }
    let call = summary
        .macros
        .define_props()
        .filter(|call| call.type_args.is_none())?;
    let binding = runtime_arg_identifier(call.runtime_args.as_deref()?)?;
    is_value_import(summary, binding).then_some(binding)
}

/// The parent-facing contract of a runtime props object: a prop is required
/// when it declares `required: true`, unless a `default` relaxes it (a plain
/// `Boolean` prop keeps `required: true` over its default, as Vue does).
pub(super) fn opaque_runtime_public_props_type(binding: &str) -> String {
    let options = cstr!("(typeof {binding})[__K]");
    let required = cstr!(
        "{options} extends {{ required: true }} ? {options} extends {{ default: any }} ? {options} extends {{ type: BooleanConstructor }} ? __K : never : __K : never"
    );
    let shape = cstr!("__RuntimePropShape<typeof {binding}>[__K]");
    cstr!(
        "{{ [__K in keyof typeof {binding} as {required}]: {shape} }} & {{ [__K in keyof typeof {binding} as [{required}] extends [never] ? __K : never]?: {shape} }}"
    )
}

/// What a template read of the object's props resolves to: Vue's resolved
/// props, where a required, defaulted or Boolean prop is never `undefined`.
pub(super) fn opaque_runtime_resolved_props_type(binding: &str) -> String {
    cstr!("__RuntimePropShape<typeof {binding}>")
}

fn is_value_import(summary: &Croquis, name: &str) -> bool {
    summary
        .scopes
        .iter()
        .filter(|scope| scope.kind == ScopeKind::ExternalModule)
        .flat_map(|scope| scope.bindings())
        .any(|(binding, data)| binding == name && data.binding_type != BindingType::ExternalModule)
}

fn runtime_arg_identifier(args: &str) -> Option<&str> {
    let mut value = args.trim();
    while let Some(inner) = value
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
    {
        value = inner.trim();
    }
    let mut chars = value.chars();
    let first = chars.next()?;
    ((first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()))
    .then_some(value)
}

#[cfg(test)]
mod tests {
    use super::runtime_arg_identifier;
    use crate::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
    use vize_croquis::{Analyzer, AnalyzerOptions};

    fn generate(script: &str, template: &str) -> vize_carton::String {
        let allocator = vize_carton::Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        generate_virtual_ts_with_offsets(
            &summary,
            Some(script),
            Some(&root),
            0,
            100,
            &VirtualTsOptions::default(),
        )
        .code
    }

    fn lines_with<'a>(code: &'a str, needle: &str) -> Vec<&'a str> {
        code.lines().filter(|line| line.contains(needle)).collect()
    }

    #[test]
    fn imported_props_object_types_contract_and_template_from_its_type() {
        let code = generate(
            "import { sharedProps } from \"props-pkg\";\ndefineProps(sharedProps);\n",
            "<button @click=\"onRemove\">{{ label }}</button>",
        );
        let shape = "__RuntimePropShape<typeof sharedProps>[__K]";
        let required = "(typeof sharedProps)[__K] extends { required: true } ? (typeof sharedProps)[__K] extends { default: any } ? (typeof sharedProps)[__K] extends { type: BooleanConstructor } ? __K : never : __K : never";
        assert_eq!(
            lines_with(&code, "export type Props"),
            [vize_carton::cstr!(
                "export type Props = {{ [__K in keyof typeof sharedProps as {required}]: {shape} }} & {{ [__K in keyof typeof sharedProps as [{required}] extends [never] ? __K : never]?: {shape} }};"
            )
            .as_str()],
            "{code}"
        );
        assert_eq!(
            lines_with(&code, "const props:"),
            ["  const props: __RuntimePropShape<typeof sharedProps> & Props = {} as __RuntimePropShape<typeof sharedProps> & Props;"],
            "{code}"
        );
        assert_eq!(
            lines_with(&code, "satisfies keyof"),
            [
                "  const label = __vize_keyed_template_context[(\"label\" satisfies keyof typeof __vize_keyed_template_context)];",
                "  const onRemove = __vize_keyed_template_context[(\"onRemove\" satisfies keyof typeof __vize_keyed_template_context)];",
            ],
            "{code}"
        );
    }

    #[test]
    fn listed_or_local_props_keep_their_own_declarations() {
        for script in [
            "defineProps({ label: String });\n",
            "const sharedProps = { label: String };\ndefineProps(sharedProps);\n",
        ] {
            let code = generate(script, "<p>{{ label }}</p>");
            assert_eq!(
                lines_with(&code, "__vize_keyed_template_context"),
                Vec::<&str>::new(),
                "{code}"
            );
        }
    }

    #[test]
    fn reads_only_a_bare_identifier_argument() {
        assert_eq!(
            runtime_arg_identifier(" (sharedProps) "),
            Some("sharedProps")
        );
        assert_eq!(runtime_arg_identifier("$props_1"), Some("$props_1"));
        assert_eq!(runtime_arg_identifier("{ ...sharedProps }"), None);
        assert_eq!(runtime_arg_identifier("props.shared"), None);
        assert_eq!(runtime_arg_identifier("1props"), None);
        assert_eq!(runtime_arg_identifier(""), None);
    }
}
