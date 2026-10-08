import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { relocateTaskConfig } from "./config-paths.ts";

const root = path.resolve("project with spaces");

for (const value of ["tsconfig.app.json", "configs/tsconfig.json", "../shared/tsconfig.json"]) {
  void test(`selected TypeScript project ${value} survives config relocation`, () => {
    const config = {
      typeChecker: { tsconfig: value, enabled: true, servers: 2, checkProps: false },
      ignores: ["generated/**"],
    };
    const original = structuredClone(config);
    const relocated = JSON.parse(JSON.stringify(relocateTaskConfig(config, root)));
    assert.deepEqual(relocated.typeChecker, {
      ...config.typeChecker,
      tsconfig: path.resolve(root, value),
    });
    assert.notEqual(relocated.typeChecker.tsconfig, path.resolve(root, "temporary/config", value));
    assert.deepEqual(config, original);
  });
}

void test("absolute project paths and disabled checker options retain their meaning", () => {
  const typeChecker = { tsconfig: path.resolve("sibling/tsconfig.json"), enabled: false };
  const config = { typeChecker };
  const relocated = relocateTaskConfig(config, root);
  assert.deepEqual(relocated.typeChecker, typeChecker);
  assert.notEqual(relocated.typeChecker, typeChecker);
  assert.deepEqual(config, { typeChecker });
});

void test("a checker without a selected project does not acquire one in serialized config", () => {
  const config = { typeChecker: { enabled: true, checkProps: false } };
  assert.deepEqual(JSON.parse(JSON.stringify(relocateTaskConfig(config, root))).typeChecker, {
    enabled: true,
    checkProps: false,
  });
  assert.equal("typeChecker" in JSON.parse(JSON.stringify(relocateTaskConfig({}, root))), false);
});
