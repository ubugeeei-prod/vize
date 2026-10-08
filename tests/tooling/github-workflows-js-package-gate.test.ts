import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("check workflow runs JS package unit tests and production dependency audit", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const packageJson = JSON.parse(readRepoFile("package.json"));
  const pnpmWorkspace = readRepoFile("pnpm-workspace.yaml");
  const jsPackageJob = workflowJobBody(workflow, "test-js-packages");
  const auditJob = workflowJobBody(workflow, "security-audit");

  assert.equal(packageJson.packageManager, "pnpm@12.1.0");
  assert.equal(packageJson.pnpm, undefined);
  assert.match(pnpmWorkspace, /^overrides:\n/m);
  assert.match(
    pnpmWorkspace,
    /^allowBuilds:\n  "@parcel\/watcher": false\n  core-js: false\n  esbuild: true\n  puppeteer: false\n  unrs-resolver: false\n  vue-demi: false$/m,
  );
  assert.match(jsPackageJob, /uses: \.\/\.github\/actions\/test-js-packages-with-history/);
  const jsAction = readRepoFile(
    ".github",
    "actions",
    "test-js-packages-with-history",
    "action.yml",
  );
  assert.match(jsAction, /run: vp run --workspace-root test:js/);
  assert.match(jsPackageJob, /uses: \.\/\.github\/actions\/setup-js-package-tests/);
  const jsSetup = readRepoFile(".github", "actions", "setup-js-package-tests", "action.yml");
  assert.match(jsSetup, /key:\s*test-js-packages/);
  assert.match(auditJob, /setup-rust-script[\s\S]*rust-script tools\/commands\/ci\/npm-audit\.rs/);
  assert.match(auditJob, /tool:\s*cargo-audit/);
  assert.match(auditJob, /cargo audit --deny warnings/);
  assert.doesNotMatch(auditJob, /continue-on-error:\s*true/);
});
