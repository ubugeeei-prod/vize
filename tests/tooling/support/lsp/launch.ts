import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
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

/**
 * Absolute path to the Corsa/tsgo binary this checkout pins through
 * `@typescript/native-preview`.
 *
 * The server discovers Corsa by walking the workspace root's ancestors, so a
 * real project that installs its own `@typescript/native-preview` wins over the
 * checkout's. Older builds reject flags the bridge sends (misskey pins a build
 * whose `api` subcommand has no `-async`), the bridge then reports "not
 * available", and type diagnostics silently disappear in exactly the workspaces
 * that ship a binary. Sessions that must type check a real workspace pass this
 * path as `CORSA_PATH`, which the resolver honors ahead of its ancestor walk.
 *
 * Resolution mirrors Node: read the meta package, then resolve the platform
 * package from the meta package's real location.
 */
export function resolvePinnedCorsaPath(): string {
  const rootRequire = createRequire(path.join(root, "package.json"));
  const metaManifestPath = fs.realpathSync(
    rootRequire.resolve("@typescript/native-preview/package.json"),
  );
  const metaRequire = createRequire(metaManifestPath);
  const basePackage = `@typescript/native-preview-${process.platform}-${process.arch}`;
  // The platform packages are optional dependencies gated on os/cpu/libc, so a
  // host only ever has one of them installed. On Linux that is either the glibc
  // or the musl build; probe both instead of assuming glibc.
  const platformPackages =
    process.platform === "linux" ? [basePackage, `${basePackage}-musl`] : [basePackage];
  const attempted: string[] = [];

  for (const platformPackage of platformPackages) {
    let platformManifestPath: string;
    try {
      platformManifestPath = metaRequire.resolve(`${platformPackage}/package.json`);
    } catch {
      attempted.push(`${platformPackage} (not installed)`);
      continue;
    }
    const binary = path.join(
      path.dirname(platformManifestPath),
      "lib",
      process.platform === "win32" ? "tsgo.exe" : "tsgo",
    );
    if (fs.existsSync(binary)) return binary;
    attempted.push(binary);
  }

  throw new Error(`missing pinned Corsa binary (checked ${attempted.join(", ")})`);
}
