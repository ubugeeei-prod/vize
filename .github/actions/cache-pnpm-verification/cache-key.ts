import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync, existsSync, readFileSync } from "node:fs";
import { isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { bootstrapCacheWrite } from "../cache-vp-bootstrap/cache-key.ts";

const sourceRoot = fileURLToPath(new URL("../../../", import.meta.url));
// pnpm 12.1.0 owns the verification record and policy comparison:
// https://github.com/pnpm/pnpm/tree/6d90c71efdffbc909b499490b64c66badc720327/pnpm/crates/lockfile-verification
const policyFields = new Set([
  "minimumReleaseAge",
  "minimumReleaseAgeExclude",
  "minimumReleaseAgeIgnoreMissingTime",
  "minimumReleaseAgeStrict",
  "trustPolicy",
  "trustPolicyExclude",
  "trustPolicyIgnoreAfter",
  "trustLockfile",
  "registrySupportsTimeField",
  "verifyStoreIntegrity",
  "strictStorePkgContentCheck",
  "blockExoticSubdeps",
]);
// These settings do not change a frozen lockfile's policy verification. Their
// committed definitions are still covered by the exact workspace/manifest hash.
const installFields = new Set([
  "allowBuilds",
  "catalogs",
  "overrides",
  "packages",
  "patchedDependencies",
  "peerDependencyRules",
  "cacheDir",
  "storeDir",
  "userAgent",
]);
const credentialField = /^(?:\/\/[^\r\n]+:)?(?:_auth|_authToken|username|_password)$/;
const scopedRegistry = /^@[^\s:]+:registry$/;

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function registryUrl(value: unknown): boolean {
  if (typeof value !== "string" || /[\r\n\0]/.test(value)) return false;
  try {
    const url = new URL(value);
    return (
      ["http:", "https:"].includes(url.protocol) &&
      !url.username &&
      !url.password &&
      !url.search &&
      !url.hash
    );
  } catch {
    return false;
  }
}

function registryDeclarations(value: unknown): boolean {
  if (!record(value)) return false;
  return Object.entries(value).every(([url, declaration]) => {
    if (!registryUrl(url) || !record(declaration)) return false;
    return Object.entries(declaration).every(([key, field]) => {
      switch (key) {
        case "scopes":
          return (
            Array.isArray(field) &&
            field.every((scope) => typeof scope === "string" && /^@[^\s:]*$/.test(scope))
          );
        case "prefix":
        case "serverType":
          return typeof field === "string" && /^[a-zA-Z0-9_-]+$/.test(field);
        case "supportsTimeField":
          return typeof field === "boolean";
        default:
          return false;
      }
    });
  });
}

function policyValue(key: string, value: unknown): boolean {
  if (key.endsWith("Exclude"))
    return Array.isArray(value) && value.every((item) => typeof item === "string");
  if (key === "minimumReleaseAge" || key === "trustPolicyIgnoreAfter")
    return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
  if (key === "trustPolicy") return value === "off" || value === "no-downgrade";
  return typeof value === "boolean";
}

function canonical(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonical);
  if (!record(value)) return value;
  return Object.fromEntries(
    Object.keys(value)
      .sort()
      .map((key) => [key, canonical(value[key])]),
  );
}

export type VerificationCache = { key: string; path: string };

/** Unknown config falls back to pnpm's normal online verification, never a bypass. */
export function verificationCacheIdentity(
  root: string,
  version: string,
  cacheDirectory: string,
  config: unknown,
  environment: NodeJS.ProcessEnv = {},
): VerificationCache | undefined {
  if (
    !/^\d+\.\d+\.\d+$/.test(version) ||
    !isAbsolute(cacheDirectory) ||
    /[\r\n\0*?[\]{}!]/.test(cacheDirectory) ||
    !record(config) ||
    config.trustLockfile === true ||
    environment.NPM_CONFIG_WORKSPACE_DIR ||
    environment.npm_config_workspace_dir ||
    [".pnpmfile.cjs", ".pnpmfile.mjs"].some((file) => existsSync(join(root, file)))
  )
    return undefined;

  const policy: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(config)) {
    if (policyFields.has(key)) {
      if (!policyValue(key, value)) return undefined;
      policy[key] = value;
    } else if (key === "registry" || scopedRegistry.test(key)) {
      if (!registryUrl(value)) return undefined;
      policy[key] = value;
    } else if (key === "registries") {
      if (!registryDeclarations(value)) return undefined;
      policy[key] = value;
    } else if (!installFields.has(key) && !credentialField.test(key)) {
      // Includes pnpmfile/globalPnpmfile/configDependencies, lockfileDir and
      // branch lockfiles: config list cannot prove their install-time graph.
      return undefined;
    }
  }
  if (!registryUrl(policy.registry) || !record(policy.registries)) return undefined;

  const manifest = readFileSync(join(root, "package.json"), "utf8");
  if (JSON.parse(manifest).packageManager !== `pnpm@${version}`) return undefined;
  const hash = createHash("sha256").update(
    JSON.stringify([
      version,
      manifest,
      readFileSync(join(root, "pnpm-workspace.yaml"), "utf8"),
      readFileSync(join(root, "pnpm-lock.yaml"), "utf8"),
      canonical(policy),
    ]),
  );
  return {
    key: `pnpm-verification-v1-${version}-${hash.digest("hex")}`,
    path: join(cacheDirectory, "lockfile-verified.jsonl"),
  };
}

export function verificationCacheWrite(
  requested: boolean,
  event: string | undefined,
  ref: string | undefined,
  defaultBranch: string | undefined,
  expectedSha: string | undefined,
  checkoutSha: string,
): boolean {
  return (
    bootstrapCacheWrite(requested, event, ref, defaultBranch) &&
    /^[0-9a-f]{40}$/.test(expectedSha ?? "") &&
    expectedSha === checkoutSha
  );
}

export function cachePnpmVerification() {
  const output = process.env.GITHUB_OUTPUT;
  if (!output) throw new Error("GITHUB_OUTPUT is unset");
  let identity: VerificationCache | undefined;
  let save = false;
  try {
    const query = (args: string[]) =>
      execFileSync("vp", ["env", "exec", "pnpm", ...args], {
        cwd: sourceRoot,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      }).trim();
    identity = verificationCacheIdentity(
      sourceRoot,
      query(["--version"]),
      query(["cache", "path"]),
      JSON.parse(query(["config", "list", "--json"])),
      process.env,
    );
    if (identity && process.env.SAVE_REQUESTED === "true")
      save = verificationCacheWrite(
        true,
        process.env.GITHUB_EVENT_NAME,
        process.env.GITHUB_REF,
        process.env.DEFAULT_BRANCH,
        process.env.GITHUB_SHA,
        execFileSync("git", ["rev-parse", "HEAD"], { cwd: sourceRoot, encoding: "utf8" }).trim(),
      );
  } catch {
    // Config/subprocess errors can carry credentials. Never print their values.
    identity = undefined;
  }
  appendFileSync(
    output,
    `key=${identity?.key ?? ""}\npath=${identity?.path ?? ""}\nsave=${save}\n`,
  );
  process.stdout.write(
    identity
      ? `pnpm verification cache ${identity.key}; save=${save}\n`
      : "pnpm verification cache unavailable; normal install verification remains enabled\n",
  );
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  cachePnpmVerification();
