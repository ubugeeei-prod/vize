use super::{Checked, Value, bound, load};

const FRESCO_UPDATE: &str = "malformed-membership/provider-fresco-sfc-update-12130.json";
const FRESCO_UPDATE_HASH: &str = "53fffb6c6fc9c03809fa184ba2f351af89c64849851fbfac3ad951bd83316792";
pub(super) const TOOLING_UPDATE: &str =
    "malformed-membership/provider-js-tooling-layout-update-12082.json";
pub(super) const TOOLING_UPDATE_HASH: &str =
    "fdcfb765b733d4ce0c66169c179ec8d78ec9068bd287e12ad3cdf1c54ca2f005";
const TOOLING_UPDATE_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-js-tooling-layout-update-12082.json";
pub(super) const FRESCO_MANIFEST: &str = "npm/fresco/package.json";

pub(super) fn fresco_update() -> Value {
    load(
        FRESCO_UPDATE,
        FRESCO_UPDATE_HASH,
        "exact-provider-qualification-fresco-sfc-successor",
    )
}

pub(super) struct FrescoQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) fresco_manifest: &'a [u8],
}

pub(super) fn verify_fresco_update(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &FrescoQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["finalQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            TOOLING_UPDATE_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen Fresco qualification successor");
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
            &update["frescoManifest"],
            FRESCO_MANIFEST,
            graph.fresco_manifest,
        ),
    ] {
        if !bound(binding, path, body) {
            return Err("exact current Fresco qualification graph bytes");
        }
    }
    if update["resolver"]["command"]
        != "vp install --lockfile-only --no-frozen-lockfile --ignore-scripts"
        || update["resolver"]["fixedPointSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact Fresco resolver fixed point");
    }

    let manifest: Value =
        serde_json::from_slice(graph.fresco_manifest).map_err(|_| "complete Fresco manifest")?;
    if manifest["name"] != "@vizejs/fresco"
        || manifest["dependencies"]["@vizejs/fresco-native"] != "workspace:*"
        || manifest["devDependencies"]["@vitejs/plugin-vue"] != "catalog:vite-stack"
        || manifest["devDependencies"]["@vue/compiler-sfc"] != "catalog:vue-stable"
        || manifest["devDependencies"]["happy-dom"] != "catalog:testing"
        || manifest["devDependencies"]["vitest"] != "catalog:vite-stack"
        || manifest["devDependencies"]["vue-tsc"] != "catalog:typescript"
        || manifest["scripts"]["test"] != "vp test run"
    {
        return Err("one declared Fresco SFC test toolchain");
    }
    let lock = std::str::from_utf8(graph.lock).map_err(|_| "UTF-8 pnpm lockfile")?;
    if lock.matches("\n  npm/fresco:\n").count() != 1 {
        return Err("one exact Fresco importer");
    }
    Ok(())
}
