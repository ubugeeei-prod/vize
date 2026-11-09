use super::{Checked, Value, bound, load};
use serde_json::json;

pub(super) const UPDATE: &str =
    "malformed-membership/provider-root-dependency-ownership-update-12123.json";
pub(super) const UPDATE_HASH: &str =
    "41c699ad911cd224fd01e43fec982ef3b705e26d28f9c033f9cbbc26bf123e06";
const PREVIOUS_BINDING: &str = "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/provider-fresco-sfc-update-12130.json";

pub(super) const OWNER_MANIFESTS: &[&str] = &[
    "docs/package.json",
    "editors/package.json",
    "npm/builder/unplugin/package.json",
    "npm/builder/vite/package.json",
    "npm/cli/package.json",
    "npm/framework/nuxt-lint-config/package.json",
    "npm/framework/nuxt/package.json",
    "npm/linter/oxlint/package.json",
    "npm/ui/package.json",
    "tests/package.json",
    "tools/benchmarks/scripts/package.json",
    "tools/package.json",
];

const OWNERSHIP: &[(&str, &[&str])] = &[
    ("docs/package.json", &["yaml"]),
    (
        "editors/package.json",
        &["@types/vscode", "vscode-languageclient"],
    ),
    (
        "npm/builder/unplugin/package.json",
        &["@tsdown/css", "tsdown"],
    ),
    ("npm/builder/vite/package.json", &["oxfmt"]),
    (
        "npm/cli/package.json",
        &["json-schema-to-typescript", "oxfmt"],
    ),
    (
        "npm/framework/nuxt-lint-config/package.json",
        &["@nuxt/eslint", "@nuxt/eslint-config"],
    ),
    ("npm/framework/nuxt/package.json", &["oxlint"]),
    (
        "npm/linter/oxlint/package.json",
        &[
            "@nuxt/eslint",
            "@nuxt/eslint-config",
            "@typescript-eslint/parser",
            "ajv",
            "eslint",
            "eslint-plugin-vue",
            "oxlint",
            "vue-eslint-parser",
        ],
    ),
    ("npm/ui/package.json", &["@babel/core"]),
    (
        "tests/package.json",
        &[
            "@babel/core",
            "@babel/plugin-syntax-typescript",
            "@iarna/toml",
            "@types/vscode",
            "@vue/babel-plugin-jsx",
            "ajv",
            "oxfmt",
            "semver",
            "tsdown",
            "vscode-languageclient",
            "vscode-oniguruma",
            "vscode-textmate",
            "vue-computed-inlay-oracle",
            "yaml",
            "zigpty",
        ],
    ),
    (
        "tools/benchmarks/scripts/package.json",
        &["@iarna/toml", "ajv", "oxfmt"],
    ),
    (
        "tools/package.json",
        &[
            "@iarna/toml",
            "chokidar-braces-oracle",
            "oxlint",
            "semver",
            "yaml",
        ],
    ),
];

pub(super) fn update() -> Value {
    load(
        UPDATE,
        UPDATE_HASH,
        "exact-provider-qualification-root-dependency-ownership-successor",
    )
}

pub(super) struct OwnershipQualificationGraph<'a> {
    pub(super) lock: &'a [u8],
    pub(super) package: &'a [u8],
    pub(super) workspace: &'a [u8],
    pub(super) owners: &'a [(&'a str, &'a [u8])],
}

pub(super) fn verify(
    predecessor: &Value,
    predecessor_bytes: &[u8],
    update: &Value,
    graph: &OwnershipQualificationGraph<'_>,
) -> Checked<()> {
    if update["previousQualificationLock"] != predecessor["finalQualificationLock"]
        || update["previousQualificationPackage"] != predecessor["finalQualificationPackage"]
        || !bound(
            &update["previousProviderQualificationRecord"],
            PREVIOUS_BINDING,
            predecessor_bytes,
        )
    {
        return Err("exact frozen root dependency ownership predecessor");
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
    ] {
        if !bound(binding, path, body) {
            return Err("exact current root dependency qualification graph bytes");
        }
    }
    let bindings = update["ownerManifests"]
        .as_array()
        .ok_or("complete owner manifest bindings")?;
    if bindings.len() != graph.owners.len() || graph.owners.len() != OWNER_MANIFESTS.len() {
        return Err("complete owner manifest bindings");
    }
    for (((path, body), expected_path), binding) in
        graph.owners.iter().zip(OWNER_MANIFESTS).zip(bindings)
    {
        if path != expected_path || !bound(binding, path, body) {
            return Err("exact current owner manifest bytes");
        }
    }
    if update["resolver"]["command"]
        != "vp install --lockfile-only --no-frozen-lockfile --ignore-scripts"
        || update["resolver"]["fixedPointSha256"] != update["finalQualificationLock"]["sha256"]
    {
        return Err("exact root dependency resolver fixed point");
    }

    let package: Value =
        serde_json::from_slice(graph.package).map_err(|_| "complete root manifest")?;
    let dependencies = package["devDependencies"]
        .as_object()
        .ok_or("six root execution dependencies")?;
    let mut names = dependencies.keys().map(String::as_str).collect::<Vec<_>>();
    names.sort_unstable();
    if names
        != [
            "@types/node",
            "oxfmt",
            "oxlint",
            "pkg-pr-new",
            "vite",
            "vite-plus",
        ]
    {
        return Err("six root execution dependencies");
    }
    for ((path, expected), (actual_path, bytes)) in OWNERSHIP.iter().zip(graph.owners) {
        if path != actual_path {
            return Err("ordered dependency owners");
        }
        let manifest: Value =
            serde_json::from_slice(bytes).map_err(|_| "complete owner manifest")?;
        let dependencies = manifest["devDependencies"]
            .as_object()
            .ok_or("owner development dependencies")?;
        if expected
            .iter()
            .any(|name| !dependencies.contains_key(*name))
        {
            return Err("declared dependency owner");
        }
    }
    let workspace = std::str::from_utf8(graph.workspace).map_err(|_| "UTF-8 workspace manifest")?;
    if workspace.matches("  - \"editors\"\n").count() != 1
        || workspace.matches("  - \"tools\"\n").count() != 1
    {
        return Err("one shared tooling workspace owner");
    }
    let lock = std::str::from_utf8(graph.lock).map_err(|_| "UTF-8 qualification lock")?;
    for importer in [
        "  docs:\n",
        "  editors:\n",
        "  npm/linter/oxlint:\n",
        "  tests:\n",
        "  tools:\n",
        "  tools/benchmarks/scripts:\n",
    ] {
        if lock.matches(importer).count() != 1 {
            return Err("one exact dependency owner importer");
        }
    }
    if update["nativeAcceptanceCredit"] != json!(false) {
        return Err("source provenance only");
    }
    Ok(())
}
