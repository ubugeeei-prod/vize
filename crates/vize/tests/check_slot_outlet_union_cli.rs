#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::path::{Path, PathBuf};
use std::process::Command;

/// A child that renders the same named slot from several `<slot>` outlets
/// (one bound, one bare; one bound, one with a static attribute) used to
/// expose only the last outlet's payload to the parent, so
/// `#panel="{ viewMode }"` reported `TS2339` on `{}` and the static
/// `viewMode="sp"` widened to `string` (`TS2322` against a literal prop).
/// vue-tsc reports neither.
#[test]
fn check_same_named_slot_outlets_merge_their_payloads() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project();
    if !project_root.join("node_modules/vue").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--tsconfig", "tsconfig.json", "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let diagnostics = json["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["diagnostics"].as_array().cloned().unwrap_or_default())
        .filter_map(|diagnostic| diagnostic.as_str().map(str::to_owned))
        .collect::<Vec<_>>();

    // `Wrong.vue` passes the merged payload where only a number is accepted:
    // the merge must still type the payload, not erase it to `any`.
    assert_eq!(
        diagnostics.len(),
        1,
        "only the deliberate misuse may be reported; got {diagnostics:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        diagnostics[0].contains("Wrong.vue") && diagnostics[0].contains("[TS2322]"),
        "the merged payload must keep its type; got {diagnostics:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        output.status.code() == Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_cli_project() -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(format!("slot-outlet-union-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_node_modules(&project_root);
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/Layout.vue"),
        r#"<template>
  <div>
    <template v-if="isPC">
      <slot name="panel" :viewMode="viewMode" />
      <slot name="side" :viewMode="viewMode" />
    </template>
    <template v-else>
      <slot name="panel" />
      <slot name="side" viewMode="sp" />
    </template>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'

export default defineComponent({
  name: 'Layout',
  props: { viewMode: String as PropType<'pc' | 'sp'> },
  computed: {
    isPC(): boolean {
      return this.viewMode === 'pc'
    },
  },
})
</script>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/Preview.vue"),
        r#"<template>
  <div>{{ viewMode }}</div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'

export default defineComponent({
  name: 'Preview',
  props: { viewMode: String as PropType<'pc' | 'sp'> },
})
</script>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/Parent.vue"),
        r#"<template>
  <Layout :viewMode="mode">
    <template #panel="{ viewMode }">
      <Preview :viewMode="viewMode" />
    </template>
    <template #side="{ viewMode }">
      <Preview :viewMode="viewMode" />
    </template>
  </Layout>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import Layout from './Layout.vue'
import Preview from './Preview.vue'

export default defineComponent({
  name: 'Parent',
  components: { Layout, Preview },
  data() {
    return { mode: 'pc' as 'pc' | 'sp' }
  },
})
</script>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/Wrong.vue"),
        r#"<template>
  <Layout viewMode="pc">
    <template #side="{ viewMode }">
      <Counter :count="viewMode" />
    </template>
  </Layout>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import Layout from './Layout.vue'

const Counter = defineComponent({
  name: 'Counter',
  props: { count: { type: Number, required: true } },
})

export default defineComponent({
  name: 'Wrong',
  components: { Layout, Counter },
})
</script>
"#,
    )
    .unwrap();
    project_root
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn link_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    if source.exists() {
        symlink_path(&source, &project_root.join("node_modules")).unwrap();
    }
}

fn resolve_test_corsa_path() -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path.display().to_string());
        }
    }
    let workspace_root = workspace_root();
    [workspace_root.join("node_modules/.bin/tsgo")]
        .into_iter()
        .find(|candidate| candidate.exists())
        .map(|candidate| candidate.display().to_string())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
