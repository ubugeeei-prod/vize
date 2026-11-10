//! Reverse the shared listener relation before authenticating older SDK history.
use super::support;
use serde_json::{Value, json};

const PREAMBLE_SOURCE: &str = "crates/vize_canon/src/virtual_ts/helpers/preamble.rs";
const GENERATOR: &str = "npm/cli/scripts/generate-content-mapper-types.ts";
const MANIFEST: &str = "npm/cli/package.json";
const PUBLIC_PATHS: [&str; 3] = [
    "npm/cli/virtual-ts.d.ts",
    "npm/cli/virtual-ts.d.mts",
    "npm/cli/virtual-ts.d.cts",
];

const PREAMBLE_INCLUDE: &str =
    "    include_str!(\"declarations/shared-listener-args.d.ts.txt\"),\n";
const GENERATOR_IMPORT: &str =
    "import { listenerArgumentExports } from \"./listener-argument-exports.ts\";\n";
const GENERATOR_CALL: &str = "for (const [file, contents] of await listenerArgumentExports(root, outputs))\n  outputs.set(file, contents);\n";
const MANIFEST_ENTRY: &str = "    \"virtual-ts-listeners.d.ts\",\n";
const PUBLIC_EXPORT: &str = r#"
import type * as __vizeListenerTypes from "./virtual-ts-listeners.js" with {
  "resolution-mode": "import",
};
export type $ListenerArgs<
  __P,
  __K extends string | number | symbol,
  __Raw extends string | number | symbol = never,
> = __vizeListenerTypes.$ListenerArgs<__P, __K, __Raw>;
"#;

const SHARED_LISTENER_CURRENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/virtual_ts/helpers/declarations/shared-listener-args.d.ts.txt"
));

/// The asset as this layer recorded it, before the shared listener contract.
fn shared_listener() -> String {
    super::listener_contract::source_before(
        "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-listener-args.d.ts.txt",
        SHARED_LISTENER_CURRENT.to_owned(),
    )
}

struct Transition {
    path: &'static str,
    after_bytes: usize,
    after_sha: &'static str,
    before_bytes: usize,
    before_sha: &'static str,
    additions: &'static [&'static str],
}

const TRANSITIONS: [Transition; 6] = [
    Transition {
        path: PREAMBLE_SOURCE,
        after_bytes: 5_052,
        after_sha: "15ad52d5dbc3166ba6fb5d7386515b6929064862ac530795c6a56525b539076e",
        before_bytes: 4_988,
        before_sha: "38c8e437524c3983a57d089febbf49ac4898ddfc1318672ded78608728c0dde6",
        additions: &[PREAMBLE_INCLUDE],
    },
    Transition {
        path: GENERATOR,
        after_bytes: 18_736,
        after_sha: "ffabac531fe547766e1e5913292b048c5478b831ad2bec8f84713b060c8987f5",
        before_bytes: 18_554,
        before_sha: "9df0e4ea2c0f466c3f717192102e3b6f14350540987d9dc3bc573111823a13c4",
        additions: &[GENERATOR_IMPORT, GENERATOR_CALL],
    },
    Transition {
        path: MANIFEST,
        after_bytes: 4_273,
        after_sha: "2e430bc9bbb14cbd07ae076459bf0a932d1558bb9a698b67527b4086d8988e6f",
        before_bytes: 4_240,
        before_sha: "cb84f2c91b84e60fbec19e3745e4fad6b8e0a35f821f54ea9b9d6e5d8e3478d8",
        additions: &[MANIFEST_ENTRY],
    },
    Transition {
        path: PUBLIC_PATHS[0],
        after_bytes: 15_357,
        after_sha: "d2afee78f759b85aa1b227caf4bd0b6f9fef54671e0c6e5f7c89f40c9b12cd6e",
        before_bytes: 15_065,
        before_sha: "417b4ab1f380eaffeb62676def8762ed78fba126e606972f262503b8cda04432",
        additions: &[PUBLIC_EXPORT],
    },
    Transition {
        path: PUBLIC_PATHS[1],
        after_bytes: 15_357,
        after_sha: "d2afee78f759b85aa1b227caf4bd0b6f9fef54671e0c6e5f7c89f40c9b12cd6e",
        before_bytes: 15_065,
        before_sha: "417b4ab1f380eaffeb62676def8762ed78fba126e606972f262503b8cda04432",
        additions: &[PUBLIC_EXPORT],
    },
    Transition {
        path: PUBLIC_PATHS[2],
        after_bytes: 15_357,
        after_sha: "d2afee78f759b85aa1b227caf4bd0b6f9fef54671e0c6e5f7c89f40c9b12cd6e",
        before_bytes: 15_065,
        before_sha: "417b4ab1f380eaffeb62676def8762ed78fba126e606972f262503b8cda04432",
        additions: &[PUBLIC_EXPORT],
    },
];

fn authenticate(text: &str, bytes: usize, sha: &str) {
    assert_eq!(text.len(), bytes);
    assert_eq!(support::hash(text.as_bytes()), sha);
}

fn inverse(current: String, transition: &Transition) -> String {
    authenticate(&current, transition.after_bytes, transition.after_sha);
    let mut previous = current;
    for addition in transition.additions {
        assert!(!addition.is_empty());
        assert_eq!(previous.matches(addition).count(), 1);
        previous = previous.replacen(addition, "", 1);
    }
    authenticate(&previous, transition.before_bytes, transition.before_sha);
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::canon_leftovers::source_before(path, current);
    let current = super::dependency_ownership::source_before(path, current);
    let current = super::strict_extra_keys::source_before(path, current);
    TRANSITIONS
        .iter()
        .find(|transition| transition.path == path)
        .map_or(current.clone(), |transition| inverse(current, transition))
}

pub(super) fn preamble_before(current: &str) -> String {
    let current = &super::canon_leftovers::preamble_before(current);
    let current = &super::strict_extra_keys::preamble_before(current);
    authenticate(
        current,
        47_268,
        "be971e511d48ce689262c5c6f422ff2685085bf166a0c93f9c0a9c93b9848ba3",
    );
    let shared = shared_listener();
    assert_eq!(current.matches(shared.as_str()).count(), 1);
    let previous = current.replacen(shared.as_str(), "", 1);
    authenticate(
        &previous,
        46_557,
        "635acca0b25877f2132902fe777d894287100a7c993066066a67812deab2f255",
    );
    previous
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    let new_files = [
        (
            "crates/vize_canon/src/virtual_ts/helpers/declarations/listener-args.d.ts.txt",
            663,
            "15cf42250c44b6c090d9c33387c707b5495bb080860fab4cf013e7c180521146",
        ),
        (
            "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-listener-args.d.ts.txt",
            711,
            "fef5c40105887cdd343341d34f38343a3522d2fd5526a73107ee6de9ebf90ff5",
        ),
        (
            "npm/cli/scripts/listener-argument-exports.ts",
            4_660,
            "60cccf633b77b2c8346a1badd14cb07ea44060c6650359b2bdf867d0b147c810",
        ),
        (
            "npm/cli/virtual-ts-listeners.d.ts",
            806,
            "643048acce2ca1f480f541b70f25fa52baa2ed452f3860cc2bd84082fb2b81ed",
        ),
    ];
    for (path, bytes, sha) in new_files {
        let body = super::listener_contract::read_current(path);
        authenticate(&body, bytes, sha);
    }
    manifest["provenance"]["currentListenerRelationUpdate"] = json!({
        "kind": "exact-listener-relation-source-inverse",
        "pullRequest": 11994,
        "nativeAcceptanceCredit": false,
        "files": TRANSITIONS.iter().map(|transition| json!({
            "path": transition.path,
            "beforeBytes": transition.before_bytes,
            "beforeSha256": transition.before_sha,
            "afterBytes": transition.after_bytes,
            "afterSha256": transition.after_sha,
        })).collect::<Vec<_>>(),
        "compiledPreamble": {
            "beforeBytes": 46557,
            "beforeSha256": "635acca0b25877f2132902fe777d894287100a7c993066066a67812deab2f255",
            "afterBytes": 47268,
            "afterSha256": "be971e511d48ce689262c5c6f422ff2685085bf166a0c93f9c0a9c93b9848ba3",
        },
        "newFiles": new_files.iter().map(|(path, bytes, sha)| json!({
            "path": path,
            "bytes": bytes,
            "sha256": sha,
        })).collect::<Vec<_>>(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sources_and_preamble_reconstruct_the_exact_predecessor() {
        for transition in &TRANSITIONS {
            let current = super::super::listener_contract::read_current(transition.path);
            authenticate(
                &source_before(transition.path, current),
                transition.before_bytes,
                transition.before_sha,
            );
        }
        let previous = preamble_before(&super::super::listener_contract::preamble_before(
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
        ));
        authenticate(
            &previous,
            46_557,
            "635acca0b25877f2132902fe777d894287100a7c993066066a67812deab2f255",
        );
    }

    #[test]
    fn malformed_or_duplicate_successors_are_refused() {
        for transition in &TRANSITIONS {
            let current = super::super::listener_contract::read_current(transition.path);
            for changed in [format!("{current}\n"), format!("{current}{current}")] {
                assert!(
                    std::panic::catch_unwind(|| source_before(transition.path, changed)).is_err()
                );
            }
        }
        let current = &super::super::listener_contract::preamble_before(
            vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
        );
        for changed in [
            current.replacen(shared_listener().as_str(), "", 1),
            format!("{current}{}", shared_listener()),
        ] {
            assert!(std::panic::catch_unwind(|| preamble_before(&changed)).is_err());
        }
    }

    #[test]
    fn provenance_authenticates_every_new_source() {
        let mut manifest = json!({ "provenance": {} });
        retain_provenance(&mut manifest);
        assert_eq!(
            manifest["provenance"]["currentListenerRelationUpdate"]["newFiles"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
    }
}
