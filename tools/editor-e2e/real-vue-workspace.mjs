#!/usr/bin/env node
// Shared setup for the editor end-to-end scenarios that drive a real
// `vize lsp` process (#3457). The VS Code extension host and the headless
// Neovim scenario both need the exact same workspace: the `real-vue` fixture,
// a `node_modules/vue` symlink so the type checker sees real Vue types, and a
// `vize.config.json` that pins the Corsa/tsgo executable instead of relying on
// ambient discovery.
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repositoryRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
export const vscodeExtensionPath = path.join(repositoryRoot, "editors", "vscode");
export const realVueFixturePath = path.join(
  vscodeExtensionPath,
  "test-fixtures",
  "extension-host",
  "real-vue",
);

/**
 * Materialize the `real-vue` fixture at `workspacePath` and wire it up for the
 * real server. Returns the workspace path so callers can chain.
 */
export function prepareRealVueWorkspace(workspacePath) {
  fs.rmSync(workspacePath, { force: true, recursive: true });
  fs.mkdirSync(path.dirname(workspacePath), { recursive: true });
  fs.cpSync(realVueFixturePath, workspacePath, { recursive: true });
  fs.mkdirSync(path.join(workspacePath, "node_modules"), { recursive: true });
  fs.symlinkSync(
    resolveVuePackagePath(),
    path.join(workspacePath, "node_modules", "vue"),
    "junction",
  );
  fs.writeFileSync(
    path.join(workspacePath, "vize.config.json"),
    `${JSON.stringify({ typeChecker: { corsaPath: resolveCorsaPath() } }, null, 2)}\n`,
  );

  return workspacePath;
}

/**
 * The real `vize` binary under test. CI builds it with `--profile ci` and
 * points `VIZE_SERVER_PATH` at the produced artifact; local runs fall back to
 * the conventional cargo target directories.
 */
export function resolveRealServerPath() {
  const exeName = process.platform === "win32" ? "vize.exe" : "vize";
  const configured = process.env.VIZE_SERVER_PATH?.trim();
  const candidates = configured
    ? [configured]
    : ["ci", "release", "debug"].map((profile) =>
        path.join(repositoryRoot, "target", profile, exeName),
      );

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    `missing real vize server binary (checked ${candidates.join(", ")}). ` +
      "Build one with `cargo build --profile ci -p vize` or set VIZE_SERVER_PATH.",
  );
}

/**
 * Pin the Corsa/tsgo executable that ships with the VS Code extension's own
 * `@typescript/native-preview` install so `vize.config.json` never depends on
 * ambient discovery. Resolution mirrors Node: read the meta package, then
 * resolve the platform package from the meta package's real location.
 */
export function resolveCorsaPath() {
  const extensionRequire = createRequire(path.join(vscodeExtensionPath, "package.json"));
  const metaManifestPath = fs.realpathSync(
    extensionRequire.resolve("@typescript/native-preview/package.json"),
  );
  const platformPackage = `@typescript/native-preview-${process.platform}-${process.arch}`;
  const platformManifestPath = createRequire(metaManifestPath).resolve(
    `${platformPackage}/package.json`,
  );
  const tsgoPath = path.join(
    path.dirname(platformManifestPath),
    "lib",
    process.platform === "win32" ? "tsgo.exe" : "tsgo",
  );

  if (!fs.existsSync(tsgoPath)) {
    throw new Error(`missing tsgo binary for ${platformPackage}: ${tsgoPath}`);
  }

  return tsgoPath;
}

/**
 * The fixture workspace symlinks `node_modules/vue` to the repository's
 * installed Vue package, so the real type checker sees the same Vue type
 * surface as the app fixtures. pnpm keeps the package's dependencies next to
 * its real path, which keeps `@vue/*` resolution working through the symlink.
 */
export function resolveVuePackagePath() {
  const testsRequire = createRequire(path.join(repositoryRoot, "tests", "package.json"));

  try {
    return path.dirname(fs.realpathSync(testsRequire.resolve("vue/package.json")));
  } catch (error) {
    throw new Error(
      "could not resolve the repository vue install for the fixture workspace. " +
        `Run \`vp install\` at the repository root first. (${String(error)})`,
    );
  }
}
