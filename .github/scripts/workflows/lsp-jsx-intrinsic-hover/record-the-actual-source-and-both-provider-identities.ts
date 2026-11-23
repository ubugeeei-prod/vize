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
assert.ok(selectedRuntime && selectedVue, "the actual protocol dependencies are required");
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
  providers: {
    intrinsic: "Original retained native Program and completed File; same-walk intrinsic observer",
    otherTypedHover: "Existing Canon TypeScript projection",
    typescriptRuntime: {
      sha256: hash(readFileSync(runtime)),
      version: execFileSync(runtime, ["--version"], { encoding: "utf8" }).trim(),
    },
    vue: { version: JSON.parse(vue.toString("utf8")).version, packageJsonSha256: hash(vue) },
  },
  latencyMeaning:
    "First DOM requests follow initial diagnostics; warm groups repeat unchanged full-result queries. Whole-request CI observations, not cold startup or universal budgets.",
  scope:
    "Source-qualified bounded intrinsic hover packets and observed CI latency; no native whole-product or publication admission",
};
writeFileSync(
  "target/vize-tests/jsx-intrinsic-hover/source.json",
  JSON.stringify(receipt, null, 2) + "\n",
);
