import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertChangedDependenciesBuilt,
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
