import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  experimentalRouterFixture,
  experimentalRouterPage,
  readExperimentalRouterFixture,
} from "./support/router-experimental-provider.ts";

test("experimental Router fixture pins the actual published provider and complete generated map", () => {
  const { authority, files } = readExperimentalRouterFixture();
  assert.equal(authority.issue, 10520);
  assert.deepEqual(
    [...files.keys()].sort(),
    [
      "generate.mjs",
      "package-lock.json",
      "package.json",
      "src/pages/index.vue",
      experimentalRouterPage,
      "src/params/date.ts",
      "src/router/index.ts",
      "tsconfig.json",
      "typed-router.d.ts",
    ].sort(),
  );
  assert.equal(authority.providerFiles.length, 4);
  assert.deepEqual(JSON.parse(files.get("tsconfig.json")!.toString()).vueCompilerOptions, {
    plugins: ["vue-router/volar/sfc-typed-router"],
  });
});

test("source and lock drift are rejected before any provider process runs", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "router-experimental-custody-"));
  try {
    fs.cpSync(experimentalRouterFixture, root, { recursive: true });
    const file = path.join(root, "project", `${experimentalRouterPage}.txt`);
    const original = fs.readFileSync(file);
    fs.appendFileSync(file, "\n");
    assert.throws(() => readExperimentalRouterFixture(root));
    fs.writeFileSync(file, original);
    const lock = path.join(root, "project/package-lock.json.txt");
    const value = JSON.parse(fs.readFileSync(lock, "utf8"));
    value.packages["node_modules/vue-router"].version = "5.1.0";
    fs.writeFileSync(lock, JSON.stringify(value));
    assert.throws(() => readExperimentalRouterFixture(root));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
