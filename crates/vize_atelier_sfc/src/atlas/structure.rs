//! Structural SFC facts used only while Atlas builds a dependency plan.

use vize_atlas::PlanningContext;

pub(super) use crate::parse::SfcSourceStructure;

pub(super) fn source_structure(context: &PlanningContext<'_>) -> SfcSourceStructure {
    super::sfc_source_structure(context.source().text())
}

#[cfg(test)]
mod tests {
    #[test]
    fn sfc_registration_does_not_claim_raw_module_products() {
        let mut compilation = vize_atlas::Compilation::new();
        crate::register_atlas_providers(&mut compilation).unwrap();
        let source = compilation
            .add_source("state.ts", "export const count = 0")
            .unwrap();

        let module = compilation
            .plan_for::<vize_module::ModuleSyntaxProduct>(source)
            .unwrap_err();
        let flow = compilation
            .plan_for::<vize_flow::FlowProduct>(source)
            .unwrap_err();

        assert!(module.to_string().contains("no provider"), "{module}");
        assert!(flow.to_string().contains("no provider"), "{flow}");
    }

    #[test]
    fn sfc_frontend_accepts_virtual_query_suffixes() {
        let mut compilation = vize_atlas::Compilation::new();
        crate::register_atlas_providers(&mut compilation).unwrap();
        let source = compilation
            .add_source(
                "App.vue?vue&type=script",
                "<script>export default {}</script>",
            )
            .unwrap();

        assert!(
            compilation
                .query::<super::super::SfcDescriptorProduct>(source)
                .is_ok()
        );
    }

    #[test]
    fn dependency_planning_does_not_execute_the_descriptor_parser() {
        let before = crate::parse::parse_sfc_call_count();
        let mut compilation = vize_atlas::Compilation::new();
        crate::register_atlas_providers(&mut compilation).unwrap();
        vize_atelier_vapor::register_atlas_provider(&mut compilation).unwrap();
        let source = compilation
            .add_source(
                "Planning.vue",
                "<script setup vapor>const ready = true</script><template>{{ ready }}</template>",
            )
            .unwrap();
        compilation
            .plan_for::<super::super::SfcCompileProduct>(source)
            .unwrap();
        assert_eq!(crate::parse::parse_sfc_call_count(), before);
        assert_eq!(
            compilation
                .counters()
                .for_product::<super::super::SfcDescriptorProduct>()
                .executions(),
            0
        );
        assert_eq!(
            compilation
                .counters()
                .for_product::<super::super::SfcScriptSyntaxProduct>()
                .executions(),
            0
        );
    }
}
