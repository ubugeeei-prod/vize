import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { root } from "./support/github-workflows.ts";

const scriptPath = path.join(
  root,
  "tools",
  "moon",
  "scripts",
  "github",
  "issue_pr_title_policy.mbtx",
);

function runPolicy(payload: unknown, eventName: string) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-title-policy-"));
  const binDir = path.join(tempDir, "bin");
  const eventPath = path.join(tempDir, "event.json");
  const ghLogPath = path.join(tempDir, "gh.log");
  const fakeGhPath = path.join(binDir, "gh");

  fs.mkdirSync(binDir);
  fs.writeFileSync(eventPath, JSON.stringify(payload));
  fs.writeFileSync(
    fakeGhPath,
    [
      "#!/usr/bin/env node",
      'const fs = require("node:fs");',
      "fs.appendFileSync(process.env.FAKE_GH_LOG, JSON.stringify(process.argv.slice(2)) + '\\n');",
    ].join("\n"),
  );
  fs.chmodSync(fakeGhPath, 0o755);

  const result = spawnSync("moon", ["run", "--target", "native", "-", "--"], {
    cwd: root,
    input: fs.readFileSync(scriptPath),
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      FAKE_GH_LOG: ghLogPath,
      GITHUB_EVENT_NAME: eventName,
      GITHUB_EVENT_PATH: eventPath,
      GITHUB_REPOSITORY: "ubugeeei/vize",
    },
  });

  const ghCalls = fs.existsSync(ghLogPath)
    ? fs
        .readFileSync(ghLogPath, "utf8")
        .trim()
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line) as string[])
    : [];

  fs.rmSync(tempDir, { recursive: true, force: true });

  return { result, ghCalls };
}

test("issue title policy normalizes titles and assigns new issues", () => {
  const { result, ghCalls } = runPolicy(
    {
      action: "opened",
      issue: {
        number: 12,
        title: "check compiler lint linter story format fmt checklist formatting",
        assignees: [],
      },
    },
    "issues",
  );

  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(ghCalls, [
    [
      "api",
      "--method",
      "PATCH",
      "-H",
      "X-GitHub-Api-Version: 2022-11-28",
      "--silent",
      "/repos/ubugeeei/vize/issues/12",
      "-f",
      "title=canon atelier patina patina musea glyph glyph checklist formatting",
    ],
    [
      "api",
      "--method",
      "POST",
      "-H",
      "X-GitHub-Api-Version: 2022-11-28",
      "--silent",
      "/repos/ubugeeei/vize/issues/12/assignees",
      "-F",
      "assignees[]=ubugeeei",
    ],
  ]);
});

test("PR title policy normalizes conventional titles before validation", () => {
  const { result, ghCalls } = runPolicy(
    {
      action: "opened",
      pull_request: {
        number: 34,
        title: "check: update lint rules",
        assignees: [{ login: "ubugeeei" }],
      },
    },
    "pull_request_target",
  );

  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(ghCalls, [
    [
      "api",
      "--method",
      "PATCH",
      "-H",
      "X-GitHub-Api-Version: 2022-11-28",
      "--silent",
      "/repos/ubugeeei/vize/issues/34",
      "-f",
      "title=canon: update patina rules",
    ],
  ]);
});

test("PR title policy fails non-conventional titles after normalization", () => {
  const { result, ghCalls } = runPolicy(
    {
      action: "opened",
      pull_request: {
        number: 56,
        title: "fix lint issue",
        assignees: [],
      },
    },
    "pull_request_target",
  );

  assert.equal(result.status, 1);
  assert.match(result.stdout, /Invalid PR title/);
  assert.deepEqual(ghCalls, [
    [
      "api",
      "--method",
      "PATCH",
      "-H",
      "X-GitHub-Api-Version: 2022-11-28",
      "--silent",
      "/repos/ubugeeei/vize/issues/56",
      "-f",
      "title=fix patina issue",
    ],
    [
      "api",
      "--method",
      "POST",
      "-H",
      "X-GitHub-Api-Version: 2022-11-28",
      "--silent",
      "/repos/ubugeeei/vize/issues/56/assignees",
      "-F",
      "assignees[]=ubugeeei",
    ],
  ]);
});
