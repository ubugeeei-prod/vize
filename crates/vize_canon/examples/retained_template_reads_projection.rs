//! Complete projection parity probe, copied identically to the baseline build.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_canon::batch::generate_vue_content_mapper_transform;

#[expect(
    clippy::disallowed_types,
    reason = "standalone parity probe error boundary"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("missing corpus directory")?,
    );
    let mut paths = Vec::new();
    collect_files(&root, &mut paths)?;
    paths.sort();
    if paths.is_empty() {
        return Err("projection corpus contains no Vue files".into());
    }
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let source = std::fs::read_to_string(&path)?;
        let transform = generate_vue_content_mapper_transform(&path, &source)?;
        files.push(json!({
            "file": path.strip_prefix(&root)?.to_string_lossy(),
            "transform": transform,
        }));
    }
    let mut edges: Vec<Value> = Vec::new();
    for (name, source) in EDGE_CASES {
        edges.push(json!({
            "file": name,
            "transform": generate_vue_content_mapper_transform(Path::new(name), source)?,
        }));
    }
    println!(
        "{}",
        serde_json::to_string(&json!({ "files": files, "edgeCases": edges }))?
    );
    Ok(())
}

fn collect_files(root: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            if !matches!(entry.file_name().to_str(), Some("node_modules" | ".git")) {
                collect_files(&path, paths)?;
            }
        } else if path.extension().is_some_and(|extension| extension == "vue") {
            paths.push(path);
        }
    }
    Ok(())
}

const EDGE_CASES: &[(&str, &str)] = &[
    (
        "NestedArrow.vue",
        r#"<script setup lang="ts">
const fallback = 1; const key = 'value'; const outer = { value: 2 }; const seed = 3;
const choose = (value: number) => value;
</script><template><button @click="({ [key]: local = fallback, ...rest }) => choose(local + seed + rest.value)">{{ ((value = fallback) => value + outer.value)() }}</button></template>"#,
    ),
    (
        "TypedFallback.vue",
        r#"<script setup lang="ts">
const value = 1; const use = (value: number) => value;
</script><template><button @click="<T,>(input: T) => use(value)">{{ value as number }}</button></template>"#,
    ),
    (
        "CommentsAndStatements.vue",
        r#"<script setup lang="ts">
let value = 1; const ready = true;
</script><template><button v-if="ready /* condition */" @click="value++; value += 2">{{ value /* trailing */ }}</button></template>"#,
    ),
    (
        "UnicodeEntities.vue",
        r#"<script setup lang="ts">
const 雪 = { value: '🌸' }; const other = 'x';
</script><template><div :title="雪.value &amp;&amp; other">{{ 雪.value }}</div></template>"#,
    ),
];
