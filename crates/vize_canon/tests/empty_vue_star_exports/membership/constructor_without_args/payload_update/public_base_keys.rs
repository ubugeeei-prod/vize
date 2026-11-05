//! Invert only the #11445 `$ComponentPublicBase` key-remap before the stall layer.
use super::support;
use serde_json::{Value, json};

const PREAMBLE_BEFORE_BYTES: usize = 44_244;
const PREAMBLE_BEFORE_SHA: &str =
    "298f0dd19a3114ee2fc55f67b598a5a3b1649c356b4f453be52312b854e65816";
const PREAMBLE_AFTER_SHA: &str = "b2158cd05d330ec5da8876b5128d621a9130fdf799cbae6a7f79357999374eb2";
const ASSET: (&str, &str) = (
    "type $ComponentPublicBase = Omit<import('vue').ComponentPublicInstance, '$props' | '$emit' | '$slots'>;\n",
    "type $ComponentPublicBase<I = import('vue').ComponentPublicInstance> = Omit<I, '$props' | '$emit' | '$slots'> & Pick<I, Exclude<$LiteralKeys<I>, '$props' | '$emit' | '$slots'>>;\n",
);

struct File {
    path: &'static str,
    edits: &'static [(&'static str, &'static str)],
    before_bytes: usize,
    before_sha: &'static str,
    after_sha: &'static str,
}

const GLOBALS_EDITS: [(&str, &str); 1] = [(
    "  type $ComponentPublicBase = Omit<\n    // @ts-ignore TS2694/TS2307: a `vue` without `ComponentPublicInstance` must degrade component public extras to unchecked, never error.\n    import(\"vue\").ComponentPublicInstance,\n    \"$props\" | \"$emit\" | \"$slots\"\n  >;\n",
    "  type $ComponentPublicBase<\n    // @ts-ignore TS2694/TS2307: a `vue` without `ComponentPublicInstance` must degrade component public extras to unchecked, never error.\n    I = import(\"vue\").ComponentPublicInstance,\n  > = Omit<I, \"$props\" | \"$emit\" | \"$slots\"> &\n    Pick<I, Exclude<$LiteralKeys<I>, \"$props\" | \"$emit\" | \"$slots\">>;\n",
)];

const GENERATED_EDITS: [(&str, &str); 1] = [(
    "\ntype $ComponentPublicBase = Omit<\n  // @ts-ignore TS2694/TS2307: a `vue` without `ComponentPublicInstance` must degrade component public extras to unchecked, never error.\n  import(\"vue\").ComponentPublicInstance,\n  \"$props\" | \"$emit\" | \"$slots\"\n>;\n",
    "\ntype $ComponentPublicBase<\n  // @ts-ignore TS2694/TS2307: a `vue` without `ComponentPublicInstance` must degrade component public extras to unchecked, never error.\n  I = import(\"vue\").ComponentPublicInstance,\n> = Omit<I, \"$props\" | \"$emit\" | \"$slots\"> &\n  Pick<I, Exclude<$LiteralKeys<I>, \"$props\" | \"$emit\" | \"$slots\">>;\n",
)];

const FILES: [File; 4] = [
    File {
        path: "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
        edits: &[ASSET],
        before_bytes: 5174,
        before_sha: "89cdc5975a77a66d891f1670d58220c9617d66b3faaa5136bfc61f462a7f74e8",
        after_sha: "fad7fd90ea638b5494ecba61abc5af885f5a4d0748bdaf0079bdc6ced416d819",
    },
    File {
        path: "npm/cli/virtual-ts-globals.d.ts",
        edits: &GLOBALS_EDITS,
        before_bytes: 19_419,
        before_sha: "0ad1f8a8892956b693732431fc6a1e277ee1c1b929d750cadf846248ae1a981e",
        after_sha: "56b1845fb5f01ef50151394f38993ed7bc554ecf0f1b446d51c7cdfd0be20eed",
    },
    File {
        path: "npm/cli/generated/content-mapper-types.d.ts",
        edits: &GENERATED_EDITS,
        before_bytes: 11_202,
        before_sha: "7457fd86aafb4b2a5b1edbddedf538a45a072517645dd0ece2fd4c074a0f632e",
        after_sha: "ea0127730ede44aef84f12775c1b49299a245486d6587cd0e766db41ee730eac",
    },
    File {
        path: "npm/cli/scripts/generate-content-mapper-types.ts",
        edits: &[(
            "Declaration = \"type $ComponentPublicBase = Omit<\";",
            "Declaration = \"type $ComponentPublicBase<\";",
        )],
        before_bytes: 17_827,
        before_sha: "d10c03704031ea315e68123a31bb08902b029e329920c52452eef94c304c5307",
        after_sha: "a32212a7833a6fcc7aa8998c0c9489dcdde895fb482a2143dad2b0708f932503",
    },
];

fn delta(edits: &[(&str, &str)]) -> isize {
    edits
        .iter()
        .map(|(before, after)| after.len() as isize - before.len() as isize)
        .sum()
}

fn invert(
    current: &str,
    edits: &[(&str, &str)],
    before_bytes: usize,
    (before_sha, after_sha): (&str, &str),
) -> String {
    assert_eq!(current.len() as isize, before_bytes as isize + delta(edits));
    assert_eq!(support::hash(current.as_bytes()), after_sha);
    let mut previous = current.to_owned();
    for (before, after) in edits {
        assert_eq!(previous.matches(after).count(), 1);
        assert_eq!(previous.matches(before).count(), 0);
        previous = previous.replacen(after, before, 1);
    }
    assert_eq!(previous.len(), before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::emit_props_twins::source_before(path, current);
    match FILES.iter().find(|file| file.path == path) {
        Some(file) => invert(
            &current,
            file.edits,
            file.before_bytes,
            (file.before_sha, file.after_sha),
        ),
        None => current,
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    invert(
        &super::emit_props_twins::preamble_before(current),
        &[ASSET],
        PREAMBLE_BEFORE_BYTES,
        (PREAMBLE_BEFORE_SHA, PREAMBLE_AFTER_SHA),
    )
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    let files = FILES
        .iter()
        .map(|file| {
            json!({ "path": file.path, "edits": file.edits.len(), "beforeBytes": file.before_bytes,
                "afterBytes": file.before_bytes as isize + delta(file.edits),
                "beforeSha256": file.before_sha, "afterSha256": file.after_sha })
        })
        .collect::<Vec<_>>();
    manifest["provenance"]["currentPublicBaseKeysUpdate"] = json!({
        "kind": "exact-current-public-base-key-remap-inverse",
        "issue": 11445,
        "nativeAcceptanceCredit": false,
        "files": files,
        "compiledPreamble": { "path": "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS",
            "beforeBytes": PREAMBLE_BEFORE_BYTES,
            "afterBytes": PREAMBLE_BEFORE_BYTES as isize + delta(&[ASSET]),
            "beforeSha256": PREAMBLE_BEFORE_SHA, "afterSha256": PREAMBLE_AFTER_SHA }
    });
}

#[cfg(test)]
mod tests {
    use super::{FILES, preamble_before, retain_provenance, source_before};
    use serde_json::json;

    fn current(path: &str) -> String {
        std::fs::read_to_string(super::support::repository().join(path)).unwrap()
    }

    fn reapply(prior: &str, edits: &[(&str, &str)]) -> String {
        edits
            .iter()
            .rev()
            .fold(prior.to_owned(), |text, (before, after)| {
                text.replacen(before, after, 1)
            })
    }

    #[test]
    fn current_assets_preamble_and_sdk_reconstruct_only_the_public_base_layer() {
        for file in &FILES {
            let body = current(file.path);
            let prior = source_before(file.path, body.clone());
            assert_eq!(
                reapply(&prior, file.edits),
                super::super::emit_props_twins::source_before(file.path, body)
            );
        }
        let preamble = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        assert_eq!(
            preamble_before(preamble).replacen(super::ASSET.0, super::ASSET.1, 1),
            super::super::emit_props_twins::preamble_before(preamble)
        );
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt";
        assert_eq!(source_before(other, current(other)), current(other));
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
    }

    #[test]
    fn missing_duplicate_stale_and_extra_bytes_refuse() {
        for file in &FILES {
            let body = current(file.path);
            for (before, after) in file.edits {
                for mutated in [
                    body.replacen(after, "", 1),
                    format!("{body}{after}"),
                    body.replacen(after, before, 1),
                    format!("{body}\n"),
                ] {
                    assert!(
                        std::panic::catch_unwind(|| source_before(file.path, mutated.clone()))
                            .is_err()
                    );
                }
            }
        }
    }
}
