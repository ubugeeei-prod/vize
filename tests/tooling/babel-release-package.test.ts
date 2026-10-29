import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");

test("the Babel package is built, uploaded and published with the native compiler", () => {
  const workflow = read(".github/workflows/release.yml");
  const packages = workflow.match(/^  RELEASE_JS_PACKAGES: (.+)$/m)?.[1].split(" ");
  assert.ok(
    packages?.includes("npm/babel"),
    "the public Babel plugin must enter release smoke and npm publication",
  );
  const build = read(".github/actions/release-build-packages/action.yml");
  assert.ok(
    build.includes("--filter './npm/babel...'"),
    "release builds install the Babel package graph",
  );
  assert.ok(
    build.includes("vp run --filter './npm/babel' build"),
    "both Babel entrypoints and declarations are built",
  );
  assert.match(build, /^          npm\/babel$/m, "the built Babel package enters the npm artifact");
  const babel = JSON.parse(read("npm/babel/package.json"));
  const native = JSON.parse(read("npm/native/package.json"));
  assert.equal(
    babel.version,
    native.version,
    "the release stamp starts with matching package versions",
  );
  assert.equal(babel.dependencies["@vizejs/native"], "workspace:*");
});
