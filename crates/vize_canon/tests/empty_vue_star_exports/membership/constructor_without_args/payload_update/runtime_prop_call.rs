//! Invert the #11734 call-before-construct runtime prop inference change first.
use super::support;
use serde_json::{Value, json};

struct File {
    path: &'static str,
    before: &'static str,
    after: &'static str,
    before_bytes: usize,
    before_sha: &'static str,
    after_sha: &'static str,
}

const FILES: [File; 3] = [
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt",
        before: r###"type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V ? V : T extends (...args: any[]) => infer V ? V : never;"###,
        after: r###"type __RuntimePropValue<T> = T extends (...args: any[]) => infer V ? V : T extends abstract new (...args: any[]) => infer V ? V : never;"###,
        before_bytes: 2004,
        before_sha: "1cb95f44b238332378bab53af58bbe0b95ad428d2a846f3136b81bfc3590cb98",
        after_sha: "3767bffe638c34126238eb7122ed70be0c5d1cbc8a79349aa10bdbc08f0820b5",
    },
    File {
        path: "npm/cli/virtual-ts-globals.d.ts",
        before: r###"  type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V
    ? V
    : T extends (...args: any[]) => infer V
      ? V
      : never;"###,
        after: r###"  type __RuntimePropValue<T> = T extends (...args: any[]) => infer V
    ? V
    : T extends abstract new (...args: any[]) => infer V
      ? V
      : never;"###,
        before_bytes: 19261,
        before_sha: "1eb094d7bc31585ac250c348d50dd22dfbaea6a3f8049cb38d5f1b965e001afd",
        after_sha: "1077ed40b76672245658ad66ded95531ccfbe41405b713b44126c1704f6c1cd2",
    },
    File {
        path: "npm/cli/generated/content-mapper-types.d.ts",
        before: r###"type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V
  ? V
  : T extends (...args: any[]) => infer V
    ? V
    : never;"###,
        after: r###"type __RuntimePropValue<T> = T extends (...args: any[]) => infer V
  ? V
  : T extends abstract new (...args: any[]) => infer V
    ? V
    : never;"###,
        before_bytes: 11420,
        before_sha: "8fff1e27182013fa69009b04ea502394ee4193081c8e1ff966aeb4cbc5b16215",
        after_sha: "1acd0abc0a47eeea5d3a6bbe6a43ace358d394c29620d8230bb1cdc01cf701f1",
    },
];
const PREAMBLE_BEFORE_SHA: &str =
    "08ebeeb3d5303bd130213edd33583849ef8e078d33c0e366562d40efd3a9985d";
const PREAMBLE_AFTER_SHA: &str = "378516f3c59cecb166ab9197761b77679824ea7145efd84d752bf5afd3060b07";

fn inverse(current: &str, before: &str, after: &str, bytes: usize, old: &str, new: &str) -> String {
    assert_eq!(current.len(), bytes + after.len() - before.len());
    assert_eq!(support::hash(current.as_bytes()), new);
    assert_eq!(current.matches(after).count(), 1);
    assert_eq!(current.matches(before).count(), 0);
    let previous = current.replacen(after, before, 1);
    assert_eq!(previous.len(), bytes);
    assert_eq!(support::hash(previous.as_bytes()), old);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::current_loose_emit::source_before(path, current);
    let Some(file) = FILES.iter().find(|file| file.path == path) else {
        return current;
    };
    inverse(
        &current,
        file.before,
        file.after,
        file.before_bytes,
        file.before_sha,
        file.after_sha,
    )
}

pub(super) fn preamble_before(current: &str) -> String {
    let current = &super::current_loose_emit::preamble_before(current);
    let file = &FILES[0];
    inverse(
        current,
        file.before,
        file.after,
        44748,
        PREAMBLE_BEFORE_SHA,
        PREAMBLE_AFTER_SHA,
    )
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentRuntimePropCallUpdate"] = json!({
        "kind": "exact-current-runtime-prop-call-inverse", "issue": 11734,
        "nativeAcceptanceCredit": false,
        "files": FILES.iter().map(|file| json!({"path":file.path,"beforeBytes":file.before_bytes,
            "afterBytes":file.before_bytes + file.after.len() - file.before.len(),
            "beforeSha256":file.before_sha,"afterSha256":file.after_sha})).collect::<Vec<_>>(),
        "compiledPreamble": {"beforeSha256": PREAMBLE_BEFORE_SHA,"afterSha256":PREAMBLE_AFTER_SHA}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_assets_and_preamble_reconstruct_only_the_runtime_prop_call_layer() {
        for file in &FILES {
            let body = std::fs::read_to_string(support::repository().join(file.path)).unwrap();
            let body = super::super::integrated_source::source_before(file.path, body);
            let body = super::super::checker_augmentations::source_before(file.path, body);
            assert_eq!(
                source_before(file.path, body.clone()).replacen(file.before, file.after, 1),
                body
            );
            for mutated in [
                body.replacen(file.after, "", 1),
                format!("{body}{}", file.after),
                body.replacen(file.after, file.before, 1),
                format!("{body}\n"),
            ] {
                assert!(
                    std::panic::catch_unwind(|| inverse(
                        &mutated,
                        file.before,
                        file.after,
                        file.before_bytes,
                        file.before_sha,
                        file.after_sha,
                    ))
                    .is_err()
                );
            }
        }
        let body = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        let integrated = super::super::integrated_source::preamble_before(body);
        let previous_loose = super::super::current_loose_emit::preamble_before(&integrated);
        assert_eq!(
            preamble_before(&integrated).replacen(FILES[0].before, FILES[0].after, 1),
            previous_loose
        );
    }
}
