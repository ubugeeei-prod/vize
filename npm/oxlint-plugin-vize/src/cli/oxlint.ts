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
