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
  const kind =
    env.GITHUB_EVENT_NAME === "merge_group"
      ? "merge_group"
      : env.GITHUB_BASE_REF || env.GITHUB_EVENT_NAME === "pull_request"
        ? "pull_request"
        : undefined;
  if (!kind) {
    return undefined;
  }

  assert.ok(env.GITHUB_EVENT_PATH, `GITHUB_EVENT_PATH is required for ${kind} source checks`);
  let event: unknown;
  try {
    event = JSON.parse(fs.readFileSync(env.GITHUB_EVENT_PATH, "utf8"));
  } catch (error) {
    throw new Error(`Failed to read ${kind} event ${env.GITHUB_EVENT_PATH}`, { cause: error });
  }
  assert.ok(event !== null && typeof event === "object", `${kind} event must be an object`);
  const payload = event as {
    merge_group?: { base_sha?: unknown };
    pull_request?: { base?: { sha?: unknown } };
  };
  const baseSha =
    kind === "merge_group" ? payload.merge_group?.base_sha : payload.pull_request?.base?.sha;
  const field = kind === "merge_group" ? "merge_group.base_sha" : "pull_request.base.sha";
  assert.ok(
    typeof baseSha === "string" && /^[0-9a-f]{40}$/.test(baseSha),
    `${field} must be a full lowercase commit SHA`,
  );

  const result = spawnSync("git", ["fetch", "--no-tags", "--depth=1", "origin", baseSha], {
    cwd,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return baseSha;
}
