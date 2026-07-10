use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn check_options_api_prop_type_long_comments_has_no_ts_parse_errors() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_options_api_long_comment_project();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/OptionsApiPropsLongComment.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
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

    let parse_error_codes = diagnostics
        .iter()
        .filter_map(|diagnostic| ts_diagnostic_code(diagnostic))
        .filter(|code| matches!(code, 1109 | 1128 | 1131))
        .collect::<Vec<_>>();
    assert_eq!(
        parse_error_codes,
        Vec::<u32>::new(),
        "Options API PropType props after long comments must not generate virtual-TS syntax errors; got {diagnostics:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        output.status.success(),
        "long-comment Options API props fixture should type-check:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], 0);

    let _ = std::fs::remove_dir_all(&project_root);
}

fn ts_diagnostic_code(diagnostic: &str) -> Option<u32> {
    let marker = diagnostic.find("[TS")?;
    let start = marker + 3;
    let end = diagnostic[start..].find(']')? + start;
    diagnostic[start..end].parse().ok()
}

fn create_options_api_long_comment_project() -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(format!(
            "options-api-props-long-comment-{}",
            std::process::id()
        ));
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
        project_root.join("src/OptionsApiPropsLongComment.vue"),
        r#"<script lang="ts">
import { defineComponent, PropType } from 'vue'

interface Item { id: number }

// long enough comment line one xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
// long enough comment line two xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
// long enough comment line three xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
// long enough comment line four xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
// long enough comment line five xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

export default defineComponent({
  props: {
    items: {
      type: Array as PropType<Item[]>,
      required: false,
    },
    size: {
      type: Number,
      default: 0,
    },
  },
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
