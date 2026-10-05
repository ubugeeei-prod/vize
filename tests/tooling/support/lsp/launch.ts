import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  BUILD_RECIPE,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../differential/build-receipt.mjs";
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
 * or launch failure before trying another candidate.
 */
export function resolveVizeLaunchCommand(
  canLaunch?: (command: string) => boolean,
  envBinary = process.env.VIZE_LSP_BIN,
  sourceBinding: {
    required?: boolean;
    repoRoot?: string;
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

  for (const candidate of candidates) {
    if (probe(candidate[0])) {
      return candidate;
    }
  }

  return ["cargo", "run", "-q", "-p", "vize", "--", "lsp"];
}
