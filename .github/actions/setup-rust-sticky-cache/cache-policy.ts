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
// Candidates mount a clone of the trusted disk that the provider never commits.
const readOnlyEvents = new Set(["pull_request", "merge_group"]);
// Actual trusted uploads exceeded the backend's 25 GB entry limit.
const restoreOnlyTargetRoles = new Set(["coverage-source", "clippy-test"]);
// Non-release roles whose manual dispatch may mount the trusted disk read-only.
const dispatchRestoreRoles = new Set(["coverage-source"]);
const fullSha = /^[0-9a-f]{40}$/;
const token = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;
const object = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);

export interface RustCacheContext {
  eventName?: string;
  event?: unknown;
  ref?: string;
  repository?: string;
  sourceSha?: string;
  checkoutSha?: string;
  runnerEnvironment?: string;
  runnerOs?: string;
  runnerArch?: string;
  role?: string;
  suffix?: string;
  targetPath?: string;
  secondaryRole?: string;
  secondarySuffix?: string;
  secondaryPath?: string;
  restoreTrusted?: boolean;
}

function requireToken(value: unknown, label: string): string {
  if (typeof value !== "string" || !token.test(value)) throw new Error(`Invalid ${label}`);
  return value;
}

function targetPath(value: string, workspace: string) {
  if (typeof value !== "string" || /[\r\n\0]/.test(value)) throw new Error("Invalid target path");
  const path = resolve(workspace, value);
  if (!path.startsWith(`${resolve(workspace)}${sep}`))
    throw new Error("Target must be inside workspace");
  return path;
}

function fingerprint(cwd: string, fallback = cwd) {
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

// A target built by another compiler is never reused, and Cargo never deletes
// it, so each pinned toolchain gets its own disk lineage. Abandoned lineages
// stop growing instead of pushing one disk past what can still be committed.
function toolchainChannel(cwd: string, fallback = cwd) {
  const root = existsSync(resolve(cwd, "rust-toolchain.toml")) ? cwd : fallback;
  const toml = readFileSync(resolve(root, "rust-toolchain.toml"), "utf8");
  const channel = /^channel\s*=\s*"([^"]+)"/m.exec(toml)?.[1] ?? "unpinned";
  return requireToken(channel, "toolchain channel");
}

export function rustCachePolicy(
  context: RustCacheContext,
  { cwd = sourceRoot, workspace = cwd }: { cwd?: string; workspace?: string } = {},
): Record<string, string> {
  const { eventName, event, ref, repository, sourceSha, checkoutSha, runnerEnvironment } = context;
  if (!eventName || !knownEvents.has(eventName) || !object(event) || !object(event.repository)) {
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
    !["github-hosted", "self-hosted"].includes(runnerEnvironment ?? "")
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
  // A validated release workflow or a manual Full coverage run may reuse the
  // default-branch build cache. This opt-in never grants a write or creates a
  // candidate-specific disk. The release builds the version-stamped candidate
  // it committed on top of a main commit, so its checkout never equals the
  // dispatched commit; it runs only from the default branch.
  const dispatchRestore =
    context.restoreTrusted === true &&
    eventName === "workflow_dispatch" &&
    (role.startsWith("release-")
      ? ref === `refs/heads/${branch}`
      : dispatchRestoreRoles.has(role) && sourceSha === checkoutSha);
  const readOnly = !trusted && (readOnlyEvents.has(eventName) || dispatchRestore);
  const sticky = (trusted || readOnly) && runnerEnvironment === "self-hosted";
  const base = `rust-cache-v1-${repository}-${runnerOs}-${runnerArch}-${suffix}-${fingerprint(cwd)}`;
  const target = `${base}-${role}-target-`;
  const output: Record<string, string> = {
    trusted: String(trusted),
    sticky: String(sticky),
    // Only the trusted default-branch checkout may write a stable disk.
    "sticky-commit": String(trusted),
    "registry-key": `${base}-registry`,
    "git-key": `${base}-git`,
    "target-key": `${target}${checkoutSha}`,
    "target-seed-key": `${target}seed`,
    "target-restore": target,
    "target-path": primaryPath,
    "target-save": String(trusted && !restoreOnlyTargetRoles.has(role)),
    "sticky-registry-key": `${repository}-cargo-registry-${suffix}`,
    "sticky-git-key": `${repository}-cargo-git-${suffix}`,
    "sticky-target-key": `${repository}-${role}-target-${suffix}-${toolchainChannel(cwd)}`,
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
    output["secondary-seed-key"] = `${secondary}seed`;
    output["secondary-restore"] = secondary;
    output["secondary-path"] = secondaryPath;
    output["secondary-save"] = String(trusted && !restoreOnlyTargetRoles.has(secondaryRole));
    output["sticky-secondary-key"] =
      `${repository}-${secondaryRole}-target-${secondarySuffix}-${toolchainChannel(parent, cwd)}`;
  } else if (context.secondaryRole || context.secondarySuffix) {
    throw new Error("Secondary cache metadata requires a target path");
  }
  if (Object.entries(output).some(([key, value]) => key.endsWith("key") && value.length > 512)) {
    throw new Error("Cache key exceeds the Actions limit");
  }
  return output;
}

export function rustCacheMountState(
  paths: { target?: string; secondary?: string },
  {
    home = homedir(),
    probe = (path: string) =>
      spawnSync("mountpoint", ["-q", path], { stdio: "ignore", shell: false }).status === 0,
  } = {},
) {
  return Object.fromEntries(
    [
      ["registry", resolve(home, ".cargo/registry")],
      ["git", resolve(home, ".cargo/git")],
      ["target", paths.target],
      ["secondary", paths.secondary],
    ].map(([key, path]) => [key, String(Boolean(path) && probe(path as string))]),
  );
}

export function runCachePolicy(mode?: string) {
  const env = process.env;
  const outputFile = env.GITHUB_OUTPUT;
  if (!outputFile) throw new Error("GITHUB_OUTPUT is unset");
  const emit = (plan: Record<string, string>) =>
    appendFileSync(
      outputFile,
      Object.entries(plan)
        .map(([key, value]) => `${key}=${value}\n`)
        .join(""),
    );
  if (mode === "--mounted") {
    emit(
      rustCacheMountState({
        target: env.VIZE_CACHE_TARGET_PATH,
        secondary: env.VIZE_CACHE_SECONDARY_PATH,
      }),
    );
  } else {
    if (mode) throw new Error("Unknown cache policy mode");
    const cwd = env.VIZE_CACHE_SOURCE_ROOT ?? sourceRoot;
    const context = {
      eventName: env.GITHUB_EVENT_NAME,
      event: JSON.parse(readFileSync(env.GITHUB_EVENT_PATH ?? "", "utf8")),
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
      restoreTrusted: env.VIZE_CACHE_RESTORE_TRUSTED === "true",
    };
    const plan = rustCachePolicy(context, { cwd, workspace: env.GITHUB_WORKSPACE ?? cwd });
    emit(plan);
    process.stdout.write(
      `Rust cache: ${plan.sticky === "true" ? "stable sticky + Actions seed" : "Actions cache"}.\n`,
    );
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  runCachePolicy(process.argv[2]);
