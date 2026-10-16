import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

const sourceRoot = fileURLToPath(new URL("../../../", import.meta.url));
const token = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;
const release = /^\d+\.\d+\.\d+$/;

export function bootstrapCachePrefix(
  os: string | undefined,
  arch: string | undefined,
  nodeVersion: string,
  packageManager: string,
): string {
  if (!token.test(os ?? "") || !token.test(arch ?? "")) throw new Error("Invalid runner identity");
  if (!release.test(nodeVersion)) throw new Error("Invalid pinned Node.js version");
  if (!/^pnpm@\d+\.\d+\.\d+$/.test(packageManager)) throw new Error("Invalid pinned pnpm version");
  return `vp-bootstrap-v2-${os}-${arch}-node-${nodeVersion}-${packageManager.replace("@", "-")}-`;
}

// A daily trusted seed contains both the installer's bootstrap runtime and the
// repository's pinned runtime. Prefix restore keeps the previous seed usable
// without resolving latest LTS over the network in every PR job. setup-vp still
// downloads any missing exact runtime after a Node release or an installer change.
export function bootstrapCacheKey(
  os: string | undefined,
  arch: string | undefined,
  nodeVersion: string,
  packageManager: string,
  date = new Date(),
): string {
  return `${bootstrapCachePrefix(os, arch, nodeVersion, packageManager)}${date.toISOString().slice(0, 10)}`;
}

export function bootstrapCacheWrite(
  requested: boolean,
  eventName: string | undefined,
  ref: string | undefined,
  defaultBranch: string | undefined,
): boolean {
  if (!requested || !token.test(defaultBranch ?? "")) return false;
  const trusted =
    eventName === "push" || eventName === "schedule" || eventName === "workflow_dispatch";
  return trusted && ref === `refs/heads/${defaultBranch}`;
}

function emit(key: string, restore: string, save: boolean) {
  const output = process.env.GITHUB_OUTPUT;
  if (!output) throw new Error("GITHUB_OUTPUT is unset");
  appendFileSync(output, `key=${key}\nrestore=${restore}\nsave=${save}\n`);
}

export function cacheBootstrapRuntime() {
  try {
    const manifest = JSON.parse(readFileSync(resolve(sourceRoot, "package.json"), "utf8"));
    const version = manifest.devEngines?.runtime?.version;
    const manager = manifest.packageManager;
    const key = bootstrapCacheKey(process.env.RUNNER_OS, process.env.RUNNER_ARCH, version, manager);
    const restore = bootstrapCachePrefix(
      process.env.RUNNER_OS,
      process.env.RUNNER_ARCH,
      version,
      manager,
    );
    const requested = bootstrapCacheWrite(
      process.env.SAVE_REQUESTED === "true",
      process.env.EVENT_NAME,
      process.env.REF,
      process.env.DEFAULT_BRANCH,
    );
    const save =
      requested &&
      process.env.GITHUB_SHA ===
        execFileSync("git", ["rev-parse", "HEAD"], { cwd: sourceRoot, encoding: "utf8" }).trim();
    emit(key, restore, save);
    process.stdout.write(`Vite+ bootstrap cache key ${key}; save=${save}\n`);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`Vite+ bootstrap cache skipped: ${message}\n`);
    emit("", "", false);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  cacheBootstrapRuntime();
