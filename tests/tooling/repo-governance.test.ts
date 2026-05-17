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
  assert.match(security, /Please do not open a public issue/);
  assert.match(security, /private vulnerability reporting/);
  assert.match(security, /latest published prerelease/);

  assert.match(contributing, /Conventional Commits/);
  assert.match(contributing, /vp install --frozen-lockfile --prefer-offline/);
  assert.match(contributing, /vp check <changed-files>/);
  assert.match(contributing, /Security issues should follow `SECURITY\.md`/);
});

test("issue templates collect reproducible production-readiness reports", () => {
  const bugReport = readRepoFile(".github", "ISSUE_TEMPLATE", "bug_report.yml");
  const featureRequest = readRepoFile(".github", "ISSUE_TEMPLATE", "feature_request.yml");
  const config = readRepoFile(".github", "ISSUE_TEMPLATE", "config.yml");

  for (const field of ["area", "version", "reproduction", "actual", "expected", "environment"]) {
    assert.match(bugReport, new RegExp(`id:\\s*${field}`));
  }
  assert.match(bugReport, /This is not a private security report/);
  assert.match(featureRequest, /id:\s*problem/);
  assert.match(featureRequest, /id:\s*proposal/);
  assert.match(featureRequest, /id:\s*compatibility/);
  assert.match(config, /blank_issues_enabled:\s*false/);
  assert.match(config, /vize\/security\/policy/);
});

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}
