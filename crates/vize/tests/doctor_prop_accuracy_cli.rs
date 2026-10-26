#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::{fs, path::Path, process::Command, time::Instant};
use vize_doctor::{DoctorCategory, DoctorReport};

#[test]
fn model_alias_and_native_fallthrough_props_preserve_the_complete_healthy_packet() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "tsconfig.json",
        r##"{"compilerOptions":{"paths":{"#shared/*":["./src/shared/*"]}}}"##,
    );
    write(
        directory.path(),
        "package.json",
        r##"{"imports":{"#shared/*":"./src/shared/*"}}"##,
    );
    write(
        directory.path(),
        "src/shared/item-property.ts",
        "export type ItemProperty = string | ((item: Record<string, unknown>) => string)",
    );
    write(
        directory.path(),
        "src/ChildModel.vue",
        r#"<script setup lang="ts">
const model = defineModel<string>({ required: true })
const keyword = defineModel<string>('keyword', { required: true })
defineProps<{ label: string }>()
</script><template><label>{{ label }}<input v-model="keyword" /></label></template>"#,
    );
    write(
        directory.path(),
        "src/ChildAlias.vue",
        r##"<script setup lang="ts">
import type { ItemProperty } from '#shared/item-property'
const { itemText = 'text' } = defineProps<{ itemText?: ItemProperty; animate?: true }>()
</script><template><span>{{ typeof itemText }}</span></template>"##,
    );
    write(
        directory.path(),
        "src/Parent.vue",
        r#"<script setup lang="ts">
import { ref } from 'vue'
import ChildModel from './ChildModel.vue'
import ChildAlias from './ChildAlias.vue'
const keyword = ref('')
</script><template><div>
<ChildModel v-model="keyword" v-model:keyword="keyword" label="Search"
 :model-modifiers="{ trim: true }" :keyword-modifiers="{ trim: true }"
 data-location="search" aria-label="search box" tabindex="0" role="search"
 controls preload="auto" custom-hook="ok" class="field" style="color: red" />
<ChildAlias item-text="name" animate />
</div></template>"#,
    );

    let report = doctor(directory.path(), "models-alias-fallthrough");
    assert_eq!(
        serde_json::to_value(&report).unwrap(),
        serde_json::to_value(DoctorReport::new(".", [])).unwrap(),
        "valid props and inherited attributes must not incur a category or overall penalty"
    );
}

#[test]
fn known_invalid_static_literals_keep_their_errors_and_scores() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/Child.vue",
        r#"<script setup lang="ts">
type Mode = 'small' | 'large'
defineProps<{ count: number; mode: Mode; animate?: true }>()
</script><template><span /></template>"#,
    );
    write(
        directory.path(),
        "src/Parent.vue",
        r#"<script setup lang="ts">
import Child from './Child.vue'
</script><template><Child count="1" mode="invalid" :animate="false" /></template>"#,
    );
    let report = doctor(directory.path(), "known-invalid-literals");
    assert_eq!(report.findings().len(), 3);
    assert!(
        report
            .findings()
            .iter()
            .all(|finding| finding.code == "VIZE_DOCTOR_CF_PROP_TYPE_MISMATCH")
    );
    assert_eq!(report.summary().counts.errors, 3);
    assert!(report.summary().has_blocking_errors);
    assert_eq!(
        report
            .summary()
            .categories
            .get(&DoctorCategory::Correctness)
            .unwrap()
            .score,
        10
    );
    assert_eq!(report.summary().overall_score, 85);
}

#[test]
fn fallthrough_diagnostics_follow_root_shape_and_explicit_forwarding() {
    for (name, options, template, expected_warning) in [
        ("single-root", "", "<button />", false),
        (
            "inherit-disabled",
            "defineOptions({ inheritAttrs: false })",
            "<button />",
            true,
        ),
        ("fragment", "", "<button /><span />", true),
        (
            "explicit-forward",
            "defineOptions({ inheritAttrs: false })",
            "<button v-bind=\"$attrs\" /><span />",
            false,
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        write(
            directory.path(),
            "src/Child.vue",
            &format!(
                "<script setup lang=\"ts\">{options}\ndefineProps<{{ label: string }}>()</script><template>{template}</template>"
            ),
        );
        write(
            directory.path(),
            "src/Parent.vue",
            r#"<script setup lang="ts">
import Child from './Child.vue'
const name = 'data-runtime'
const attrs = { 'data-spread': 'value' }
</script><template><Child label="ok" data-location="button" aria-label="OK" :[name]="'value'" v-bind="attrs" /></template>"#,
        );
        let report = doctor(directory.path(), name);
        let undeclared: Vec<_> = report
            .findings()
            .iter()
            .filter(|finding| finding.code == "VIZE_DOCTOR_CF_UNDECLARED_PROP")
            .collect();
        assert_eq!(!undeclared.is_empty(), expected_warning, "{name}");
        for finding in undeclared {
            assert_eq!(
                serde_json::to_value(&finding.fix).unwrap(),
                serde_json::json!({
                    "safety": "review-required",
                    "title": "Forward attributes to the intended element with `v-bind=\"$attrs\"`, or review the component's `inheritAttrs` setting and root elements."
                })
            );
        }
        assert!(!report.summary().has_blocking_errors, "{name}");
        if !expected_warning {
            assert_eq!(
                serde_json::to_value(&report).unwrap(),
                serde_json::to_value(DoctorReport::new(".", [])).unwrap(),
                "{name}"
            );
        }
    }
}

fn doctor(root: &Path, name: &str) -> DoctorReport {
    let start = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["doctor", "src", "--format", "json", "--exit-zero", "--root"])
        .arg(root)
        .output()
        .unwrap();
    let elapsed = start.elapsed();
    if let Some(artifacts) = std::env::var_os("VIZE_PROP_ACCURACY_ARTIFACTS") {
        let artifacts = Path::new(&artifacts);
        fs::create_dir_all(artifacts).unwrap();
        fs::write(artifacts.join(format!("{name}.json")), &output.stdout).unwrap();
        fs::write(artifacts.join(format!("{name}.stderr")), &output.stderr).unwrap();
        fs::write(artifacts.join(format!("{name}.measurement.json")), serde_json::to_vec_pretty(&serde_json::json!({"elapsedMicros": elapsed.as_micros(), "exitCode": output.status.code()})).unwrap()).unwrap();
    }
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}
