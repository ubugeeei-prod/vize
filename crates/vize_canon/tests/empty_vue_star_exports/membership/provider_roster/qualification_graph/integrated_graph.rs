use super::{Checked, Value, bound, fs, load, support};
use serde_json::json;

const UPDATE: &str = "malformed-membership/provider-integrated-graph-update-12318.json";
const UPDATE_HASH: &str = "31f19514feb3e7f883bf8a007295c572c97357d8a527ca271aa73d89e7f282f8";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-strict-checker-update-11628.json";
const CREATE_RECORD: &str = "malformed-membership/provider-create-fresco-update-12233.json";
const CREATE_RECORD_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-create-fresco-update-12233.json";
const DOCS_RECORD: &str = "malformed-membership/provider-docs-metadata-update-12236.json";
const DOCS_RECORD_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-docs-metadata-update-12236.json";
const DEPENDENCY_RECORD: &str = "malformed-membership/provider-dependency-update-12212.json";
const DEPENDENCY_RECORD_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-dependency-update-12212.json";
const CREATE_MANIFEST: &str = "npm/create-fresco/package.json";
const DOCS_MANIFEST: &str = "docs/package.json";

fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-integrated-graph-successor",
    )
}

pub(super) fn authorize_after_strict(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    lock: &[u8],
    package: &[u8],
    workspace: &[u8],
) -> Checked<Value> {
    if let Ok(update) =
        super::docs_metadata::authorize(predecessor, predecessor_bytes, lock, package, workspace)
    {
        return Ok(update);
    }
    authorize(predecessor, predecessor_bytes, lock, package, workspace)
}

pub(super) fn authorize(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    lock: &[u8],
    package: &[u8],
    workspace: &[u8],
) -> Checked<Value> {
    let update = update();
    let frozen_predecessor: Value = serde_json::from_slice(predecessor_bytes)
        .map_err(|_| "complete integrated qualification predecessor")?;
    if predecessor != &frozen_predecessor
        || !bound(
            &update["previousProviderQualificationRecord"],
            PREVIOUS_BINDING,
            predecessor_bytes,
        )
        || predecessor["kind"] != "exact-provider-qualification-strict-checker-successor"
    {
        return Err("exact frozen integrated qualification predecessor");
    }
    for (binding, path, source) in [
        (
            "createFrescoQualificationRecord",
            CREATE_RECORD_BINDING,
            CREATE_RECORD,
        ),
        (
            "docsMetadataQualificationRecord",
            DOCS_RECORD_BINDING,
            DOCS_RECORD,
        ),
        (
            "providerDependencyRecord",
            DEPENDENCY_RECORD_BINDING,
            DEPENDENCY_RECORD,
        ),
    ] {
        let bytes = fs::read(support::corpus().join(source))
            .map_err(|_| "complete integrated qualification history")?;
        if !bound(&update[binding], path, &bytes) {
            return Err("exact integrated qualification history");
        }
    }
    let create = fs::read(support::repository().join(CREATE_MANIFEST))
        .map_err(|_| "complete create-fresco manifest")?;
    let docs = fs::read(support::repository().join(DOCS_MANIFEST))
        .map_err(|_| "complete docs manifest")?;
    for (binding, path, body) in [
        (&update["finalQualificationLock"], "pnpm-lock.yaml", lock),
        (
            &update["finalQualificationPackage"],
            "package.json",
            package,
        ),
        (
            &update["workspaceManifest"],
            "pnpm-workspace.yaml",
            workspace,
        ),
        (&update["createFrescoManifest"], CREATE_MANIFEST, &create),
        (&update["docsManifest"], DOCS_MANIFEST, &docs),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current integrated qualification graph bytes");
        }
    }
    if update["resolver"]["command"] != "vp install --frozen-lockfile"
        || update["resolver"]["fixedPointSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact frozen integrated resolver input");
    }

    let package: Value =
        serde_json::from_slice(package).map_err(|_| "complete root package manifest")?;
    if package["packageManager"] != "pnpm@12.1.0"
        || package["devDependencies"]["vite"] != "catalog:vite-stack"
        || package["devDependencies"]["vite-plus"] != "catalog:vite-stack"
    {
        return Err("one current root Vite qualification owner");
    }
    let create: Value =
        serde_json::from_slice(&create).map_err(|_| "complete create-fresco manifest")?;
    let dependencies = create["devDependencies"]
        .as_object()
        .ok_or("four create-fresco development dependencies")?;
    if create["name"] != "@vizejs/create-fresco"
        || create["bin"]["create-fresco"] != "./dist/cli.mjs"
        || dependencies.len() != 4
        || dependencies.get("vite-plus") != Some(&json!("catalog:vite-stack"))
    {
        return Err("one current create-fresco package contract");
    }
    let docs: Value = serde_json::from_slice(&docs).map_err(|_| "complete docs manifest")?;
    for script in ["build", "dev"] {
        if docs["scripts"][script]
            .as_str()
            .is_none_or(|value| value.matches("pnpm generate:metadata").count() != 1)
        {
            return Err("one current docs metadata contract");
        }
    }
    let workspace = std::str::from_utf8(workspace).map_err(|_| "UTF-8 workspace manifest")?;
    let lock = std::str::from_utf8(lock).map_err(|_| "UTF-8 qualification lock")?;
    if workspace.matches("  - \"npm/*\"\n").count() != 1
        || lock
            .matches("specifier: npm:@voidzero-dev/vite-plus-core@0.1.24")
            .count()
            != 1
        || !lock.contains("@tsdown/css@0.22.14")
    {
        return Err("one current integrated Vite provider graph");
    }
    if update["nativeAcceptanceCredit"] != json!(false) {
        return Err("source provenance only");
    }
    Ok(update)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_integrated_successor_binds_the_current_whole_graph() {
        let predecessor_bytes = fs::read(
            support::corpus()
                .join("malformed-membership/provider-strict-checker-update-11628.json"),
        )
        .unwrap();
        let predecessor: Value = serde_json::from_slice(&predecessor_bytes).unwrap();
        let lock = fs::read(support::repository().join("pnpm-lock.yaml")).unwrap();
        let package = fs::read(support::repository().join("package.json")).unwrap();
        let workspace = fs::read(support::repository().join("pnpm-workspace.yaml")).unwrap();
        authorize(
            &predecessor,
            &predecessor_bytes,
            &lock,
            &package,
            &workspace,
        )
        .unwrap();
    }
}
