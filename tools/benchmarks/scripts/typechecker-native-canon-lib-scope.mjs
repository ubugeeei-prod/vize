import assert from "node:assert/strict";
import path from "node:path";
import TOML from "@iarna/toml";
import {
  normalizeAttribute,
  physicalSource,
  registrations,
  rustItems,
} from "./typechecker-native-canon-lib-source.mjs";
import { qualifyDeriveProvider } from "./typechecker-native-canon-lib-provider.mjs";
import { cargoConfigurations } from "./typechecker-native-canon-lib-config.mjs";
import { validateScopeAuthority } from "./typechecker-native-canon-lib-syntax.mjs";

export const canonLibFilters = [
  "original_outside_import_preserves_types_in_native_editor",
  "native_editor_resolves_package_alias_from_monorepo_root",
];
const canon = "crates/vize_canon";

export function canonSource(root) {
  const source = physicalSource(root);
  return {
    ...source,
    cargoContext: cargoConfigurations(root),
    paths: new Set([...source.files(canon), ...source.files(".cargo"), "Cargo.toml", "Cargo.lock"]),
  };
}

function declarationPath(target, fallback) {
  const relative = target.path ?? fallback;
  assert.ok(
    typeof relative === "string" && !path.posix.isAbsolute(relative),
    "opaque Cargo target path",
  );
  const file = path.posix.normalize(`${canon}/${relative}`);
  assert.ok(file.startsWith(`${canon}/`) && file.endsWith(".rs"), "external Cargo target");
  return file;
}

export function outsideLibTargets(manifest, paths) {
  assert.equal(manifest.package.name, "vize_canon", "wrong Canon package");
  assert.equal(manifest.package.build, undefined, "opaque Cargo build script");
  assert.ok(!paths.has(`${canon}/build.rs`), "implicit Cargo build script");
  assert.equal(manifest.lib, undefined, "unreviewed library target configuration");
  assert.deepEqual(manifest.features.default, ["native"], "inactive original native library cases");
  const targets = new Map();
  const add = (kind, name, file, declaration = {}) => {
    assert.ok(
      typeof name === "string" && /^[A-Za-z_][A-Za-z0-9_-]*$/u.test(name),
      "opaque Cargo target name",
    );
    assert.ok(
      declaration.harness !== false && declaration.test !== false,
      "custom or disabled test harness",
    );
    assert.ok(paths.has(file), `missing Cargo target: ${file}`);
    targets.set(`${kind}:${name}`, {
      kind,
      name,
      file,
      requiredFeatures: declaration["required-features"] ?? [],
    });
  };
  for (const [directory, kind, automatic] of [
    ["tests", "test", "autotests"],
    ["src/bin", "bin", "autobins"],
  ]) {
    assert.ok(
      manifest.package[automatic] === undefined || typeof manifest.package[automatic] === "boolean",
      "opaque Cargo discovery flag",
    );
    if (manifest.package[automatic] === false) continue;
    const escaped = `${canon}/${directory}/`;
    for (const file of paths) {
      if (!file.startsWith(escaped)) continue;
      const tail = file.slice(escaped.length);
      const match = tail.match(/^([^/]+)\.rs$/u) ?? tail.match(/^([^/]+)\/main\.rs$/u);
      if (match) add(kind, match[1], file);
    }
  }
  if (manifest.package.autobins !== false && paths.has(`${canon}/src/main.rs`))
    add("bin", "vize_canon", `${canon}/src/main.rs`);
  for (const kind of ["test", "bin", "example", "bench"]) {
    for (const target of manifest[kind] ?? []) {
      assert.ok(
        Object.keys(target).every((key) =>
          ["name", "path", "required-features", "test", "harness"].includes(key),
        ),
        "unreviewed Cargo target setting",
      );
      assert.ok(
        Array.isArray(target["required-features"] ?? []) &&
          (target["required-features"] ?? []).every((feature) =>
            Object.hasOwn(manifest.features, feature),
          ),
        "unknown Cargo target feature",
      );
      // TESTNAME requests test targets. Untested examples/benches have no
      // libtest registrations in either original command; keep other lanes.
      if ((kind === "example" || kind === "bench") && target.test !== true) continue;
      const directory = kind === "test" ? "tests" : kind === "bin" ? "src/bin" : `${kind}s`;
      const fallback = `${directory}/${target.name}.rs`;
      const file =
        target.path === undefined && targets.has(`${kind}:${target.name}`)
          ? targets.get(`${kind}:${target.name}`).file
          : declarationPath(target, fallback);
      add(kind, target.name, file, target);
    }
  }
  return [...targets.values()].sort((left, right) => left.file.localeCompare(right.file));
}

function activeLibraryCase(source, rootItems, bridgeItems, leaf, name, unix) {
  const module = rootItems.filter((item) => item.kind === "mod" && item.name === "corsa_bridge");
  assert.equal(module.length, 1, "missing original native library module");
  assert.deepEqual(
    module[0].attrs.map(normalizeAttribute),
    ['#[cfg(feature="native")]'],
    "inactive native library module",
  );
  assert.equal(module[0].children, null, "redirected native library module");
  const declaration = bridgeItems.filter((item) => item.kind === "mod" && item.name === leaf);
  assert.equal(declaration.length, 1, "missing original library test module");
  assert.deepEqual(
    declaration[0].attrs.map(normalizeAttribute),
    unix ? ["#[cfg(test)]", "#[cfg(unix)]"] : ["#[cfg(test)]"],
    "inactive or redirected original test module",
  );
  assert.equal(declaration[0].children, null, "redirected original test module");
  const text = source.read(`${canon}/src/corsa_bridge/${leaf}.rs`);
  assert.ok(
    !rustItems(text, { library: true }).some((item) =>
      item.attrs.some((attr) => /^#\s*!/u.test(attr)),
    ),
    "inactive original library file",
  );
  const members = rustItems(text, { library: true });
  const original = members.filter((item) => item.kind === "fn" && item.name === name);
  assert.equal(original.length, 1, "missing original library case");
  assert.deepEqual(
    original[0].attrs.map(normalizeAttribute),
    ["#[test]"],
    "inactive original library case",
  );
  validateScopeAuthority(members, { library: true });
  return `corsa_bridge::${leaf}::${name}`;
}

export function qualifyCanonLibScope(source) {
  const root = TOML.parse(source.read("Cargo.toml"));
  const manifest = TOML.parse(source.read(`${canon}/Cargo.toml`));
  const provider = qualifyDeriveProvider(source, root, manifest);
  const targets = outsideLibTargets(manifest, source.paths);
  // Union feature/platform branches, including presently feature-gated targets.
  // This does not borrow binaries or cfg values from the ci workspace profile.
  const inventory = registrations(
    source,
    targets.map((target) => target.file),
    new Set(Object.keys(manifest.features)),
  );
  for (const entry of inventory.cases) {
    assert.ok(
      !canonLibFilters.some((filter) => entry.name.includes(filter)),
      `outside-library selected case: ${entry.name}`,
    );
  }
  const rootItems = rustItems(source.read(`${canon}/src/lib.rs`), { library: true });
  const bridgeItems = rustItems(source.read(`${canon}/src/corsa_bridge.rs`), { library: true });
  validateScopeAuthority(rootItems, { library: true });
  validateScopeAuthority(bridgeItems, { library: true });
  assert.ok(
    ![...rootItems, ...bridgeItems].some((item) => item.attrs.some((attr) => /^#\s*!/u.test(attr))),
    "inactive native library file",
  );
  const originalCases = [
    activeLibraryCase(
      source,
      rootItems,
      bridgeItems,
      "outside_import_editor_tests",
      canonLibFilters[0],
      false,
    ),
    activeLibraryCase(
      source,
      rootItems,
      bridgeItems,
      "vue_document_package_config_tests",
      canonLibFilters[1],
      true,
    ),
  ];
  return { targets, ...inventory, provider, originalCases };
}

export function qualifyCanonLibCommands(workflow) {
  for (const filter of canonLibFilters) {
    const clauses = workflow.split(/\n|;\s*/u).filter((clause) => clause.includes(filter));
    assert.equal(clauses.length, 1, "ambiguous original native filter command");
    const expected = `cargo test --locked --profile ci-opt -p vize_canon --lib ${filter} --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' -- --nocapture`;
    assert.equal(clauses[0].trim(), expected, "changed native filter/profile/target command");
  }
}
