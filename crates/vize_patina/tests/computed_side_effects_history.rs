//! Issue #8826: the original pinned n8n Composition getter must diagnose writes.
use vize_patina::{LintDiagnostic, LintPreset, LintResult, Linter};

const RULE: &str = "script/no-side-effects-in-computed-properties";

fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]))
}

#[test]
fn original_n8n_getter_preserves_the_complete_authored_mutation_report() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/linter/computed-side-effects-8826/CredentialResolverEditModal.vue.fixture"
    );
    let filename = "CredentialResolverEditModal.vue";
    let start = source
        .find("hasEverMadeNonNameChange.value = true")
        .unwrap() as u32;
    let end = start + "hasEverMadeNonNameChange.value = true".len() as u32;
    let diagnostic = LintDiagnostic::error(RULE,
        "Unexpected side effect in a computed property: an assignment to external state.", start, end)
        .with_label("side effect inside computed getter", start, end)
        .with_help("A computed getter must be pure: derive and return a value without mutating component state. Move the mutation to a method or a watch handler.");
    let expected = LintResult {
        filename: filename.into(),
        error_count: 1,
        warning_count: 0,
        diagnostics: vec![diagnostic],
    };
    let actual = linter().lint_sfc(source, filename);
    assert_eq!(format!("{actual:#?}"), format!("{expected:#?}"));
}

#[test]
fn imported_computed_identity_and_external_write_controls() {
    for (source, errors) in [
        (
            r#"import { computed } from 'vue'; const state={value:0}; computed(()=>{state.value=1;return 1})"#,
            1,
        ),
        (
            r#"import * as Vue from 'vue'; const state={value:0}; Vue['\u0063omputed'](()=>{state.value=1;return 1})"#,
            1,
        ),
        (
            r#"import {computed} from '@vue/composition-api';const state={value:0};computed(()=>{state.value=1;return 1})"#,
            1,
        ),
        (
            r#"export default{computed:{value(){this['count']=1;return this['count']}}}"#,
            1,
        ),
        (
            "import { computed, ref } from 'vue'; const state=ref(0); computed(()=>{state.value=1;return state.value})",
            1,
        ),
        (
            "import { computed as derive } from 'vue'; const state={value:0}; derive(()=>{state.value++;return state.value})",
            1,
        ),
        (
            "import * as Vue from 'vue'; const state={value:[]}; Vue.computed(()=>state.value.reverse())",
            1,
        ),
        (
            "import { computed } from 'vue'; const derive=computed; const state={value:0}; derive(()=>{state.value=1;return 1})",
            1,
        ),
        (
            "import * as Vue from 'vue'; const derive=Vue['computed']; const state={value:0}; derive(()=>state['value']=1)",
            1,
        ),
        (
            "import { computed } from 'vue'; const state={value:0}; computed({get(){state.value=1;return state.value},set(v){state.value=v}})",
            1,
        ),
        (
            "import { computed } from 'vue'; computed(()=>{const state={value:0};state.value=1;return state.value})",
            0,
        ),
        (
            "import { computed } from 'vue'; const state={value:0}; computed(()=>state.value)",
            0,
        ),
        (
            "import { computed } from 'vue'; const state={value:[]}; computed(()=>[...state.value].reverse())",
            0,
        ),
        (
            "import { computed } from 'vue'; const state={value:[]}; computed(()=>state.value.slice().reverse())",
            0,
        ),
        (
            "import { computed } from 'vue'; const state={value:0}; computed(()=>()=>{state.value=1})",
            0,
        ),
        (
            "import { computed } from 'vue'; const state={value:0}; computed(()=>{function deferred(){state.value=1};return deferred})",
            0,
        ),
        (
            "import { computed } from 'elsewhere'; const state={value:0}; computed(()=>state.value=1)",
            0,
        ),
        (
            "import { computed } from 'vue'; const state={value:0}; function use(computed){computed(()=>state.value=1)}",
            0,
        ),
        (
            "import * as Vue from 'vue'; const state={value:0}; function use(Vue){Vue.computed(()=>state.value=1)}",
            0,
        ),
        (
            "import { computed } from 'vue'; const state={value:0}; computed((state)=>{state.value=1;return state.value})",
            0,
        ),
        (
            "import { computed } from 'vue'; computed(()=>{let total=0;total++;return total})",
            0,
        ),
    ] {
        let actual = linter().lint_script(source, "control.ts");
        assert_eq!(actual.error_count, errors, "{source}: {actual:#?}");
        assert_eq!(actual.warning_count, 0, "{source}: {actual:#?}");
        assert_eq!(actual.diagnostics.len(), errors, "{source}: {actual:#?}");
    }
}
