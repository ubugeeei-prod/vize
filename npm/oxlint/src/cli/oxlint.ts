import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const OXLINT_ENTRYPOINT_SEGMENTS = ["node_modules", "oxlint", "bin", "oxlint"];
const PNPM_OXLINT_ENTRYPOINT_PATTERN = "node_modules/.pnpm/oxlint@*/node_modules/oxlint/bin/oxlint";

export function resolveOxlintCliEntrypoint(cwd: string): string {
  let currentDir = cwd;

  for (;;) {
    const candidate = path.join(currentDir, ...OXLINT_ENTRYPOINT_SEGMENTS);
    if (fs.existsSync(candidate)) {
      return candidate;
    }

    const pnpmCandidate = fs.globSync(PNPM_OXLINT_ENTRYPOINT_PATTERN, {
      cwd: currentDir,
      withFileTypes: false,
    })[0];
    if (pnpmCandidate != null) {
      return path.resolve(currentDir, pnpmCandidate);
    }

    const parentDir = path.dirname(currentDir);
    if (parentDir === currentDir) {
      throw new Error(
        "Unable to locate oxlint. Install `oxlint` in the current workspace before using `oxlint-vize`.",
      );
    }

    currentDir = parentDir;
  }
}

/** Real oxlint prints `Version: <semver>`; any semver-shaped token passes. */
const VERSION_HANDSHAKE_PATTERN = /\b\d+\.\d+\.\d+/u;

/**
 * Confirms the resolved entrypoint actually behaves like the oxlint CLI.
 *
 * Workspaces can place a non-lint wrapper at the oxlint bin path — vite-plus
 * ships an LSP-only shim there, for example. Spawning such a wrapper yields no
 * diagnostics, which the caller would otherwise forward as a clean lint run: a
 * silent false green. A `--version` handshake separates the real CLI from
 * wrappers before any lint output is trusted.
 */
export function verifyOxlintCliEntrypoint(executable: string, entrypoint: string): void {
  const handshake = spawnSync(executable, [entrypoint, "--version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  const output = `${handshake.stdout ?? ""}${handshake.stderr ?? ""}`.trim();

  if (handshake.error == null && handshake.status === 0 && VERSION_HANDSHAKE_PATTERN.test(output)) {
    return;
  }

  const detail =
    handshake.error == null
      ? `exit ${String(handshake.status)}${output === "" ? " with no output" : `: ${output.split("\n", 1)[0]}`}`
      : handshake.error.message;
  throw new Error(
    `The oxlint entrypoint at ${entrypoint} did not answer the \`--version\` handshake (${detail}). ` +
      "It appears to be a non-lint wrapper shim, so its runs cannot be trusted as lint results. " +
      "Install the real `oxlint` package where `oxlint-vize` can resolve it.",
  );
}
