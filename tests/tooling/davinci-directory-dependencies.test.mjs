import assert from "node:assert/strict";
import { test } from "node:test";
import { inspectDirectoryDependencies } from "../../tools/support/compat/davinci/directory-dependencies.mjs";

function fixture() {
  const packages = [
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
  const add = (from, to, extra = {}) =>
    packages
      .find((pkg) => pkg.name === from)
      .dependencies.push({
        name: to,
        kind: null,
        target: null,
        optional: false,
        rename: null,
        ...extra,
      });
  return { metadata, add, report: () => inspectDirectoryDependencies(metadata) };
}

test("products consume Davinci and dev-only differential oracles remain permitted", () => {
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
  test(`reverse declaration is rejected: ${JSON.stringify(extra)}`, () => {
    const value = fixture();
    value.add("native", "product", extra);
    assert.deepEqual(
      value.report().map((entry) => entry.path),
      [["native", "product"]],
    );
  });
}

test("an external helper cannot hide a transitive reverse edge or loop forever", () => {
  const value = fixture();
  value.add("native", "helper");
  value.add("helper", "native");
  value.add("helper", "product", { kind: "build" });
  assert.deepEqual(
    value.report().map((entry) => entry.path),
    [["native", "helper", "product"]],
  );
});
