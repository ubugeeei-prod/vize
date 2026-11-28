import assert from "node:assert/strict";
import { test } from "node:test";
import type { CargoDependency } from "../../tools/support/compat/davinci/directory-dependencies.ts";
import {
  inspectStageDependencies,
  stageDependencies,
  stageDependencyClosure,
} from "../../tools/support/compat/davinci/stage-dependencies.ts";

function fixture() {
  const packages = [...Object.keys(stageDependencies), "helper"].map((name) => ({
    id: name,
    name,
    manifest_path: `/fixture/davinci/${name}/Cargo.toml`,
    dependencies: (stageDependencies[name] ?? []).map((target): CargoDependency => ({
      name: target,
      kind: null,
      target: null,
    })),
  }));
  const metadata = {
    workspace_root: "/fixture",
    workspace_members: packages.map((pkg) => pkg.id),
    packages,
  };
  const add = (
    from: string,
    to: string,
    extra: Partial<CargoDependency> & { optional?: boolean; rename?: string } = {},
  ) => {
    const pkg = packages.find((candidate) => candidate.name === from);
    assert.ok(pkg, from);
    pkg.dependencies.push({ name: to, kind: null, target: null, ...extra });
  };
  return { packages, metadata, add, report: () => inspectStageDependencies(metadata) };
}

test("all physical levels, conversions and the derive leaf have one acyclic role graph", () => {
  const value = fixture();
  assert.deepEqual(value.report(), []);
  assert.deepEqual([...stageDependencyClosure("vize_l0")], ["vize_l0_derive"]);
  assert.ok(stageDependencyClosure("vize_l4").has("vize_l1_to_l2"));
});

for (const extra of [
  {},
  { kind: "build" },
  { optional: true },
  { target: "cfg(windows)" },
  { rename: "foundation" },
]) {
  test(`L0 cannot consume L4 through a declaration: ${JSON.stringify(extra)}`, () => {
    const value = fixture();
    value.add("vize_l0", "vize_l4", extra);
    assert.deepEqual(
      value.report().map((entry) => entry.path),
      [["vize_l0", "vize_l4"]],
    );
  });
}

for (const kind of [null, "build"]) {
  test(`the derive leaf cannot depend on L0 (${kind ?? "normal"})`, () => {
    const value = fixture();
    value.add("vize_l0_derive", "vize_l0", { kind });
    assert.deepEqual(
      value.report().map((entry) => entry.path),
      [["vize_l0_derive", "vize_l0"]],
    );
  });
}

test("conversion roles reject build edges that a coarse tier number would permit", () => {
  const value = fixture();
  value.add("vize_l1_to_l2", "vize_l3", { kind: "build" });
  assert.equal(value.report()[0]?.reason, "forbidden-stage-edge");
});

test("helpers cannot hide a reverse normal/build edge or loop forever", () => {
  const value = fixture();
  value.add("vize_l0", "helper");
  value.add("helper", "helper");
  value.add("helper", "vize_l4", { kind: "build", target: "cfg(windows)" });
  assert.deepEqual(
    value.report().map((entry) => entry.path),
    [["vize_l0", "helper", "vize_l4"]],
  );
});

test("dev-only differential oracles and downward helper paths remain permitted", () => {
  const value = fixture();
  value.add("vize_l0", "vize_l4", { kind: "dev" });
  value.add("vize_l3", "helper", { kind: "build" });
  value.add("helper", "vize_l1");
  assert.deepEqual(value.report(), []);
});

test("a new physical stage requires a declared dependency role", () => {
  const value = fixture();
  const name = "vize_l5";
  value.packages.push({
    id: name,
    name,
    manifest_path: `/fixture/davinci/${name}/Cargo.toml`,
    dependencies: [],
  });
  value.metadata.workspace_members.push(name);
  assert.equal(value.report()[0]?.reason, "unregistered-stage");
});

test("an alias cannot replace an existing physical stage owner", () => {
  const value = fixture();
  assert.throws(
    () => inspectStageDependencies(value.metadata, { vize_l4: "helper" }),
    /physical stage owner/u,
  );
  assert.throws(
    () => inspectStageDependencies(value.metadata, { vize_l4: "vize_l0" }),
    /physical stage owner/u,
  );
});
