#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::{fs, path::Path, process::Command, time::Instant};
use vize_doctor::DoctorReport;

#[test]
fn direct_and_reexported_components_have_identical_complete_healthy_reports() {
    for (name, import) in [
        (
            "direct",
            "import AnsweringScreen from './layout/main/MainScreen.vue'",
        ),
        ("barrel", "import { AnsweringScreen } from './layout'"),
        (
            "renamed-chain",
            "import { Screen as AnsweringScreen } from './chain'",
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        project(directory.path());
        write(
            directory.path(),
            "src/Page.vue",
            &format!(
                r#"<script setup lang="ts">
{import}
const current = 'a'
const indexed = {{ id: 1 }}
const playing = false
</script><template><div><template v-if="current">
<AnsweringScreen :current-item="current" :is-playing="playing" :indexed-value="indexed"
 @update:indexed-value="() => {{}}" @action:go-next="() => {{}}" />
</template></div></template>"#
            ),
        );
        let report = doctor(directory.path(), name);
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            serde_json::to_value(DoctorReport::new(".", [])).unwrap(),
            "{name}"
        );
    }
}

#[test]
fn genuinely_missing_barrel_prop_uses_the_authored_alias_and_span() {
    let directory = tempfile::tempdir().unwrap();
    project(directory.path());
    let parent = r#"<script setup lang="ts">
import { AnsweringScreen as PageScreen } from './layout'
</script><template><PageScreen :indexed-value="null" :is-playing="false" /></template>"#;
    write(directory.path(), "src/Page.vue", parent);
    let report = doctor(directory.path(), "missing-required");
    let missing: Vec<_> = report
        .findings()
        .iter()
        .filter(|finding| finding.code == "VIZE_DOCTOR_CF_MISSING_REQUIRED_PROP")
        .collect();
    assert_eq!(missing.len(), 1);
    let finding = missing.first().unwrap();
    assert_eq!(
        finding.message,
        "**Missing Required Prop**: `currentItem` must be passed to `<PageScreen>`\n\nThis prop is declared as required in the component's `defineProps`."
    );
    assert_eq!(
        parent.get(finding.primary.start as usize..finding.primary.end as usize),
        Some("<PageScreen :indexed-value=\"null\" :is-playing=\"false\" />")
    );
    assert_eq!(
        finding.related.first().unwrap().location.path,
        "src/layout/main/MainScreen.vue"
    );
    assert!(report.summary().has_blocking_errors);
}

#[test]
fn same_basename_components_and_repeated_usages_do_not_share_prop_contracts() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "src/admin/Button.vue",
        "<script setup lang=\"ts\">defineProps<{ adminId: string }>()</script><template><button /></template>",
    );
    write(
        directory.path(),
        "src/shop/Button.vue",
        "<script setup lang=\"ts\">defineProps<{ shopId: string }>()</script><template><button /></template>",
    );
    write(
        directory.path(),
        "src/buttons.ts",
        "export { default as AdminButton } from './admin/Button.vue'; export { default as ShopButton } from './shop/Button.vue'",
    );
    let parent = r#"<script setup lang="ts">
import { AdminButton as Button, ShopButton as Shop } from './buttons'
</script><template><div><Button admin-id="admin" /><Shop shop-id="shop" /><Shop /></div></template>"#;
    write(directory.path(), "src/Page.vue", parent);
    let report = doctor(directory.path(), "same-basename-repeated");
    assert_eq!(report.findings().len(), 1);
    let finding = report.findings().first().unwrap();
    assert_eq!(finding.code, "VIZE_DOCTOR_CF_MISSING_REQUIRED_PROP");
    assert_eq!(
        finding.message,
        "**Missing Required Prop**: `shopId` must be passed to `<Shop>`\n\nThis prop is declared as required in the component's `defineProps`."
    );
    assert_eq!(
        parent.get(finding.primary.start as usize..finding.primary.end as usize),
        Some("<Shop />")
    );
    assert_eq!(
        finding.related.first().unwrap().location.path,
        "src/shop/Button.vue"
    );
}

fn project(root: &Path) {
    write(
        root,
        "src/layout/main/MainScreen.vue",
        r#"<script setup lang="ts">
defineProps<{ indexedValue: { id: number } | null; currentItem: string | null; isPlaying: boolean }>()
defineEmits<{ 'update:indexed-value': [value: { id: number }]; 'action:go-next': [] }>()
</script><template><div>{{ currentItem }} {{ indexedValue?.id }} {{ isPlaying }}</div></template>"#,
    );
    write(
        root,
        "src/layout/index.ts",
        "export { default as AnsweringScreen } from './main/MainScreen.vue'",
    );
    write(
        root,
        "src/chain.ts",
        "export { AnsweringScreen as Screen } from './layout'",
    );
}

fn doctor(root: &Path, name: &str) -> DoctorReport {
    let start = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["doctor", "src", "--root"])
        .arg(root)
        .args(["--format", "json", "--exit-zero"])
        .output()
        .unwrap();
    if let Some(artifacts) = std::env::var_os("VIZE_BARREL_PROPS_ARTIFACTS") {
        let artifacts = Path::new(&artifacts);
        fs::create_dir_all(artifacts).unwrap();
        fs::write(artifacts.join(format!("{name}.json")), &output.stdout).unwrap();
        fs::write(artifacts.join(format!("{name}.stderr")), &output.stderr).unwrap();
        fs::write(artifacts.join(format!("{name}.measurement.json")), serde_json::to_vec_pretty(&serde_json::json!({"elapsedMicros":start.elapsed().as_micros(),"exitCode":output.status.code()})).unwrap()).unwrap();
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
