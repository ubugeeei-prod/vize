import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import {
  arg,
  readJson,
  sha256,
  type LintReport,
  type PackageManifest,
  type RoleBefore,
} from "./utils.ts";

const workspace = path.resolve(arg(0));
const version = arg(1);
const out = arg(2);
const require = createRequire(path.join(workspace, "package.json"));
const manifest = require("@vizejs/native/package.json") as PackageManifest;
assert.equal(manifest.version, version);
const beforeUrl = new URL("./public-before-0.596.json.txt", import.meta.url);
assert.equal(
  sha256(fs.readFileSync(beforeUrl)),
  "6ca13af2ce200795ce72823a60741edf62b3da36cf22c84f71fca09b12860495",
);
const before = readJson<RoleBefore>(beforeUrl);
const native = require("@vizejs/native") as {
  lintPatinaSfc(source: string, options: Record<string, unknown>): LintReport;
};
const positive = native.lintPatinaSfc(before.source, before.options);
const negative = native.lintPatinaSfc(before.source.replace(":role=", ":data="), before.options);
const evidence = {
  version: manifest.version,
  source: before.source,
  options: before.options,
  positive,
  negative,
  negativeExpected: before.result,
};
fs.writeFileSync(out, JSON.stringify(evidence, null, 2) + "\n");
assert.deepEqual(positive, {
  filename: "ClickableTag.vue",
  errorCount: 0,
  warningCount: 0,
  diagnostics: [],
});
assert.deepEqual(negative, before.result);
console.log(
  JSON.stringify({
    version: manifest.version,
    boundRole: positive.warningCount,
    unrelatedBinding: negative.warningCount,
  }),
);
