import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";

export function resolveSourceLengthBase(
  cwd: string,
  env: NodeJS.ProcessEnv = process.env,
): string | undefined {
  if (env.SOURCE_LENGTH_BASE_REF) {
    return env.SOURCE_LENGTH_BASE_REF;
  }
  if (!env.GITHUB_BASE_REF) {
    return undefined;
  }

  assert.ok(env.GITHUB_EVENT_PATH, "GITHUB_EVENT_PATH is required for pull-request source checks");
  let event: unknown;
  try {
    event = JSON.parse(fs.readFileSync(env.GITHUB_EVENT_PATH, "utf8"));
  } catch (error) {
    throw new Error(`Failed to read pull-request event ${env.GITHUB_EVENT_PATH}`, {
      cause: error,
    });
  }
  const baseSha = (event as { pull_request?: { base?: { sha?: unknown } } }).pull_request?.base
    ?.sha;
  assert.ok(
    typeof baseSha === "string" && /^[0-9a-f]{40}$/.test(baseSha),
    "pull_request.base.sha must be a full lowercase commit SHA",
  );

  const result = spawnSync("git", ["fetch", "--no-tags", "--depth=1", "origin", baseSha], {
    cwd,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return baseSha;
}
