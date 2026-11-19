import assert from "node:assert/strict";
import { test } from "node:test";
import {
  inspectDirectoryDependencies,
  type CargoDependency,
} from "../../tools/support/compat/davinci/directory-dependencies.ts";

type FixtureDependency = CargoDependency & { optional: boolean; rename: string | null };

function fixture() {
  const packages: {
    id: string;
    name: string;
    manifest_path: string;
    dependencies: FixtureDependency[];
  }[] = [
    ["native", "davinci"],
    ["product", "crates"],
    ["helper", "tools"],
  ].map(([name, directory]) => ({
    id: name,
    name,
    manifest_path: `/fixture/${directory}/${name}/Cargo.toml`,
    dependencies: [],
  }));
  const metadata = {
    workspace_root: "/fixture",
    workspace_members: packages.map((pkg) => pkg.id),
    packages,
  };
  const add = (from: string, to: string, extra: Partial<FixtureDependency> = {}) => {
    const pkg = packages.find((candidate) => candidate.name === from);
    assert.ok(pkg, from);
    pkg.dependencies.push({
      name: to,
      kind: null,
      target: null,
      optional: false,
      rename: null,
      ...extra,
    });
  };
  return { metadata, add, report: () => inspectDirectoryDependencies(metadata) };
}

void test("products consume Davinci and dev-only differential oracles remain permitted", () => {
  const value = fixture();
  value.add("product", "native");
  value.add("native", "product", { kind: "dev" });
  assert.deepEqual(value.report(), []);
});

for (const extra of [
  {},
  { kind: "build" },
  { optional: true },
  { target: "cfg(windows)" },
  { rename: "vize_l0" },
]) {
  void test(`reverse declaration is rejected: ${JSON.stringify(extra)}`, () => {
    const value = fixture();
    value.add("native", "product", extra);
    assert.deepEqual(
      value.report().map((entry) => entry.path),
      [["native", "product"]],
    );
  });
}

void test("an external helper cannot hide a transitive reverse edge or loop forever", () => {
  const value = fixture();
  value.add("native", "helper");
  value.add("helper", "native");
  value.add("helper", "product", { kind: "build" });
  assert.deepEqual(
    value.report().map((entry) => entry.path),
    [["native", "helper", "product"]],
  );
});
