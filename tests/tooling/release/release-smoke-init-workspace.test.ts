import assert from "node:assert/strict";
import { test } from "node:test";

import {
  WORKSPACE_PACKAGE_MANAGERS,
  workspaceRootFiles,
} from "../../../tools/support/compat/npm/smoke-release-init-workspace-roots.ts";

const redirects = {
  "@vizejs/native": "file:../../packs/vizejs-native.tgz",
  vize: "file:../../packs/vize.tgz",
};

test("fresh workspace qualification includes pnpm and Bun", () => {
  assert.deepEqual(WORKSPACE_PACKAGE_MANAGERS, ["pnpm", "bun"]);
});

test("the historical pnpm workspace root stays byte-identical", () => {
  assert.deepEqual(workspaceRootFiles("pnpm", redirects), {
    "package.json": '{\n  "name": "vize-fresh-workspace",\n  "private": true\n}\n',
    "pnpm-workspace.yaml": [
      "packages:",
      '  - "packages/*"',
      "overrides:",
      '  "@vizejs/native": "file:../../packs/vizejs-native.tgz"',
      '  "vize": "file:../../packs/vize.tgz"',
      "",
    ].join("\n"),
  });
});

test("the Bun workspace root pins the same packed graph", () => {
  const files = workspaceRootFiles("bun", redirects);
  assert.deepEqual(Object.keys(files), ["package.json"]);
  assert.deepEqual(JSON.parse(files["package.json"]), {
    name: "vize-fresh-bun-workspace",
    private: true,
    workspaces: ["packages/*"],
    overrides: redirects,
  });
});
