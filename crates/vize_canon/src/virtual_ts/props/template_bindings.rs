use vize_carton::{FxHashSet, String};
use vize_croquis::macros::PropDefinition;
use vize_croquis::macros::PropsDestructuredBindings;
use vize_croquis::{BindingType, Croquis};

use super::mappings::PropBindingMappings;

#[inline]
pub(super) fn should_skip_template_prop_binding(summary: &Croquis, prop_name: &str) -> bool {
    is_define_props_destructure_local(summary.macros.props_destructure(), prop_name)
        || super::super::script_facts::binding_type(summary, prop_name)
            .is_some_and(|binding_type| !matches!(binding_type, BindingType::Props))
}

fn is_define_props_destructure_local(
    destructure: Option<&PropsDestructuredBindings>,
    prop_name: &str,
) -> bool {
    destructure.is_some_and(|destructure| {
        destructure
            .get(prop_name)
            .is_some_and(|binding| binding.local.as_str() == prop_name)
            || destructure.rest_id.as_deref() == Some(prop_name)
            || destructure
                .bindings
                .values()
                .any(|binding| binding.local.as_str() == prop_name)
    })
}

pub(super) fn emit_macro_template_prop_bindings(
    ts: &mut String,
    binding_mappings: &mut PropBindingMappings<'_>,
    summary: &Croquis,
    props_type_ref: &str,
    props: &[PropDefinition],
    defaulted_prop_names: &FxHashSet<String>,
    emitted_names: &mut FxHashSet<String>,
) {
    for prop in props {
        if emitted_names.contains(prop.name.as_str())
            || should_skip_template_prop_binding(summary, prop.name.as_str())
        {
            continue;
        }
        binding_mappings.emit(
            ts,
            props_type_ref,
            prop.name.as_str(),
            prop.default_value.is_some() || defaulted_prop_names.contains(&prop.name),
        );
        emitted_names.insert(prop.name.as_str().into());
    }
}

#[cfg(test)]
mod tests {
    use vize_croquis::{Analyzer, AnalyzerOptions};

    use super::should_skip_template_prop_binding;

    #[test]
    fn renamed_destructured_prop_keeps_its_authored_template_name() {
        let script = "const { expanded: expandedProp } = defineProps<{ expanded: boolean }>();";
        let allocator = vize_carton::Allocator::new();
        let (root, errors) =
            vize_armature::parse(&allocator, "<p v-if=\"expanded\">{{ expanded }}</p>");
        assert!(errors.is_empty());

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        assert!(!should_skip_template_prop_binding(&summary, "expanded"));
        assert!(should_skip_template_prop_binding(&summary, "expandedProp"));
        let output = crate::virtual_ts::generate_virtual_ts(&summary, Some(script), Some(&root), 0);
        assert!(
            output.code.contains("const expanded = props[\"expanded\"]"),
            "{}",
            output.code
        );
    }
}
