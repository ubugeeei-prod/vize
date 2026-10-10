import { execFileSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const policyPath = "jsr/vize/channel.json";
const repository = fileURLToPath(new URL("../../../../", import.meta.url));

export function parseChannelPolicy(bytes) {
  const policy = JSON.parse(bytes);
  if (
    !policy ||
    Array.isArray(policy) ||
    Object.keys(policy).sort().join(",") !== "enabled,schema" ||
    policy.schema !== "vize-jsr-channel-v1" ||
    typeof policy.enabled !== "boolean"
  ) {
    throw new Error("Exact JSR channel schema and boolean enabled policy required");
  }
  return policy;
}

export function frozenChannelPolicy(head, { root = repository } = {}) {
  if (!/^[0-9a-f]{40}$/.test(head)) throw new Error("Full frozen source H required");
  const git = (...args) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      cwd: resolve(root),
      env: { ...process.env, GIT_NO_REPLACE_OBJECTS: "1" },
      stdio: ["ignore", "pipe", "pipe"],
      maxBuffer: 64 * 1024,
      timeout: 30_000,
    }).toString("utf8");
  if (git("for-each-ref", "--format=%(refname)", "refs/replace").trim()) {
    throw new Error("Replacement objects cannot authorize a JSR release");
  }
  if (git("cat-file", "-t", head).trim() !== "commit") {
    throw new Error("Frozen source H must be a commit");
  }
  const entry = git("ls-tree", "-z", head, "--", policyPath);
  const match = /^100644 blob ([0-9a-f]{40})\tjsr\/vize\/channel\.json\0$/.exec(entry);
  if (!match) throw new Error("Frozen H requires the regular JSR channel policy blob");
  return parseChannelPolicy(git("cat-file", "blob", match[1]));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [head, ...extra] = process.argv.slice(2);
  if (extra.length) throw new Error("Usage: channel.mjs <frozen-source-H>");
  console.log(`jsr_required=${frozenChannelPolicy(head).enabled}`);
}
