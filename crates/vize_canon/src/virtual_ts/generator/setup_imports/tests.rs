use super::SetupImportPlan;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn script_slot_consumers_require_a_runtime_vue_accessor_reference() {
    for (script, expected) in [
        (
            "import { useSlots } from 'vue'; const slots = useSlots()",
            true,
        ),
        (
            "import { useSlots as slotsOf } from 'vue'; const slots = slotsOf()",
            true,
        ),
        (
            "import * as Vue from 'vue'; const slots = Vue.useSlots()",
            true,
        ),
        (
            "import * as Vue from 'vue'; const slots = Vue['useSlots']()",
            true,
        ),
        (
            "import * as Vue from 'vue'; Vue.useCssModule(); Vue.useSlots()",
            true,
        ),
        ("import { useSlots } from 'vue'", false),
        (
            "import { useSlots } from 'vue'; type Slots = ReturnType<typeof useSlots>",
            false,
        ),
        (
            "import { useSlots } from 'vue'; function local(useSlots: () => {}) { return useSlots() }",
            false,
        ),
        ("import { useSlots } from './other'; useSlots()", false),
        (
            "import * as Vue from 'vue'; Vue.ref(0); // useSlots()",
            false,
        ),
        ("const useSlots = () => ({}); useSlots()", false),
        (
            "import { useSlots } from 'vue'; useSlots(); defineSlots<{ named(): unknown }>()",
            false,
        ),
    ] {
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();
        let plan = SetupImportPlan::new(Some(script), &summary, None, Default::default());
        assert_eq!(plan.has_untyped_script_slots(), expected, "{script}");
    }
}
