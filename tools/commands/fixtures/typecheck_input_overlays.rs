//! Additional generated inputs remain pinned on both sides of a comparison.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

fn relative_file(value: &str) -> bool {
    !value.is_empty()
        && value.split('/').all(|part| {
            !part.is_empty()
                && ![".", "..", ".git", "node_modules"].contains(&part)
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        })
}

fn read_regular(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let mut path = root.canonicalize().map_err(|error| error.to_string())?;
    for (index, part) in relative.split('/').enumerate() {
        path.push(part);
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        let last = index + 1 == relative.split('/').count();
        if metadata.is_symlink() || (last && !metadata.is_file()) || (!last && !metadata.is_dir()) {
            return Err(format!(
                "Typecheck input overlay has an invalid file kind: {relative}"
            ));
        }
    }
    fs::read(path).map_err(|error| error.to_string())
}

pub(super) fn validate(
    root: &Path,
    fixture_root: &Path,
    project: &Value,
    artifact: &Value,
) -> Result<Option<Vec<Value>>, String> {
    let Some(inputs) = project.pointer("/typecheckPerformance/baseline/inputOverlays") else {
        if artifact.pointer("/baselinePrepare/inputOverlays").is_some() {
            return Err("Unexpected typecheck input overlay evidence".into());
        }
        return Ok(None);
    };
    let entries = inputs
        .as_array()
        .filter(|entries| !entries.is_empty())
        .ok_or("Typecheck input overlays must be a nonempty array")?;
    let mut targets = BTreeSet::<String>::new();
    let mut records = Vec::new();
    for entry in entries {
        let object = entry.as_object().ok_or("Invalid typecheck input overlay")?;
        let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
        keys.sort_unstable();
        if keys != ["sha256", "source", "target"] {
            return Err("Typecheck input overlay fields differ".into());
        }
        let source = entry["source"].as_str().ok_or("Invalid overlay source")?;
        let target = entry["target"].as_str().ok_or("Invalid overlay target")?;
        let hash = entry["sha256"].as_str().ok_or("Invalid overlay hash")?;
        if !relative_file(source)
            || !source.starts_with("tests/_fixtures/typecheck-inputs/")
            || !relative_file(target)
            || hash.len() != 64
            || !hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || targets.iter().any(|prior| {
                target == prior
                    || target.starts_with(&format!("{prior}/"))
                    || prior.starts_with(&format!("{target}/"))
            })
        {
            return Err("Invalid typecheck input overlay coordinates".into());
        }
        targets.insert(target.into());
        let original = read_regular(root, source)?;
        let prepared = read_regular(fixture_root, target)?;
        if format!("{:x}", Sha256::digest(&original)) != hash || prepared != original {
            return Err(format!("Typecheck input overlay bytes differ: {target}"));
        }
        records.push(json!({
            "source": source,
            "target": target,
            "sha256": hash,
            "sizeBytes": original.len(),
        }));
    }
    if artifact.pointer("/baselinePrepare/inputOverlays") != Some(&json!(records))
        || artifact.pointer("/baselinePrepare/command")
            != project.pointer("/typecheckPerformance/baseline/prepare")
        || artifact
            .pointer("/baselinePrepare/exitCode")
            .and_then(Value::as_i64)
            != Some(0)
    {
        return Err("Typecheck input overlay preparation evidence differs".into());
    }
    Ok(Some(records))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn fixture() -> (Fixture, PathBuf, Value, Value) {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "vize-input-overlays-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&root).unwrap();
        let owner = Fixture(root.clone());
        let source = "tests/_fixtures/typecheck-inputs/project/schema.gql";
        let target = "gql-gen/schema.gql";
        let upstream = root.join("upstream");
        fs::create_dir_all(root.join(source).parent().unwrap()).unwrap();
        fs::create_dir_all(upstream.join(target).parent().unwrap()).unwrap();
        let bytes = b"type Query { answer: Int! }\n";
        fs::write(root.join(source), bytes).unwrap();
        fs::write(upstream.join(target), bytes).unwrap();
        let input = json!({"source": source, "target": target, "sha256": format!("{:x}", Sha256::digest(bytes))});
        let command = json!([["pnpm", "run", "codegen"]]);
        let project = json!({"typecheckPerformance":{"baseline":{"inputOverlays":[input.clone()],"prepare":command.clone()}}});
        let mut prepared = input;
        prepared["sizeBytes"] = json!(bytes.len());
        let artifact =
            json!({"baselinePrepare":{"inputOverlays":[prepared],"command":command,"exitCode":0}});
        (owner, upstream, project, artifact)
    }

    #[test]
    fn exact_generated_input_and_evidence_are_required_at_the_rust_report_boundary() {
        let (owner, upstream, project, artifact) = fixture();
        assert_eq!(
            validate(&owner.0, &upstream, &project, &artifact)
                .unwrap()
                .unwrap()
                .len(),
            1
        );
        fs::write(
            upstream.join("gql-gen/schema.gql"),
            "changed after preparation",
        )
        .unwrap();
        assert!(
            validate(&owner.0, &upstream, &project, &artifact)
                .unwrap_err()
                .contains("bytes differ")
        );
    }

    #[test]
    fn a_missing_or_changed_generated_input_receipt_cannot_claim_preparation() {
        let (owner, upstream, project, mut artifact) = fixture();
        artifact["baselinePrepare"]["inputOverlays"][0]["sizeBytes"] = json!(0);
        assert!(validate(&owner.0, &upstream, &project, &artifact).is_err());
        artifact["baselinePrepare"]
            .as_object_mut()
            .unwrap()
            .remove("inputOverlays");
        assert!(validate(&owner.0, &upstream, &project, &artifact).is_err());
    }

    #[test]
    fn old_receipts_keep_their_scope_and_reject_unregistered_generated_input_claims() {
        let root = Path::new("not-read-for-an-ordinary-project");
        assert_eq!(validate(root, root, &json!({}), &json!({})).unwrap(), None);
        assert!(
            validate(
                root,
                root,
                &json!({}),
                &json!({"baselinePrepare":{"inputOverlays":[]}})
            )
            .is_err()
        );
    }
}
