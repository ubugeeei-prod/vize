import { spawnSync } from "node:child_process";
import path from "node:path";
import { root } from "./paths.ts";

/**
 * Resolves the fastest available way to launch `vize lsp` for smoke tests.
 *
 * A CI-profile binary wins when it is already present, then a release binary,
 * then a debug binary, then a globally installed CLI. Falling back to Cargo
 * keeps fresh checkouts usable at the cost of a slower first run, which is
 * acceptable for CI coverage.
 */
export function resolveVizeLaunchCommand(): string[] {
  const candidates = [
    [path.join(root, "target/ci/vize"), "lsp"],
    [path.join(root, "target/release/vize"), "lsp"],
    [path.join(root, "target/debug/vize"), "lsp"],
    ["vize", "lsp"],
  ];

  for (const candidate of candidates) {
    const probe = spawnSync(candidate[0], ["--version"], {
      cwd: root,
      encoding: "utf8",
    });
    if (probe.status === 0) {
      return candidate;
    }
  }

  return ["cargo", "run", "-q", "-p", "vize", "--", "lsp"];
}
