import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync, existsSync, readFileSync } from "node:fs";
import { dirname, resolve, sep } from "node:path";
import { homedir } from "node:os";
import { fileURLToPath } from "node:url";

const sourceRoot = fileURLToPath(new URL("../../../", import.meta.url));
const knownEvents = new Set([
  "push",
  "schedule",
  "workflow_dispatch",
  "pull_request",
  "pull_request_target",
  "merge_group",
  "workflow_run",
  "release",
  "repository_dispatch",
]);
const trustedEvents = new Set(["push", "schedule", "workflow_dispatch"]);
const fullSha = /^[0-9a-f]{40}$/;
const token = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;
const object = (value) => value !== null && typeof value === "object" && !Array.isArray(value);

function requireToken(value, label) {
  if (!token.test(value ?? "")) throw new Error(`Invalid ${label}`);
  return value;
}

function targetPath(value, workspace) {
  if (typeof value !== "string" || /[\r\n\0]/.test(value)) throw new Error("Invalid target path");
  const path = resolve(workspace, value);
  if (!path.startsWith(`${resolve(workspace)}${sep}`))
    throw new Error("Target must be inside workspace");
  return path;
}

function fingerprint(cwd, fallback = cwd) {
  const hash = createHash("sha256");
  for (const file of ["Cargo.lock", "rust-toolchain.toml"]) {
    const root = existsSync(resolve(cwd, file)) ? cwd : fallback;
    hash
      .update(file)
      .update("\0")
      .update(readFileSync(resolve(root, file)))
      .update("\0");
  }
  return hash.digest("hex");
}

export function rustCachePolicy(context, { cwd = sourceRoot, workspace = cwd } = {}) {
  const { eventName, event, ref, repository, sourceSha, checkoutSha, runnerEnvironment } = context;
  if (!knownEvents.has(eventName) || !object(event) || !object(event.repository)) {
    throw new Error("Missing or unknown cache event");
  }
  const branch = event.repository.default_branch;
  if (
    typeof branch !== "string" ||
    !branch ||
    /[\r\n\0]/.test(branch) ||
    !/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository ?? "") ||
    event.repository.full_name !== repository ||
    typeof ref !== "string" ||
    !/^refs\/(heads|tags|pull)\/[^\s\0]+$/.test(ref) ||
    !fullSha.test(sourceSha ?? "") ||
    !fullSha.test(checkoutSha ?? "") ||
    !["github-hosted", "self-hosted"].includes(runnerEnvironment)
  )
    throw new Error("Malformed cache event metadata");
  if (eventName === "push" && (event.ref !== ref || typeof event.deleted !== "boolean")) {
    throw new Error("Malformed push event");
  }
  if (["pull_request", "pull_request_target"].includes(eventName) && !object(event.pull_request)) {
    throw new Error("Malformed pull request event");
  }
  if (eventName === "merge_group" && !object(event.merge_group))
    throw new Error("Malformed merge event");
  const role = requireToken(context.role, "cache role");
  const suffix = requireToken(context.suffix, "runner suffix");
  const runnerOs = requireToken(context.runnerOs, "runner OS");
  const runnerArch = requireToken(context.runnerArch, "runner architecture");
  const primaryPath = targetPath(context.targetPath ?? "target", workspace);
  const trusted =
    trustedEvents.has(eventName) &&
    ref === `refs/heads/${branch}` &&
    sourceSha === checkoutSha &&
    event.deleted !== true;
  const sticky = trusted && runnerEnvironment === "self-hosted";
  const base = `rust-cache-v1-${repository}-${runnerOs}-${runnerArch}-${suffix}-${fingerprint(cwd)}`;
  const target = `${base}-${role}-target-`;
  const output = {
    sticky: String(sticky),
    "registry-key": `${base}-registry`,
    "git-key": `${base}-git`,
    "target-key": `${target}${checkoutSha}`,
    "target-restore": target,
    "target-path": primaryPath,
    "sticky-registry-key": `${repository}-cargo-registry-${suffix}`,
    "sticky-git-key": `${repository}-cargo-git-${suffix}`,
    "sticky-target-key": `${repository}-${role}-target-${suffix}`,
    secondary: String(Boolean(context.secondaryPath)),
  };
  if (context.secondaryPath) {
    const secondaryPath = targetPath(context.secondaryPath, workspace);
    if (
      secondaryPath === primaryPath ||
      secondaryPath.startsWith(`${primaryPath}${sep}`) ||
      primaryPath.startsWith(`${secondaryPath}${sep}`)
    ) {
      throw new Error("Target paths must not overlap");
    }
    const secondaryRole = requireToken(context.secondaryRole, "secondary role");
    const secondarySuffix = requireToken(context.secondarySuffix, "secondary suffix");
    // A separate benchmark checkout owns its lock/toolchain; other extra target
    // directories use this checkout's metadata. Never restore into a mounted path.
    const parent = dirname(secondaryPath);
    const secondaryFingerprint = fingerprint(parent, cwd);
    const secondary = `rust-cache-v1-${repository}-${runnerOs}-${runnerArch}-${secondarySuffix}-${secondaryFingerprint}-${secondaryRole}-target-`;
    output["secondary-key"] = `${secondary}${checkoutSha}`;
    output["secondary-restore"] = secondary;
    output["secondary-path"] = secondaryPath;
    output["sticky-secondary-key"] = `${repository}-${secondaryRole}-target-${secondarySuffix}`;
  } else if (context.secondaryRole || context.secondarySuffix) {
    throw new Error("Secondary cache metadata requires a target path");
  }
  if (Object.entries(output).some(([key, value]) => key.endsWith("key") && value.length > 512)) {
    throw new Error("Cache key exceeds the Actions limit");
  }
  return output;
}

export function rustCacheMountState(
  paths,
  {
    home = homedir(),
    probe = (path) =>
      spawnSync("mountpoint", ["-q", path], { stdio: "ignore", shell: false }).status === 0,
  } = {},
) {
  return Object.fromEntries(
    [
      ["registry", resolve(home, ".cargo/registry")],
      ["git", resolve(home, ".cargo/git")],
      ["target", paths.target],
      ["secondary", paths.secondary],
    ].map(([key, path]) => [key, String(Boolean(path) && probe(path))]),
  );
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const env = process.env;
  const emit = (plan) =>
    appendFileSync(
      env.GITHUB_OUTPUT,
      Object.entries(plan)
        .map(([key, value]) => `${key}=${value}\n`)
        .join(""),
    );
  if (process.argv[2] === "--mounted") {
    emit(
      rustCacheMountState({
        target: env.VIZE_CACHE_TARGET_PATH,
        secondary: env.VIZE_CACHE_SECONDARY_PATH,
      }),
    );
  } else {
    if (process.argv[2]) throw new Error("Unknown cache policy mode");
    const cwd = env.VIZE_CACHE_SOURCE_ROOT ?? sourceRoot;
    const context = {
      eventName: env.GITHUB_EVENT_NAME,
      event: JSON.parse(readFileSync(env.GITHUB_EVENT_PATH, "utf8")),
      ref: env.GITHUB_REF,
      repository: env.GITHUB_REPOSITORY,
      sourceSha: env.GITHUB_SHA,
      checkoutSha: execFileSync("git", ["rev-parse", "HEAD"], { cwd, encoding: "utf8" }).trim(),
      runnerEnvironment: env.VIZE_CACHE_RUNNER_ENVIRONMENT,
      runnerOs: env.VIZE_CACHE_RUNNER_OS,
      runnerArch: env.VIZE_CACHE_RUNNER_ARCH,
      role: env.VIZE_CACHE_ROLE,
      suffix: env.VIZE_CACHE_SUFFIX,
      targetPath: env.VIZE_CACHE_TARGET_PATH,
      secondaryRole: env.VIZE_CACHE_SECONDARY_ROLE,
      secondarySuffix: env.VIZE_CACHE_SECONDARY_SUFFIX,
      secondaryPath: env.VIZE_CACHE_SECONDARY_PATH,
    };
    const plan = rustCachePolicy(context, { cwd, workspace: env.GITHUB_WORKSPACE ?? cwd });
    emit(plan);
    process.stdout.write(
      `Rust cache: ${plan.sticky === "true" ? "stable sticky + Actions seed" : "Actions cache"}.\n`,
    );
  }
}
