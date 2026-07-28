#!/usr/bin/env node
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runTests } from "@vscode/test-electron";

const extensionDevelopmentPath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryRoot = path.resolve(extensionDevelopmentPath, "..", "..");
const fixtureWorkspacePath = path.join(
  extensionDevelopmentPath,
  "test-fixtures",
  "extension-host",
  "real-vue",
);
const testDataPath = path.join(extensionDevelopmentPath, ".vscode-test", "host-smoke-real");
const workspacePath = path.join(testDataPath, "workspaces", "real-vue");
const extensionTestsPath = path.join(
  extensionDevelopmentPath,
  "test",
  "suite",
  "extension-host-real.cjs",
);

const serverPath = resolveRealServerPath();
const corsaPath = resolveCorsaPath();
const vuePackagePath = resolveVuePackagePath();

fs.rmSync(testDataPath, { force: true, recursive: true });
fs.cpSync(fixtureWorkspacePath, workspacePath, { recursive: true });
fs.mkdirSync(path.join(workspacePath, "node_modules"), { recursive: true });
fs.symlinkSync(vuePackagePath, path.join(workspacePath, "node_modules", "vue"), "junction");
fs.writeFileSync(
  path.join(workspacePath, "vize.config.json"),
  `${JSON.stringify({ typeChecker: { corsaPath } }, null, 2)}\n`,
);

// macOS caps AF_UNIX socket paths at 104 bytes, and VS Code creates its
// singleton socket inside the user-data directory. Deep checkouts overflow the
// default `.vscode-test/user-data` location, so keep user data in a short
// temporary directory instead.
const userDataPath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-host-real-"));

try {
  await runTests({
    extensionDevelopmentPath,
    extensionTestsPath,
    extensionTestsEnv: {
      VIZE_TEST_SERVER_PATH: serverPath,
    },
    launchArgs: [
      "--disable-extensions",
      "--disable-workspace-trust",
      "--skip-welcome",
      "--skip-release-notes",
      `--user-data-dir=${userDataPath}`,
      workspacePath,
    ],
  });
} finally {
  fs.rmSync(userDataPath, { force: true, recursive: true });
}

/**
 * The real `vize` binary under test. CI builds it with `--profile ci` and
 * points `VIZE_SERVER_PATH` at the produced artifact; local runs fall back to
 * the conventional cargo target directories.
 */
function resolveRealServerPath() {
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
 * Pin the Corsa/tsgo executable that ships with the extension's own
 * `@typescript/native-preview` install so `vize.config.json` never depends on
 * ambient discovery. Resolution mirrors Node: read the meta package, then
 * resolve the platform package from the meta package's real location.
 */
function resolveCorsaPath() {
  const extensionRequire = createRequire(path.join(extensionDevelopmentPath, "package.json"));
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
function resolveVuePackagePath() {
  const testsRequire = createRequire(path.join(repositoryRoot, "tests", "package.json"));

  try {
    return path.dirname(fs.realpathSync(testsRequire.resolve("vue/package.json")));
  } catch (error) {
    throw new Error(
      "could not resolve the repository vue install for the fixture workspace. " +
        `Run \`vp install\` at the repository root first. (${error})`,
    );
  }
}
