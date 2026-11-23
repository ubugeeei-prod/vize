import { requiredEnv } from "../shared/files.ts";
import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { appendFileSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
const { resolveTypecheckRuntime } = (await import(
  pathToFileURL(resolveWorkflowInput("./tests/tooling/support/typecheck-dependency.ts")).href
)) as typeof import("../../../../tests/tooling/support/typecheck-dependency.ts");
const { resolveFreshVuePackage } = (await import(
  pathToFileURL(resolveWorkflowInput("./tests/tooling/support/lsp/fresh-vue-project.ts")).href
)) as typeof import("../../../../tests/tooling/support/lsp/fresh-vue-project.ts");
const hash = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const build = JSON.parse(readFileSync("target/ci/vize.differential-build.json", "utf8"));
assert.equal(build.sourceRevision, process.env.SOURCE_SHA);
const selectedRuntime = resolveTypecheckRuntime(process.cwd());
const selectedVue = resolveFreshVuePackage();
assert.ok(selectedRuntime && selectedVue, "actual native TypeScript and Vue are required");
const runtime = realpathSync(selectedRuntime);
const vuePackage = realpathSync(selectedVue);
appendFileSync(
  requiredEnv("GITHUB_ENV"),
  `CORSA_BIN=${runtime}\nVIZE_TEST_VUE_PACKAGE=${vuePackage}\n`,
);
const vue = readFileSync(`${vuePackage}/package.json`);
const receipt = {
  source: build,
  sourceTree: execFileSync("git", ["rev-parse", "HEAD^{tree}"], { encoding: "utf8" }).trim(),
  provider: "Existing Canon acknowledged projection and native Vue component classification",
  typescriptRuntime: {
    sha256: hash(readFileSync(runtime)),
    version: execFileSync(runtime, ["--version"], { encoding: "utf8" }).trim(),
  },
  vue: { version: JSON.parse(vue.toString("utf8")).version, packageJsonSha256: hash(vue) },
  scope:
    "Whole completion packets and actual native requested positions. Captured correctness runs are separate from uncaptured CI latency observations; no consumer-wide or publication admission.",
};
writeFileSync(
  "target/vize-tests/tag-prefix-classification-10622/source.json",
  JSON.stringify(receipt, null, 2) + "\n",
);
