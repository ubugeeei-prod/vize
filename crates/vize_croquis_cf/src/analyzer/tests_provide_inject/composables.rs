use super::*;
use crate::diagnostics::CrossFileDiagnosticKind;

#[test]
fn called_imported_composable_projects_only_its_inject_into_the_component() {
    let mut analyzer = CrossFileAnalyzer::new(
        CrossFileOptions::minimal()
            .with_provide_inject(true)
            .with_reactivity_tracking(true),
    );
    let provider = analyzer.add_file_with_analysis(
        Path::new("src/components/MyForm.vue"),
        "",
        script_analysis(
            r#"import { provide } from 'vue'
import { FormKey } from '../composables/form'
provide(FormKey, { register() {}, unregister() {} })"#,
            &["MyInput"],
        ),
    );
    let consumer_script = r#"import { inject } from 'vue'
import { useFormField as bindField } from '../composables/form'
bindField(() => true)
function useLocalUncalled() {
  inject(LocalKey)
}"#;
    let consumer = analyzer.add_file_with_analysis(
        Path::new("src/components/MyInput.vue"),
        "",
        script_analysis(consumer_script, &[]),
    );
    analyzer.add_file(
        Path::new("src/composables/form.ts"),
        r#"import { inject } from 'vue'
export const FormKey = Symbol('form')
export const UnusedKey = Symbol('unused')
export function useFormField() {
  const form = inject(FormKey, null)
  return form
}
export function useUnusedField() {
  return inject(UnusedKey)
}"#,
    );
    analyzer.add_file(
        Path::new("src/other/form.ts"),
        r#"import { inject } from 'vue'
export const WrongKey = Symbol('wrong')
export function useFormField() {
  return inject(WrongKey)
}"#,
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let matched = result
        .provide_inject_matches
        .iter()
        .filter(|matched| matched.key == "FormKey")
        .collect::<Vec<_>>();

    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].provider, provider);
    assert_eq!(matched[0].consumer, consumer);
    assert!(result.diagnostics.iter().all(|diagnostic| {
        !matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::UnusedProvide { key }
                | CrossFileDiagnosticKind::UnmatchedInject { key }
                if key == "FormKey"
                    || key == "UnusedKey"
                    || key == "WrongKey"
                    || key == "LocalKey"
        )
    }));
    assert!(result.diagnostics.iter().all(|diagnostic| {
        !matches!(
            diagnostic.kind,
            CrossFileDiagnosticKind::DestructuringBreaksReactivity { .. }
        )
    }));
}

#[test]
fn required_composable_inject_reports_at_the_setup_call_site() {
    let mut analyzer = CrossFileAnalyzer::new(
        CrossFileOptions::minimal()
            .with_provide_inject(true)
            .with_reactivity_tracking(true),
    );
    let consumer_script = r#"import { useRequired as bindRequired } from '../composables/required'
bindRequired()"#;
    let call_start = consumer_script.find("bindRequired()").unwrap() as u32;
    let consumer = analyzer.add_file_with_analysis(
        Path::new("src/components/MyInput.vue"),
        "",
        script_analysis(consumer_script, &[]),
    );
    analyzer.add_file(
        Path::new("src/composables/required.ts"),
        r#"import { inject } from 'vue'
export const RequiredKey = Symbol('required')
export function useRequired() {
  return inject(RequiredKey)
}"#,
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::UnmatchedInject { key } if key == "RequiredKey"
            )
        })
        .expect("the required composable inject should be checked");

    assert_eq!(diagnostic.primary_file, consumer);
    assert_eq!(diagnostic.primary_offset, call_start);
    assert_eq!(
        diagnostic.primary_end_offset,
        call_start + "bindRequired()".len() as u32
    );
}
