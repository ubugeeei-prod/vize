import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("repository governance docs cover contribution and security paths", () => {
  const security = readRepoFile("SECURITY.md");
  const contributing = readRepoFile("CONTRIBUTING.md");

  assert.match(security, /Supported Versions/);
  assert.match(security, /Please do not open a public tracker entry/);
  assert.match(security, /private vulnerability reporting/);
  assert.match(security, /latest published prerelease/);

  assert.match(contributing, /Conventional Commits/);
  assert.match(contributing, /vp install --frozen-lockfile --prefer-offline/);
  assert.match(contributing, /vp check <changed-files>/);
  assert.match(contributing, /Security reports should follow `SECURITY\.md`/);
});

test("fix templates collect reproducible production-readiness reports", () => {
  const fixReport = readRepoFile(".github", "ISSUE_TEMPLATE", "fix_report.yml");
  const featureRequest = readRepoFile(".github", "ISSUE_TEMPLATE", "feature_request.yml");
  const config = readRepoFile(".github", "ISSUE_TEMPLATE", "config.yml");

  for (const field of ["area", "version", "reproduction", "actual", "expected", "environment"]) {
    assert.match(fixReport, new RegExp(`id:\\s*${field}`));
  }
  assert.match(fixReport, /This is not a private security report/);
  assert.match(featureRequest, /id:\s*problem/);
  assert.match(featureRequest, /id:\s*proposal/);
  assert.match(featureRequest, /id:\s*compatibility/);
  assert.match(config, /blank_issues_enabled:\s*false/);
  assert.match(config, /vize\/security\/policy/);
});

test("root quality gates ignore local generated environments", () => {
  const viteConfig = readRepoFile("vite.config.ts");

  assert.match(viteConfig, /localGeneratedIgnorePatterns/);
  for (const pattern of [".cache/**", ".direnv/**", "target/**"]) {
    assert.match(viteConfig, new RegExp(pattern.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
  }
});

test("CodeRabbit reviews run in strict mode with actionable workflow context", () => {
  const coderabbit = readRepoFile(".coderabbit.yaml");

  assert.match(coderabbit, /^  profile:\s*assertive$/m);
  assert.match(coderabbit, /^  request_changes_workflow:\s*true$/m);
  assert.match(coderabbit, /^  review_details:\s*true$/m);
  assert.match(coderabbit, /artifact\/linkage verification/);
  assert.match(coderabbit, /workflow-shape tests cover the changed behavior/);
});

test("socket.dev configuration scopes dependency and workflow scans", () => {
  const socket = readRepoFile("socket.yml");

  assert.match(socket, /^version:\s*2$/m);
  assert.match(socket, /projectIgnorePaths:\n(?:\s+- .+\n)+/);
  assert.match(socket, /triggerPaths:\n(?:\s+- .+\n)+/);
  assert.match(socket, /githubApp:\n/);
  assert.match(socket, /\s+enabled:\s*true/);
  assert.match(socket, /\s+pullRequestAlertsEnabled:\s*true/);
  assert.match(socket, /\s+projectReportsEnabled:\s*true/);

  for (const pattern of [
    "/tests/_fixtures",
    "/pnpm-lock.yaml",
    "/Cargo.lock",
    "/.github/workflows/*.yml",
  ]) {
    assert.match(socket, new RegExp(pattern.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
  }
});

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}
