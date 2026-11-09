use super::{Checked, Value, bound, fs, load, support};
use serde_json::json;

pub(super) const UPDATE: &str =
    "malformed-membership/provider-eslint-vue-scorecard-update-12539.json";
pub(super) const UPDATE_HASH: &str =
    "e6959fc8db8571a036a24aaf339d61451da165ef7fc8400549605c7f700fed14";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-js-tooling-refresh-update-12212.json";
pub(super) const MANIFEST: &str = "tools/benchmarks/scripts/package.json";

pub(super) fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-eslint-vue-scorecard-successor",
    )
}

pub(super) struct EslintVueScorecardQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
    pub(super) manifest: &'a [u8],
}

pub(super) fn authorize(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    lock: &[u8],
    package: &[u8],
    workspace: &[u8],
) -> Checked<Value> {
    let update = update();
    let manifest = fs::read(support::repository().join(MANIFEST))
        .map_err(|_| "complete eslint-plugin-vue scorecard manifest")?;
    let graph = EslintVueScorecardQualificationGraph {
        lock,
        package,
        workspace,
        manifest: &manifest,
    };
    verify(predecessor, predecessor_bytes, &update, &graph)?;
    Ok(update)
}

fn unique_offset(haystack: &[u8], needle: &[u8]) -> Checked<usize> {
    if needle.is_empty() {
        return Err("non-empty scorecard lock insertion");
    }
    let mut offsets = haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(offset, candidate)| (candidate == needle).then_some(offset));
    let offset = offsets.next().ok_or("complete scorecard lock insertion")?;
    if offsets.next().is_some() {
        return Err("one scorecard lock insertion");
    }
    Ok(offset)
}

fn reconstruct_predecessor(update: &Value, lock: &[u8]) -> Checked<Vec<u8>> {
    let insertions = update["lockInsertions"]
        .as_array()
        .ok_or("two exact scorecard lock insertions")?;
    if insertions.len() != 2 {
        return Err("two exact scorecard lock insertions");
    }
    let mut removals = insertions
        .iter()
        .map(|insertion| {
            let added = insertion["addedUtf8"]
                .as_str()
                .ok_or("complete scorecard lock insertion")?
                .as_bytes();
            Ok::<_, &'static str>((unique_offset(lock, added)?, added.len()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    removals.sort_unstable_by(|left, right| right.0.cmp(&left.0));

    let mut predecessor = lock.to_vec();
    for (offset, len) in removals {
        predecessor.drain(offset..offset + len);
    }
    Ok(predecessor)
}

pub(super) fn verify(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &EslintVueScorecardQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["previousQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            PREVIOUS_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen scorecard qualification predecessor");
    }
    for (binding, path, body) in [
        (
            &update["finalQualificationLock"],
            "pnpm-lock.yaml",
            graph.lock,
        ),
        (
            &update["finalQualificationPackage"],
            "package.json",
            graph.package,
        ),
        (
            &update["workspaceManifest"],
            "pnpm-workspace.yaml",
            graph.workspace,
        ),
        (&update["scorecardManifest"], MANIFEST, graph.manifest),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current scorecard qualification graph bytes");
        }
    }

    let reconstructed = reconstruct_predecessor(update, graph.lock)?;
    if !bound(
        &predecessor["finalQualificationLock"],
        "pnpm-lock.yaml",
        &reconstructed,
    ) {
        return Err("complete scorecard lock predecessor reconstruction");
    }

    let manifest: Value = serde_json::from_slice(graph.manifest)
        .map_err(|_| "complete eslint-plugin-vue scorecard manifest")?;
    if manifest["name"] != "@vize/bench"
        || manifest["devDependencies"]["eslint-plugin-vue"] != "10.9.2"
        || manifest["devDependencies"]["eslint-plugin-vue-scorecard"]
            != "npm:eslint-plugin-vue@10.12.0"
    {
        return Err("one pinned scorecard beside the frozen parity provider");
    }
    if update["verification"]["command"] != "vp install --frozen-lockfile --ignore-scripts"
        || update["verification"]["lockSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact scorecard frozen-lock verification");
    }
    if update["nativeAcceptanceCredit"] != json!(false) {
        return Err("source provenance only");
    }
    Ok(())
}
