use super::{
    Checked, Value, bound,
    create_fresco::{
        self, CreateFrescoQualificationGraph, MANIFEST as CREATE_FRESCO_MANIFEST,
        update as create_fresco_update, verify as verify_create_fresco_update,
    },
    fs, load, support,
};
use serde_json::json;

pub(super) const UPDATE: &str = "malformed-membership/provider-docs-metadata-update-12236.json";
pub(super) const UPDATE_HASH: &str =
    "42fc71d00781431925ce8f76325a225f6caf9bcfb3f47caaaf28653639e0644f";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-create-fresco-update-12233.json";
pub(super) const MANIFEST: &str = "docs/package.json";

pub(super) fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-docs-metadata-successor",
    )
}

pub(super) fn authorize(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    lock: &[u8],
    package: &[u8],
    workspace: &[u8],
) -> Checked<Value> {
    let successor = create_fresco_update();
    let manifest = fs::read(support::repository().join(CREATE_FRESCO_MANIFEST))
        .map_err(|_| "complete create-fresco manifest")?;
    let graph = CreateFrescoQualificationGraph {
        lock,
        package,
        workspace,
        manifest: &manifest,
    };
    verify_create_fresco_update(predecessor, predecessor_bytes, &successor, &graph)?;

    let predecessor_bytes = fs::read(support::corpus().join(create_fresco::UPDATE))
        .map_err(|_| "complete docs metadata qualification predecessor")?;
    let update = update();
    let manifest =
        fs::read(support::repository().join(MANIFEST)).map_err(|_| "complete docs manifest")?;
    let graph = DocsMetadataQualificationGraph {
        lock,
        package,
        workspace,
        manifest: &manifest,
    };
    verify(&successor, &predecessor_bytes, &update, &graph)?;
    Ok(update)
}

pub(super) struct DocsMetadataQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
    pub(super) manifest: &'a [u8],
}

pub(super) fn verify(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &DocsMetadataQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["previousQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            PREVIOUS_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen docs metadata qualification predecessor");
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
        (&update["docsManifest"], MANIFEST, graph.manifest),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current docs metadata qualification graph bytes");
        }
    }
    if update["resolver"]["command"]
        != "vp install --lockfile-only --no-frozen-lockfile --ignore-scripts"
        || update["resolver"]["fixedPointSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact docs metadata resolver fixed point");
    }

    let manifest: Value =
        serde_json::from_slice(graph.manifest).map_err(|_| "complete docs manifest")?;
    let scripts = manifest["scripts"]
        .as_object()
        .ok_or("complete docs scripts")?;
    if manifest["name"] != "vize-docs"
        || scripts.get("generate:metadata")
            != Some(&json!("node ./scripts/generate-contract-metadata.ts"))
        || scripts
            .get("build")
            .and_then(Value::as_str)
            .is_none_or(|script| script.matches("pnpm generate:metadata").count() != 1)
        || scripts
            .get("dev")
            .and_then(Value::as_str)
            .is_none_or(|script| script.matches("pnpm generate:metadata").count() != 1)
    {
        return Err("one executable docs metadata contract");
    }
    if update["nativeAcceptanceCredit"] != json!(false) {
        return Err("source provenance only");
    }
    Ok(())
}
