use std::{fs, path::PathBuf};

const FAST_PATHS: &[&str] = &[
    "src/core/corsa_core/src/process.rs",
    "src/core/corsa_runtime/src/broadcast.rs",
    "src/core/corsa_lsp/src/lsp/client.rs",
    "src/core/corsa_lsp/src/lsp/overlay.rs",
    "src/core/corsa_lsp/src/lsp/virtual_document.rs",
    "src/core/corsa_lsp/src/lsp/custom.rs",
    "src/core/corsa_orchestrator/src/orchestrator/api.rs",
    "src/core/corsa_orchestrator/src/orchestrator/distributed.rs",
    "src/core/corsa_orchestrator/src/orchestrator/raft/config.rs",
    "src/core/corsa_orchestrator/src/orchestrator/raft/messages.rs",
    "src/core/corsa_orchestrator/src/orchestrator/raft/mod.rs",
    "src/core/corsa_orchestrator/src/orchestrator/raft/node.rs",
    "src/core/corsa_orchestrator/src/orchestrator/raft/transport.rs",
    "src/core/corsa_orchestrator/src/orchestrator/state.rs",
    "src/core/corsa_ref/src/git.rs",
    "src/core/corsa_ref/src/lockfile.rs",
    "src/core/corsa_ref/src/manager.rs",
    "src/core/corsa_ref/src/status.rs",
    "src/core/corsa_ref/src/main.rs",
];

#[test]
fn fast_modules_avoid_std_alloc_shorthands() {
    let root = workspace_root();
    let mut violations = Vec::new();
    for relative in FAST_PATHS {
        let path = root.join(relative);
        let content = fs::read_to_string(&path).unwrap();
        let body = content.split("\n#[cfg(test)]").next().unwrap_or(&content);
        for (index, line) in body.lines().enumerate() {
            if line.contains("format!(")
                || contains_word(line, "String")
                || contains_word(line, "Vec")
            {
                violations.push(format!("{}:{}", relative, index + 1));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "fast-path allocation policy violated:\n{}",
        violations.join("\n")
    );
}

fn workspace_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            return dir;
        }
        assert!(
            dir.pop(),
            "failed to locate workspace root from {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
}

fn contains_word(line: &str, word: &str) -> bool {
    let bytes = line.as_bytes();
    let needle = word.as_bytes();
    let mut offset = 0;
    while let Some(index) = line[offset..].find(word) {
        let start = offset + index;
        let end = start + needle.len();
        let left = start
            .checked_sub(1)
            .and_then(|index| bytes.get(index).copied())
            .is_none_or(|byte| !is_ident(byte));
        let right = bytes.get(end).copied().is_none_or(|byte| !is_ident(byte));
        if left && right {
            return true;
        }
        offset = end;
    }
    false
}

fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}
