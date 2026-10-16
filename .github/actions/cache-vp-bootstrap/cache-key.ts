import { appendFileSync } from "node:fs";
import { get } from "node:https";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

const token = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;
const release = /^(\d+)\.(\d+)\.(\d+)$/;
const indexUrl = "https://nodejs.org/dist/index.json";

function compareRelease(left, right) {
  for (let index = 0; index < 3; index += 1) {
    if (left[index] !== right[index]) return left[index] - right[index];
  }
  return 0;
}

// Highest Node.js release whose index entry names an LTS codename. That is the
// version `vp` resolves before it downloads the runtime used to install itself.
export function latestLtsVersion(index) {
  if (!Array.isArray(index)) throw new Error("Node.js index is not a list");
  let best = null;
  let bestParts = null;
  for (const entry of index) {
    if (entry === null || typeof entry !== "object" || typeof entry.lts !== "string") continue;
    if (typeof entry.version !== "string" || !entry.version.startsWith("v")) continue;
    const version = entry.version.slice(1);
    const match = release.exec(version);
    if (!match) continue;
    const parts = [Number(match[1]), Number(match[2]), Number(match[3])];
    if (!bestParts || compareRelease(parts, bestParts) > 0) {
      best = version;
      bestParts = parts;
    }
  }
  if (!best) throw new Error("Node.js index has no LTS release");
  return best;
}

export function bootstrapCacheKey(os, arch, nodeVersion) {
  if (!token.test(os ?? "") || !token.test(arch ?? "")) throw new Error("Invalid runner identity");
  if (!release.test(nodeVersion ?? "")) throw new Error("Invalid Node.js LTS version");
  return `vp-bootstrap-v1-${os}-${arch}-node-${nodeVersion}`;
}

// One default-branch entry. Pull requests restore it and must not replace it.
export function bootstrapCacheSave(eventName, ref, defaultBranch) {
  if (!token.test(defaultBranch ?? "")) return false;
  const trusted =
    eventName === "push" || eventName === "schedule" || eventName === "workflow_dispatch";
  return trusted && ref === `refs/heads/${defaultBranch}`;
}

// The caller must ask to save, and the event must still be a trusted default
// branch. A pull request cannot turn the request on.
export function bootstrapCacheWrite(requested, eventName, ref, defaultBranch) {
  return requested === true && bootstrapCacheSave(eventName, ref, defaultBranch);
}

function fetchIndex(url = indexUrl, redirects = 0) {
  return new Promise((resolveIndex, reject) => {
    const request = get(
      url,
      { headers: { "user-agent": "vize-cache-vp-bootstrap" }, timeout: 15_000 },
      (response) => {
        const status = response.statusCode ?? 0;
        const location = response.headers.location;
        if (status >= 300 && status < 400 && location) {
          response.resume();
          if (redirects >= 3) {
            reject(new Error("Node.js index redirected too many times"));
            return;
          }
          const next = new URL(location, url);
          if (next.protocol !== "https:") {
            reject(new Error("Node.js index redirected off HTTPS"));
            return;
          }
          resolveIndex(fetchIndex(next.href, redirects + 1));
          return;
        }
        if (status !== 200) {
          response.resume();
          reject(new Error(`Node.js index status ${status}`));
          return;
        }
        const chunks = [];
        response.on("data", (chunk) => chunks.push(chunk));
        response.on("end", () => {
          try {
            resolveIndex(JSON.parse(Buffer.concat(chunks).toString("utf8")));
          } catch (error) {
            reject(error instanceof Error ? error : new Error(String(error)));
          }
        });
      },
    );
    request.on("timeout", () => request.destroy(new Error("Node.js index timed out")));
    request.on("error", reject);
  });
}

function emit(key, save) {
  const output = process.env.GITHUB_OUTPUT;
  if (!output) throw new Error("GITHUB_OUTPUT is unset");
  appendFileSync(output, `key=${key}\nsave=${save}\n`);
}

async function main() {
  try {
    const version = latestLtsVersion(await fetchIndex());
    const key = bootstrapCacheKey(process.env.RUNNER_OS, process.env.RUNNER_ARCH, version);
    const save = bootstrapCacheWrite(
      process.env.SAVE_REQUESTED === "true",
      process.env.EVENT_NAME,
      process.env.REF,
      process.env.DEFAULT_BRANCH,
    );
    emit(key, save ? "true" : "false");
    process.stdout.write(`Vite+ bootstrap cache key ${key}; save=${save}\n`);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`Vite+ bootstrap cache skipped: ${message}\n`);
    emit("", "false");
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
