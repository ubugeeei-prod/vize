import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  BUILD_RECIPE,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../differential/build-receipt.ts";
import { root } from "./paths.ts";

export type VerifiedLspLaunch = {
  binary: string;
  expected: ReturnType<typeof expectedBuildIdentity>;
  receipt: Record<string, unknown>;
  buildRecipe?: string;
  versionProbe?: {
    exitStatus: number | null;
    signal: string | null;
    stdoutBase64: string;
    stderrBase64: string;
    processError: string | null;
  };
};

/**
 * Resolves the fastest available way to launch `vize lsp` for smoke tests.
 *
 * A caller-provided binary wins first, then the checkout's debug binary, then
 * CI/release artifacts. If none exist, build and run the current workspace;
 * a globally installed CLI may exercise unrelated code and invalidate the
 * regression that the test is meant to cover.
 * Full tooling requires the receipted CI executable and rejects every identity
 * or launch failure before trying another candidate. Without that receipt, a
 * checkout binary older than the Rust sources is skipped: a stale build would
 * report false failures or false passes for the current source (#3957).
 */
export function resolveVizeLaunchCommand(
  canLaunch?: (command: string) => boolean,
  envBinary = process.env.VIZE_LSP_BIN,
  sourceBinding: {
    required?: boolean;
    repoRoot?: string;
    /** Whether a checkout binary predates the sources; injectable for tests. */
    isStale?: (binary: string) => boolean;
    buildRecipe?: string;
    onVerifiedLaunch?: (launch: VerifiedLspLaunch) => void;
  } = {},
): string[] {
  let versionProbe: VerifiedLspLaunch["versionProbe"];
  const probe = (command: string): boolean => {
    if (canLaunch) return canLaunch(command);
    const result = spawnSync(command, ["--version"], {
      cwd: sourceBinding.repoRoot ?? root,
    });
    versionProbe = {
      exitStatus: result.status,
      signal: result.signal,
      stdoutBase64: result.stdout?.toString("base64") ?? "",
      stderrBase64: result.stderr?.toString("base64") ?? "",
      processError: result.error?.message ?? null,
    };
    return result.status === 0;
  };
  if (sourceBinding.required ?? process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD === "1") {
    const repoRoot = sourceBinding.repoRoot ?? root;
    assert.ok(envBinary, "VIZE_LSP_BIN is required for source-built LSP validation");
    const binary = path.resolve(repoRoot, envBinary);
    const expected = expectedBuildIdentity(repoRoot);
    assert.equal(binary, path.resolve(repoRoot, expected.binaryPath));
    const receipt = JSON.parse(
      fs.readFileSync(`${binary}.differential-build.json`, "utf8"),
    ) as Record<string, unknown>;
    const buildRecipe =
      sourceBinding.buildRecipe ?? process.env.VIZE_LSP_BUILD_RECIPE ?? BUILD_RECIPE;
    validateBuildReceipt(receipt, expected, buildRecipe);
    assert.ok(probe(binary), "the verified source-built LSP binary cannot launch");
    sourceBinding.onVerifiedLaunch?.({ binary, expected, receipt, buildRecipe, versionProbe });
    return [binary, "lsp"];
  }
  const candidates = [
    ...(envBinary ? [[envBinary, "lsp"]] : []),
    [path.join(root, "target/debug/vize"), "lsp"],
    [path.join(root, "target/ci/vize"), "lsp"],
    [path.join(root, "target/release/vize"), "lsp"],
  ];

  const isStale =
    sourceBinding.isStale ??
    ((binary: string) =>
      isStaleLocalBuild(
        fs.statSync(binary, { throwIfNoEntry: false })?.mtimeMs,
        newestSourceMtimeMs(sourceBinding.repoRoot ?? root),
      ));
  for (const candidate of candidates) {
    const local = candidate[0] !== envBinary;
    if (local && isStale(candidate[0])) {
      process.stderr.write(`skipping ${candidate[0]}: older than the Rust sources\n`);
      continue;
    }
    if (probe(candidate[0])) {
      return candidate;
    }
  }

  return ["cargo", "run", "-q", "-p", "vize", "--", "lsp"];
}

/** A build is stale when it predates the newest compiled source. */
export function isStaleLocalBuild(
  binaryMtimeMs: number | undefined,
  sourcesMtimeMs: number,
): boolean {
  return binaryMtimeMs !== undefined && binaryMtimeMs < sourcesMtimeMs;
}

const newestSourceByRoot = new Map<string, number>();

/** Newest mtime of the crates' Rust sources and manifests, once per process. */
function newestSourceMtimeMs(repoRoot: string): number {
  const cached = newestSourceByRoot.get(repoRoot);
  if (cached !== undefined) return cached;
  let newest =
    fs.statSync(path.join(repoRoot, "Cargo.lock"), { throwIfNoEntry: false })?.mtimeMs ?? 0;
  const stack = [path.join(repoRoot, "crates")];
  while (stack.length > 0) {
    const dir = stack.pop()!;
    let entries: fs.Dirent[];
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      continue;
    }
    for (const entry of entries) {
      const entryPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        if (entry.name !== "target" && entry.name !== "node_modules") stack.push(entryPath);
      } else if (entry.name.endsWith(".rs") || entry.name === "Cargo.toml") {
        newest = Math.max(newest, fs.statSync(entryPath).mtimeMs);
      }
    }
  }
  newestSourceByRoot.set(repoRoot, newest);
  return newest;
}
