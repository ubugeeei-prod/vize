use std::{collections::BTreeMap, fs, process::Command};

#[test]
fn inspector_compare_preserves_musea_define_art_script_setup() {
    if !dev_vue_compiler_available() {
        return;
    }

    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("CmsButton.art.vue"),
        r#"<script setup lang="ts">
defineArt("./CmsButton.vue", {
  title: "CmsButton",
  category: "Components",
  status: "draft",
  tags: ["button", "form"],
});
</script>

<art>
  <variant name="Primary" default>
    <CmsButton color="primary">Primary</CmsButton>
  </variant>
</art>
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["inspector", "src/CmsButton.art.vue", "--format", "compare"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let vize_code = json["files"][0]["vize"]["code"].as_str().unwrap();
    let official_code = json["files"][0]["official"]["code"].as_str().unwrap();
    assert_eq!(
        define_art_first_args(official_code),
        vec!["\"./CmsButton.vue\""]
    );
    assert_eq!(
        define_art_first_args(vize_code),
        vec!["\"./CmsButton.vue\""]
    );
}

#[test]
fn inspector_compare_preserves_musea_define_art_script_setup_matrix() {
    if !dev_vue_compiler_available() {
        return;
    }

    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("art-meta.ts"),
        r#"export const buttonPath = "./AliasButton.vue";
export const metadata = { title: "AliasButton" };
"#,
    )
    .unwrap();
    fs::write(
        src.join("AliasButton.art.vue"),
        r#"<script lang="ts">
export const localKind = "mixed";
</script>

<script setup lang="ts">
import { buttonPath as componentPath, metadata } from "./art-meta";

defineArt(componentPath, {
  title: metadata.title,
  category: "Components",
  tags: ["button", localKind],
});

defineArt("./AliasButton.secondary.vue", {
  title: "AliasButtonSecondary",
});
</script>

<art>
  <variant name="Primary" default>
    <AliasButton>Primary</AliasButton>
  </variant>
</art>
"#,
    )
    .unwrap();
    fs::write(
        src.join("PlainBadge.art.vue"),
        r#"<script setup>
defineArt("./PlainBadge.vue", {
  title: "PlainBadge",
  category: "Components",
});
</script>

<art>
  <variant name="Default" default>
    <PlainBadge />
  </variant>
</art>
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["inspector", "src", "--format", "compare"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let summaries = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            let path = file["path"].as_str().unwrap().to_owned();
            let official = define_art_first_args(file["official"]["code"].as_str().unwrap());
            let vize = define_art_first_args(file["vize"]["code"].as_str().unwrap());
            (path, (official, vize))
        })
        .collect::<BTreeMap<_, _>>();

    assert_eq!(
        summaries,
        BTreeMap::from([
            (
                "src/AliasButton.art.vue".to_owned(),
                (
                    vec![
                        "componentPath".to_owned(),
                        "\"./AliasButton.secondary.vue\"".to_owned()
                    ],
                    vec![
                        "componentPath".to_owned(),
                        "\"./AliasButton.secondary.vue\"".to_owned()
                    ],
                ),
            ),
            (
                "src/PlainBadge.art.vue".to_owned(),
                (
                    vec!["\"./PlainBadge.vue\"".to_owned()],
                    vec!["\"./PlainBadge.vue\"".to_owned()],
                ),
            ),
        ])
    );
}

fn define_art_first_args(code: &str) -> Vec<String> {
    let marker = "defineArt(";
    let mut args = Vec::new();
    let mut offset = 0;
    while let Some(index) = code[offset..].find(marker) {
        let start = offset + index + marker.len();
        if is_identifier_boundary(code, offset + index, "defineArt".len())
            && let Some(arg) = first_call_arg(&code[start..])
        {
            args.push(arg);
        }
        offset = start;
    }
    args
}

fn is_identifier_boundary(code: &str, start: usize, len: usize) -> bool {
    let before = start
        .checked_sub(1)
        .and_then(|index| code.as_bytes().get(index))
        .copied();
    let after = code.as_bytes().get(start + len).copied();
    before.is_none_or(|byte| !is_identifier_byte(byte))
        && after.is_none_or(|byte| !is_identifier_byte(byte))
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn first_call_arg(source: &str) -> Option<String> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;

    for (index, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '(' | '[' | '{' => depth += 1,
            ')' if depth == 0 => return Some(source[..index].trim().to_owned()),
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => return Some(source[..index].trim().to_owned()),
            _ => {}
        }
    }
    None
}

fn dev_vue_compiler_available() -> bool {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(workspace_root) = manifest_dir.parent().and_then(|path| path.parent()) else {
        return false;
    };
    Command::new("node")
        .current_dir(workspace_root)
        .args(["--input-type=module", "-e", "import('vue/compiler-sfc')"])
        .status()
        .is_ok_and(|status| status.success())
}
