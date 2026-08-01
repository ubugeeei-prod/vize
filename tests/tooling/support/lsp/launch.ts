import { spawnSync } from "node:child_process";
import path from "node:path";
import { root } from "./paths.ts";

/**
 * Resolves the fastest available way to launch `vize lsp` for smoke tests.
 *
 * A caller-provided binary wins first, then the checkout's debug binary, then
 * CI/release artifacts. If none exist, build and run the current workspace;
 * a globally installed CLI may exercise unrelated code and invalidate the
 * regression that the test is meant to cover.
 */
export function resolveVizeLaunchCommand(
  canLaunch: (command: string) => boolean = (command) =>
    spawnSync(command, ["--version"], {
      cwd: root,
      encoding: "utf8",
    }).status === 0,
  envBinary = process.env.VIZE_LSP_BIN,
): string[] {
  const candidates = [
    ...(envBinary ? [[envBinary, "lsp"]] : []),
    [path.join(root, "target/debug/vize"), "lsp"],
    [path.join(root, "target/ci/vize"), "lsp"],
    [path.join(root, "target/release/vize"), "lsp"],
  ];

  for (const candidate of candidates) {
    if (canLaunch(candidate[0])) {
      return candidate;
    }
  }

  return ["cargo", "run", "-q", "-p", "vize", "--", "lsp"];
}
