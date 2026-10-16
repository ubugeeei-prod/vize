import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertChangedDependenciesBuilt,
  assertChangedPackageArtifactsBuilt,
  changedDependencyPackages,
  changedSourceDependencies,
} from "../performance/support/warm-type-backed-dependency-custody.ts";

test("qualified config and lint changes rebuild their actual dependencies on both sides", () => {
  const dependencies = changedSourceDependencies([
    "davinci/vize_l0/src/config/model/linter_rule_options.rs",
    "crates/vize_patina/src/linter/restricted_rules.rs",
    "crates/vize_maestro/src/ide/diagnostics/linter_options.rs",
  ]);
  assert.deepEqual(dependencies, ["vize_l0", "vize_patina"]);
  assert.deepEqual(changedSourceDependencies(["tests/tooling/receipt.test.ts"]), []);
  assert.deepEqual(changedSourceDependencies(["davinci/vize_l00/src/lib.rs"]), []);
  const artifacts = dependencies.map((name) => ({ target: { name }, fresh: false }));
  assert.doesNotThrow(() => assertChangedDependenciesBuilt(dependencies, artifacts));
  for (const side of ["before", "after"])
    for (const name of dependencies) {
      assert.throws(
        () =>
          assertChangedDependenciesBuilt(
            dependencies,
            artifacts.map((artifact) => ({
              ...artifact,
              fresh: artifact.target.name === name,
            })),
          ),
        /must actually compile/,
        `${side} cannot reuse a previous source's dependency`,
      );
      assert.throws(
        () =>
          assertChangedDependenciesBuilt(
            dependencies,
            artifacts.filter((artifact) => artifact.target.name !== name),
          ),
        /must supply an actual linked artifact/,
      );
    }
});

test("six owned Corsa packages rebuild both registry and path identities with preserved lib names", () => {
  const names = [
    "corsa",
    "corsa_core",
    "corsa_jsonrpc",
    "corsa_client",
    "corsa_lsp",
    "corsa_orchestrator",
  ];
  const dependencies = changedSourceDependencies(names.map((name) => `vendor/${name}/src/lib.rs`));
  assert.deepEqual(dependencies, names);
  assert.deepEqual(
    changedSourceDependencies([
      "vendor/corsa_runtime/src/lib.rs",
      "vendor/corsa_core_other/src/lib.rs",
    ]),
    [],
  );
  for (const owned of [false, true]) {
    const metadata = {
      packages: names.map((name) => ({
        id: owned
          ? `path+file:///source/vendor/${name}#vize_${name}@0.457.0`
          : `registry+https://github.com/rust-lang/crates.io-index#${name}@1.14.2`,
        name: owned ? `vize_${name}` : name,
        version: owned ? "0.457.0" : "1.14.2",
        source: owned ? null : "registry+https://github.com/rust-lang/crates.io-index",
        manifest_path: owned ? `/source/vendor/${name}/Cargo.toml` : `/registry/${name}/Cargo.toml`,
        targets: [
          {
            name,
            kind: ["lib"],
            src_path: owned ? `/source/vendor/${name}/src/lib.rs` : `/registry/${name}/src/lib.rs`,
          },
        ],
      })),
    };
    const packages = changedDependencyPackages(dependencies, metadata, "/source");
    assert.deepEqual(
      packages.map((entry) => entry.name),
      names.map((name) => (owned ? `vize_${name}` : name)),
    );
    const artifacts = packages.map((entry) => ({
      package_id: entry.id,
      manifest_path: entry.manifest_path,
      target: entry.targets[0],
      fresh: false,
    }));
    assert.doesNotThrow(() => assertChangedPackageArtifactsBuilt(packages, artifacts));
    for (let index = 0; index < names.length; index++) {
      assert.throws(
        () =>
          assertChangedPackageArtifactsBuilt(
            packages,
            artifacts.filter((_, position) => position !== index),
          ),
        /exact linked package/u,
      );
      for (const changes of [
        { fresh: true },
        { package_id: "other-package" },
        { manifest_path: "/impostor/Cargo.toml" },
        { target: { ...artifacts[index].target, src_path: "/impostor/lib.rs" } },
      ])
        assert.throws(() =>
          assertChangedPackageArtifactsBuilt(
            packages,
            artifacts.map((entry, position) =>
              position === index ? { ...entry, ...changes } : entry,
            ),
          ),
        );
    }
    assert.throws(
      () =>
        changedDependencyPackages(
          dependencies,
          { packages: metadata.packages.slice(1) },
          "/source",
        ),
      /one exact package identity/u,
    );
    assert.throws(
      () =>
        changedDependencyPackages(
          dependencies,
          { packages: [...metadata.packages, metadata.packages[0]] },
          "/source",
        ),
      /one exact package identity/u,
    );
    if (owned) {
      assert.throws(
        () =>
          changedDependencyPackages(
            dependencies,
            {
              packages: metadata.packages.map((entry) => ({
                ...entry,
                source: "registry+impostor",
              })),
            },
            "/source",
          ),
        /path package/u,
      );
      assert.throws(() => changedDependencyPackages(dependencies, metadata, "/impostor"));
    }
  }
});

test("the path host move rebuilds both exact foundation owners", () => {
  const dependencies = changedSourceDependencies([
    "davinci/vize_l0/src/lib.rs",
    "crates/vize_carton/src/path.rs",
  ]);
  assert.deepEqual(dependencies, ["vize_l0", "vize_carton"]);
  assert.deepEqual(changedSourceDependencies(["crates/vize_carton_other/src/path.rs"]), []);
  const artifacts = dependencies.map((name) => ({ target: { name }, fresh: false }));
  assert.doesNotThrow(() => assertChangedDependenciesBuilt(dependencies, artifacts));
  assert.throws(
    () => assertChangedDependenciesBuilt(dependencies, artifacts.slice(0, 1)),
    /changed vize_carton must supply an actual linked artifact/u,
  );
  assert.throws(
    () =>
      assertChangedDependenciesBuilt(
        dependencies,
        artifacts.map((artifact) => ({
          ...artifact,
          fresh: artifact.target.name === "vize_carton",
        })),
      ),
    /changed vize_carton must actually compile/u,
  );
});

test("changed Croquis macros cannot reuse the baseline analysis crate", () => {
  const dependencies = changedSourceDependencies([
    "crates/vize_croquis/src/macros/defaults.rs",
    "crates/vize_canon/src/virtual_ts/props/mappings.rs",
  ]);
  assert.deepEqual(dependencies, ["vize_croquis"]);
  assert.deepEqual(changedSourceDependencies(["crates/vize_croquis_other/src/macros.rs"]), []);
  for (const side of ["before", "after"]) {
    assert.doesNotThrow(() =>
      assertChangedDependenciesBuilt(dependencies, [
        { target: { name: "vize_croquis" }, fresh: false },
      ]),
    );
    assert.throws(
      () =>
        assertChangedDependenciesBuilt(dependencies, [
          { target: { name: "vize_croquis" }, fresh: true },
        ]),
      /changed vize_croquis must actually compile/u,
      `${side} must execute the changed dependency compiler`,
    );
    assert.throws(
      () =>
        assertChangedDependenciesBuilt(dependencies, [
          { target: { name: "vize_canon" }, fresh: false },
        ]),
      /changed vize_croquis must supply an actual linked artifact/u,
    );
  }
});

test("changed Incremental descriptors refuse the baseline's cached metadata", () => {
  const dependencies = changedSourceDependencies([
    "crates/vize_incremental/src/descriptor.rs",
    "crates/vize_incremental/src/lib.rs",
    "crates/vize_maestro/src/ide/workspace_symbols/descriptor.rs",
  ]);
  assert.deepEqual(dependencies, ["vize_incremental"]);
  assert.deepEqual(changedSourceDependencies(["crates/vize_incremental_other/src/lib.rs"]), []);
  for (const side of ["before", "after"]) {
    assert.doesNotThrow(() =>
      assertChangedDependenciesBuilt(dependencies, [
        { target: { name: "vize_incremental" }, fresh: false },
      ]),
    );
    assert.throws(
      () =>
        assertChangedDependenciesBuilt(dependencies, [
          { target: { name: "vize_incremental" }, fresh: true },
        ]),
      /changed vize_incremental must actually compile/u,
      `${side} cannot reuse the previous descriptor API metadata`,
    );
    assert.throws(
      () =>
        assertChangedDependenciesBuilt(dependencies, [
          { target: { name: "vize_maestro" }, fresh: false },
        ]),
      /changed vize_incremental must supply an actual linked artifact/u,
    );
  }
});
