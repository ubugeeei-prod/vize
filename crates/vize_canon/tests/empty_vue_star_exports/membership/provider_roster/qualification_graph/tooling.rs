use super::{Checked, Value, WORKSPACE, bound, load};

const TOOLING_UPDATE: &str = "malformed-membership/provider-js-tooling-layout-update-12082.json";
const TOOLING_UPDATE_HASH: &str =
    "fdcfb765b733d4ce0c66169c179ec8d78ec9068bd287e12ad3cdf1c54ca2f005";
const PARCEL_UPDATE_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-parcel-native-update-12011.json";
pub(super) const BABEL_MANIFEST: &str = "npm/builder/babel/package.json";
pub(super) const ESLINT_MANIFEST: &str = "npm/linter/eslint/package.json";
pub(super) const OXLINT_MANIFEST: &str = "npm/linter/oxlint/package.json";
pub(super) const PRETTIER_MANIFEST: &str = "npm/formatter/prettier/package.json";

pub(super) fn tooling_update() -> Value {
    load(
        TOOLING_UPDATE,
        TOOLING_UPDATE_HASH,
        "exact-provider-qualification-js-tooling-layout-successor",
    )
}

pub(super) struct ToolingQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
    pub(super) babel_manifest: &'a [u8],
    pub(super) eslint_manifest: &'a [u8],
    pub(super) oxlint_manifest: &'a [u8],
    pub(super) prettier_manifest: &'a [u8],
}

pub(super) fn verify_tooling_update(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &ToolingQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["finalQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            PARCEL_UPDATE_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen JavaScript tooling qualification successor");
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
        (&update["workspaceManifest"], WORKSPACE, graph.workspace),
        (
            &update["babelManifest"],
            BABEL_MANIFEST,
            graph.babel_manifest,
        ),
        (
            &update["eslintManifest"],
            ESLINT_MANIFEST,
            graph.eslint_manifest,
        ),
        (
            &update["oxlintManifest"],
            OXLINT_MANIFEST,
            graph.oxlint_manifest,
        ),
        (
            &update["prettierManifest"],
            PRETTIER_MANIFEST,
            graph.prettier_manifest,
        ),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current JavaScript tooling qualification graph bytes");
        }
    }
    if update["resolver"]["command"]
        != "vp install --lockfile-only --no-frozen-lockfile --ignore-scripts"
        || update["resolver"]["fixedPointSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact JavaScript tooling resolver fixed point");
    }

    let workspace = std::str::from_utf8(graph.workspace).map_err(|_| "UTF-8 workspace manifest")?;
    for member in [
        "  - \"npm/builder/*\"\n",
        "  - \"npm/linter/*\"\n",
        "  - \"npm/formatter/*\"\n",
    ] {
        if workspace.matches(member).count() != 1 {
            return Err("one workspace glob for every JavaScript tooling role");
        }
    }
    let lock = std::str::from_utf8(graph.lock).map_err(|_| "UTF-8 pnpm lockfile")?;
    for importer in [
        "  npm/builder/babel:\n",
        "  npm/linter/eslint:\n",
        "  npm/linter/oxlint:\n",
        "  npm/formatter/prettier:\n",
    ] {
        if lock.matches(importer).count() != 1 {
            return Err("one exact role-specific JavaScript tooling importer");
        }
    }
    for retired in [
        "  npm/babel:\n",
        "  npm/eslint:\n",
        "  npm/oxlint:\n",
        "  npm/prettier:\n",
    ] {
        if lock.contains(retired) {
            return Err("no retired flat JavaScript tooling importer");
        }
    }
    Ok(())
}
