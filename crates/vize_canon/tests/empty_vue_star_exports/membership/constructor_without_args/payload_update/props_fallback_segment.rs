//! Invert only the #11628 split of vue-tsc's props fallback from current main.
use super::support;
use serde_json::{Value, json};

const F0_BEFORE_0: &str = concat!(
    "declare namespace JSX { interface IntrinsicAttributes { class?: unknown; ",
    "style?: unknown; } }\n",
    "// vue-tsc generic library components accept unknown attrs through this\n",
    "// fallback, which vue-tsc itself declares unless `checkUnknownProps` is set.\n",
    "declare var __VLS_PROPS_FALLBACK: Record<string, unknown>;\n\n",
    "// Shared type helpers used by generated virtual modules\n",
);
const F0_AFTER_0: &str = concat!(
    "declare namespace JSX { interface IntrinsicAttributes { class?: unknown; ",
    "style?: unknown; } }\n",
);
const F1_BEFORE_0: &str = concat!(
    "    \"generated/content-mapper-types.d.ts\",\n",
    "    \"virtual-ts-checker.d.ts\",\n",
    "    \"virtual-ts-emit-overloads.d.ts\",\n",
    "    \"virtual-ts-globals.d.ts\",\n",
    "    \"virtual-ts-listeners.d.ts\",\n",
    "    \"virtual-ts-patterns.d.ts\",\n",
);
const F1_AFTER_0: &str = concat!(
    "    \"generated/content-mapper-types.d.ts\",\n",
    "    \"virtual-ts-checker.d.ts\",\n",
    "    \"virtual-ts-checker-strict.d.ts\",\n",
    "    \"virtual-ts-emit-overloads.d.ts\",\n",
    "    \"virtual-ts-globals.d.ts\",\n",
    "    \"virtual-ts-listeners.d.ts\",\n",
    "    \"virtual-ts-patterns.d.ts\",\n",
);
const F1_BEFORE_1: &str = concat!(
    "      \"types\": \"./virtual-ts-checker.d.ts\"\n",
    "    },\n",
    "    \"./content-mapper-helpers\": {\n",
    "      \"import\": {\n",
);
const F1_AFTER_1: &str = concat!(
    "      \"types\": \"./virtual-ts-checker.d.ts\"\n",
    "    },\n",
    "    \"./virtual-ts-checker-strict\": {\n",
    "      \"types\": \"./virtual-ts-checker-strict.d.ts\"\n",
    "    },\n",
    "    \"./content-mapper-helpers\": {\n",
    "      \"import\": {\n",
);
const F2_BEFORE_0: &str = concat!(
    "\npub const SHARED_PREAMBLE_DTS: &str = concat!(\n",
    "    include_str!(\"declarations/shared-header.d.ts.txt\"),\n",
    "    vue_type_helpers_text!(),\n",
    "    include_str!(\"declarations/shared-template-refs.d.ts.txt\"),\n",
    "    include_str!(\"declarations/shared-slot-resolver.d.ts.txt\"),\n",
    "    include_str!(\"declarations/shared-slot-payload.d.ts.txt\"),\n",
    "    emit_overload_helpers_text!(),\n",
    "    include_str!(\"declarations/shared-emit-props.d.ts.txt\"),\n",
    "    include_str!(\"declarations/shared-emit-inference.d.ts.txt\"),\n",
    "    include_str!(\"declarations/shared-listener-args.d.ts.txt\"),\n",
    "    include_str!(\"declarations/component-checks.d.ts.txt\"),\n",
    "    include_str!(\"declarations/shared-macros.d.ts.txt\"),\n",
    "    include_str!(\"pattern_matching.d.ts\")\n",
    ");\n\n",
);
const F2_AFTER_0: &str = concat!(
    "\n/// The program-wide checker preamble. vue-tsc declares its props fallback only\n",
    "/// for files that leave `checkUnknownProps` off, so the fallback is a segment.\n",
    "macro_rules! shared_preamble_text {\n",
    "    ($props_fallback:expr) => {\n",
    "        concat!(\n",
    "            include_str!(\"declarations/shared-header.d.ts.txt\"),\n",
    "            $props_fallback,\n",
    "            include_str!(\"declarations/shared-helpers-banner.d.ts.txt\"),\n",
    "            vue_type_helpers_text!(),\n",
    "            include_str!(\"declarations/shared-template-refs.d.ts.txt\"),\n",
    "            include_str!(\"declarations/shared-slot-resolver.d.ts.txt\"),\n",
    "            include_str!(\"declarations/shared-slot-payload.d.ts.txt\"),\n",
    "            emit_overload_helpers_text!(),\n",
    "            include_str!(\"declarations/shared-emit-props.d.ts.txt\"),\n",
    "            include_str!(\"declarations/shared-emit-inference.d.ts.txt\"),\n",
    "            include_str!(\"declarations/shared-listener-args.d.ts.txt\"),\n",
    "            include_str!(\"declarations/component-checks.d.ts.txt\"),\n",
    "            include_str!(\"declarations/shared-macros.d.ts.txt\"),\n",
    "            include_str!(\"pattern_matching.d.ts\")\n",
    "        )\n",
    "    };\n",
    "}\n\n",
    "/// Preamble of a program with at least one file that leaves `checkUnknownProps` off.\n",
    "pub const SHARED_PREAMBLE_DTS: &str =\n",
    "    shared_preamble_text!(include_str!(\"declarations/props-fallback.d.ts.txt\"));\n",
    "/// Preamble of a program whose every file checks unknown props (#11628).\n",
    "pub const SHARED_PREAMBLE_STRICT_DTS: &str = shared_preamble_text!(\"\");\n\n",
);

struct File {
    path: &'static str,
    edits: &'static [(&'static str, &'static str)],
    before_bytes: usize,
    before_sha: &'static str,
    after_sha: &'static str,
}

const FILES: [File; 3] = [
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-header.d.ts.txt",
        edits: &[(F0_BEFORE_0, F0_AFTER_0)],
        before_bytes: 967,
        before_sha: "1c0fc02eb692116e0b06156b95cfc5e7d1b6e6d023d24bb21ab09c993937bbba",
        after_sha: "f08b5f2e153ff1de311598a9549b9b8a2b541067148a97f0b2f3211cf65413cf",
    },
    File {
        path: "npm/cli/package.json",
        edits: &[(F1_BEFORE_0, F1_AFTER_0), (F1_BEFORE_1, F1_AFTER_1)],
        before_bytes: 4362,
        before_sha: "11abb0e379b919dcff38cc4aed99e1a8934aab4a55d9000115cf54dc1608f5c9",
        after_sha: "e4fa0c743fbc3fe1efec08f09ee72c38b2faa476e831f0233ea05efea63644a1",
    },
    File {
        path: "crates/vize_canon/src/virtual_ts/helpers/preamble.rs",
        edits: &[(F2_BEFORE_0, F2_AFTER_0)],
        before_bytes: 5052,
        before_sha: "15ad52d5dbc3166ba6fb5d7386515b6929064862ac530795c6a56525b539076e",
        after_sha: "a8ef812b81fda63ef7d2f9104cccffaa5330df2ab5ea4e49360410536f0e4464",
    },
];

fn delta(edits: &[(&str, &str)]) -> isize {
    edits
        .iter()
        .map(|(before, after)| after.len() as isize - before.len() as isize)
        .sum()
}

fn invert(current: &str, file: &File) -> String {
    if current.len() == file.before_bytes && support::hash(current.as_bytes()) == file.before_sha {
        return current.to_owned();
    }
    assert_eq!(
        current.len() as isize,
        file.before_bytes as isize + delta(file.edits)
    );
    assert_eq!(support::hash(current.as_bytes()), file.after_sha);
    let mut previous = current.to_owned();
    for (before, after) in file.edits {
        assert_eq!(previous.matches(after).count(), 1, "{}", file.path);
        assert_eq!(previous.matches(before).count(), 0, "{}", file.path);
        previous = previous.replacen(after, before, 1);
    }
    assert_eq!(previous.len(), file.before_bytes);
    assert_eq!(support::hash(previous.as_bytes()), file.before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::shared_usage_gates::source_before(path, current);
    match FILES.iter().find(|file| file.path == path) {
        Some(file) => invert(&current, file),
        None => current,
    }
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    let files = FILES
        .iter()
        .map(|file| {
            json!({ "path": file.path, "beforeBytes": file.before_bytes,
                "afterBytes": file.before_bytes as isize + delta(file.edits),
                "beforeSha256": file.before_sha, "afterSha256": file.after_sha })
        })
        .collect::<Vec<_>>();
    manifest["provenance"]["currentPropsFallbackSegmentUpdate"] = json!({
        "kind": "exact-current-props-fallback-segment-inverse",
        "issue": 11628,
        "nativeAcceptanceCredit": false,
        "files": files,
        "compiledPreamble": "unchanged; the fallback remains a byte-identical segment"
    });
}

#[cfg(test)]
mod tests {
    use super::{FILES, invert, retain_provenance, source_before};
    use serde_json::json;

    /// The physical image with only the newer template-ref target layer removed.
    fn current(path: &str) -> String {
        let body = std::fs::read_to_string(super::support::repository().join(path)).unwrap();
        super::super::shared_kebab_case::source_before(path, body)
    }

    fn redo(mut previous: String, edits: &[(&str, &str)]) -> String {
        for (before, after) in edits.iter().rev() {
            previous = previous.replacen(before, after, 1);
        }
        previous
    }

    #[test]
    fn current_assets_reconstruct_only_the_props_fallback_segment_layer() {
        for file in &FILES {
            let body = current(file.path);
            assert_eq!(
                redo(source_before(file.path, body.clone()), file.edits),
                body
            );
        }
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt";
        assert_eq!(source_before(other, current(other)), current(other));
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
    }

    #[test]
    fn missing_duplicate_and_extra_bytes_refuse() {
        for file in &FILES {
            let body = current(file.path);
            let (_, after) = file.edits[0];
            for mutated in [
                body.replacen(after, "", 1),
                format!("{body}{after}"),
                format!("{body}\n"),
            ] {
                assert!(std::panic::catch_unwind(|| invert(&mutated, file)).is_err());
            }
            let predecessor = file.edits.iter().fold(body, |text, (before, after)| {
                text.replacen(after, before, 1)
            });
            assert_eq!(invert(&predecessor, file), predecessor);
        }
    }
}
