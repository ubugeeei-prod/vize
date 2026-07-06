import { spawnSync } from "node:child_process";
import path from "node:path";
import { root } from "./paths.ts";

/**
 * Resolves the fastest available way to launch `vize lsp` for smoke tests.
 *
 * A caller-provided binary wins first, then the checkout's debug binary, then
 * CI/release artifacts, then a globally installed CLI. Preferring the debug
 * binary keeps local and workflow tests tied to the code that was just built
 * instead of a stale `target/ci/vize` left by a previous job.
 */
export function resolveVizeLaunchCommand(): string[] {
  const envBinary = process.env.VIZE_LSP_BIN;
  const candidates = [
    ...(envBinary ? [[envBinary, "lsp"]] : []),
    [path.join(root, "target/debug/vize"), "lsp"],
    [path.join(root, "target/ci/vize"), "lsp"],
    [path.join(root, "target/release/vize"), "lsp"],
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
