//! Tests for the lint command.

use super::{
    build_cross_file_lint_output, collect::collect_lint_files, should_render_lint_details,
};
use std::{fs, path::Path};
use vize_patina::{LintPreset, LintResult, Linter, OutputFormat};

fn result_for_file<'a>(results: &'a [LintResult], file_name: &str) -> &'a LintResult {
    results
        .iter()
        .find(|result| {
            Path::new(result.filename.as_str())
                .file_name()
                .and_then(|name| name.to_str())
                == Some(file_name)
        })
        .expect("result should exist")
}

fn diagnostic_summary(result: &LintResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{:?}|{}|{}..{}",
                diagnostic.severity, diagnostic.message, diagnostic.start, diagnostic.end
            )
        })
        .collect()
}

fn all_diagnostic_summary(results: &[LintResult]) -> Vec<String> {
    results.iter().flat_map(diagnostic_summary).collect()
}

#[test]
fn quiet_text_output_skips_detailed_diagnostics() {
    assert!(!should_render_lint_details(OutputFormat::Text, true));
}

#[test]
fn json_output_remains_machine_readable_in_quiet_mode() {
    assert!(should_render_lint_details(OutputFormat::Json, true));
}

#[test]
fn report_formats_render_in_quiet_mode() {
    assert!(should_render_lint_details(OutputFormat::Ansi, true));
    assert!(should_render_lint_details(OutputFormat::Plain, true));
    assert!(should_render_lint_details(OutputFormat::Markdown, true));
    assert!(should_render_lint_details(OutputFormat::Html, true));
    assert!(should_render_lint_details(OutputFormat::Agent, true));
}

#[test]
fn lint_collection_includes_jsx_and_tsx() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("App.vue"), "").unwrap();
    fs::write(src.join("Panel.jsx"), "").unwrap();
    fs::write(src.join("Widget.tsx"), "").unwrap();
    fs::write(src.join("skip.ts"), "").unwrap();

    let files = collect_lint_files(&[src.display().to_string().into()]);

    assert_eq!(
        files,
        vec![
            src.join("App.vue"),
            src.join("Panel.jsx"),
            src.join("Widget.tsx"),
        ]
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn strict_reactivity_can_be_enabled_without_opinionated_preset() {
    let linter = Linter::with_preset(LintPreset::HappyPath).with_rule(Box::new(
        vize_patina::rules::type_aware::NoReactivityLoss::new(),
    ));

    assert!(linter.registry().has_rule("type/no-reactivity-loss"));
}

#[test]
fn cross_file_opt_in_reports_reactivity_and_tree() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("App.vue");
    let middle = dir.path().join("Middle.vue");
    let child = dir.path().join("Child.vue");

    fs::write(
        &app,
        r#"<script setup lang="ts">
import { provide, reactive } from 'vue'
import Middle from './Middle.vue'

const state = reactive({ count: 0 })
provide('state', state)
</script>

<template>
  <Middle />
</template>
"#,
    )
    .unwrap();
    fs::write(
        &middle,
        r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child />
</template>
"#,
    )
    .unwrap();
    fs::write(
        &child,
        r#"<script setup lang="ts">
import { inject } from 'vue'

const { count } = inject('state') as { count: number }
</script>
"#,
    )
    .unwrap();

    let files = [&app, &middle, &child]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, true);

    let child_result = result_for_file(&output.results, "Child.vue");
    insta::assert_debug_snapshot!(diagnostic_summary(child_result));

    let tree = output
        .provide_inject_tree
        .as_deref()
        .expect("tree should be rendered");
    insta::assert_snapshot!(tree);
}

#[test]
fn cross_file_opt_in_resolves_provide_inject_through_default_slot() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("App.vue");
    let provider = dir.path().join("Provider.vue");
    let consumer = dir.path().join("Consumer.vue");

    fs::write(
        &provider,
        r#"<script setup lang="ts">
import { provide, ref } from 'vue'

const count = ref(0)
provide('count', count)
</script>

<template>
  <slot />
</template>
"#,
    )
    .unwrap();
    fs::write(
        &consumer,
        r#"<script setup lang="ts">
import { inject } from 'vue'

const count = inject('count')
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
    )
    .unwrap();
    fs::write(
        &app,
        r#"<script setup lang="ts">
import Consumer from './Consumer.vue'
import Provider from './Provider.vue'
</script>

<template>
  <Provider>
<Consumer />
  </Provider>
</template>
"#,
    )
    .unwrap();

    let files = [&app, &provider, &consumer]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, true);

    insta::assert_debug_snapshot!(all_diagnostic_summary(&output.results));

    let tree = output
        .provide_inject_tree
        .as_deref()
        .expect("tree should be rendered");
    insta::assert_snapshot!(tree);
}

#[test]
fn cross_file_opt_in_reports_duplicate_element_ids_at_template_offsets() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("First.vue");
    let second = dir.path().join("Second.vue");

    let first_source = r#"<script setup lang="ts">
const ready = true
</script>

<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
"#;
    fs::write(&first, first_source).unwrap();
    fs::write(
        &second,
        r#"<template>
  <input id="email" />
</template>
"#,
    )
    .unwrap();

    let files = [&first, &second]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

    let first_result = result_for_file(&output.results, "First.vue");
    insta::assert_debug_snapshot!(diagnostic_summary(first_result));
    assert_eq!(first_result.diagnostics.len(), 1);
    let diagnostic = &first_result.diagnostics[0];

    let expected_start = first_source.find("id=\"email\"").unwrap() as u32;
    assert_eq!(diagnostic.start, expected_start);
    assert!(diagnostic.end > diagnostic.start);
}

#[test]
fn cross_file_opt_in_skips_template_ast_after_fatal_parse_error() {
    let dir = tempfile::tempdir().unwrap();
    let malformed = dir.path().join("Malformed.vue");
    let valid = dir.path().join("Valid.vue");

    fs::write(
        &malformed,
        r#"<template>
  <div>
<input id="email">
</template>
"#,
    )
    .unwrap();
    fs::write(
        &valid,
        r#"<template>
  <input id="email" />
</template>
"#,
    )
    .unwrap();

    let files = [&malformed, &valid]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

    assert_eq!(
        all_diagnostic_summary(&output.results),
        Vec::<String>::new()
    );
}

#[test]
fn cross_file_opt_in_reports_reactive_prop_destructure() {
    let dir = tempfile::tempdir().unwrap();
    let parent = dir.path().join("Parent.vue");
    let child = dir.path().join("Child.vue");

    fs::write(
        &parent,
        r#"<script setup lang="ts">
import { reactive } from 'vue'
import Child from './Child.vue'

const state = reactive({ count: 0 })
</script>

<template>
  <Child :item="state" />
</template>
"#,
    )
    .unwrap();
    fs::write(
        &child,
        r#"<script setup lang="ts">
const props = defineProps<{ item: { count: number } }>()
const { item } = props
</script>
"#,
    )
    .unwrap();

    let files = [&parent, &child]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

    let child_result = result_for_file(&output.results, "Child.vue");
    insta::assert_debug_snapshot!(diagnostic_summary(child_result));
}

#[test]
fn cross_file_opt_in_allows_direct_define_props_destructure_until_aliased() {
    let dir = tempfile::tempdir().unwrap();
    let direct = dir.path().join("Direct.vue");
    let alias = dir.path().join("Alias.vue");

    fs::write(
        &direct,
        r#"<script setup lang="ts">
const { item } = defineProps<{ item: { count: number } }>()
</script>
"#,
    )
    .unwrap();
    fs::write(
        &alias,
        r#"<script setup lang="ts">
const { item } = defineProps<{ item: { count: number } }>()
const item2 = item
</script>
"#,
    )
    .unwrap();

    let files = [&direct, &alias]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

    let direct_result = result_for_file(&output.results, "Direct.vue");
    assert_eq!(diagnostic_summary(direct_result), Vec::<String>::new());

    let alias_result = result_for_file(&output.results, "Alias.vue");
    insta::assert_debug_snapshot!(diagnostic_summary(alias_result));
}

#[test]
fn cross_file_opt_in_reports_loop_element_ids_at_template_offsets() {
    let dir = tempfile::tempdir().unwrap();
    let list = dir.path().join("List.vue");

    let source = r#"<script setup lang="ts">
const rows = [{ name: 'Ada' }]
</script>

<template>
  <ul>
<li v-for="row in rows">
  <span id="row-label">{{ row.name }}</span>
</li>
  </ul>
</template>
"#;
    fs::write(&list, source).unwrap();

    let files = [list]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

    let list_result = result_for_file(&output.results, "List.vue");
    insta::assert_debug_snapshot!(diagnostic_summary(list_result));
    assert_eq!(list_result.diagnostics.len(), 1);
    let diagnostic = &list_result.diagnostics[0];

    let expected_start = source.find("id=\"row-label\"").unwrap() as u32;
    assert_eq!(diagnostic.start, expected_start);
    assert!(diagnostic.end > diagnostic.start);
}

#[test]
fn cross_file_opt_in_reports_async_injected_state_race() {
    let dir = tempfile::tempdir().unwrap();
    let provider = dir.path().join("Provider.vue");
    let child = dir.path().join("Child.vue");

    fs::write(
        &provider,
        r#"<script setup lang="ts">
import { provide, reactive } from 'vue'
import Child from './Child.vue'

const store = reactive({ count: 0 })
provide('store', store)
</script>

<template>
  <Child />
</template>
"#,
    )
    .unwrap();
    fs::write(
        &child,
        r#"<script setup lang="ts">
import { inject, ref, watch } from 'vue'

const store = inject('store')!
const query = ref('')

watch(query, async () => {
  await load()
  store.count = 1
})
</script>
"#,
    )
    .unwrap();

    let files = [&provider, &child]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output = build_cross_file_lint_output(&files, vize_patina::HelpLevel::Short, false);

    let child_result = result_for_file(&output.results, "Child.vue");
    insta::assert_debug_snapshot!(diagnostic_summary(child_result));
}
