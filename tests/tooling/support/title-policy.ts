import fs from "node:fs";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { writeFakeCommand } from "./fake-command.ts";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const repository = "ubugeeei/vize";
const commandPath = "tools/commands/ci/github/issue-pr-title-policy.rs";

export type PolicyRun = {
  result: SpawnSyncReturns<string>;
  ghCalls: string[][];
};

/**
 * Runs the issue/PR title policy Rust Script command against a real event payload
 * with `gh` stubbed out. The stub records the exact argv of every invocation so
 * tests can assert the complete API call list, including "no call at all".
 */
export function runPolicy(payload: unknown, eventName: string): PolicyRun {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-title-policy-"));
  const binDir = path.join(tempDir, "bin");
  const eventPath = path.join(tempDir, "event.json");
  const ghLogPath = path.join(tempDir, "gh.log");

  fs.mkdirSync(binDir);
  fs.writeFileSync(eventPath, JSON.stringify(payload));
  writeFakeCommand(
    binDir,
    "gh",
    [
      "const fs = require('node:fs');",
      "fs.appendFileSync(process.env.FAKE_GH_LOG, JSON.stringify(process.argv.slice(2)) + '\\n');",
    ].join("\n"),
  );

  const env = {
    ...process.env,
    PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
    FAKE_GH_LOG: ghLogPath,
    GITHUB_EVENT_NAME: eventName,
    GITHUB_EVENT_PATH: eventPath,
    GITHUB_REPOSITORY: repository,
  };
  delete env.NODE_TEST_CONTEXT;

  const result = spawnSync("rust-script", [commandPath], {
    cwd: repoRoot,
    env,
    encoding: "utf8",
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

/** The exact argv the tool uses to rewrite a title. */
export function patchTitleCall(issueNumber: number, title: string): string[] {
  return [
    "api",
    "--method",
    "PATCH",
    "-H",
    "X-GitHub-Api-Version: 2022-11-28",
    "--silent",
    `/repos/${repository}/issues/${issueNumber}`,
    "-f",
    `title=${title}`,
  ];
}

/** The exact argv the tool uses to assign the default maintainer. */
export function assignCall(issueNumber: number): string[] {
  return [
    "api",
    "--method",
    "POST",
    "-H",
    "X-GitHub-Api-Version: 2022-11-28",
    "--silent",
    `/repos/${repository}/issues/${issueNumber}/assignees`,
    "-F",
    "assignees[]=ubugeeei",
  ];
}
