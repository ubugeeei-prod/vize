import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const nativePackageDir = fileURLToPath(new URL("../../../npm/native/", import.meta.url));
export const packageVersion = "1.0.0";

export function run(
  command: string,
  args: string[],
  options: { cwd: string; env?: NodeJS.ProcessEnv },
): { status: number | null; stdout: string; stderr: string } {
  const isWindowsBatch = process.platform === "win32" && /\.(cmd|bat)$/i.test(command);
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: "utf8",
    env: options.env ?? process.env,
    shell: isWindowsBatch,
  });

  if (result.error != null) {
    throw result.error;
  }

  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

export function assertSucceeded(
  result: { status: number | null; stdout: string; stderr: string },
  command: string,
): void {
  assert.equal(result.status, 0, `${command}\n${result.stderr}\n${result.stdout}`.trim());
}

export function currentTargetPackageName(): string {
  if (process.platform === "darwin") {
    return "@vizejs/native-darwin-universal";
  }
  if (process.platform === "win32" && (process.arch === "x64" || process.arch === "arm64")) {
    return `@vizejs/native-win32-${process.arch}-msvc`;
  }
  if (process.platform === "linux" && (process.arch === "x64" || process.arch === "arm64")) {
    const report = process.report?.getReport() as
      | { header?: { glibcVersionRuntime?: unknown } }
      | undefined;
    const isGlibc = typeof report?.header?.glibcVersionRuntime === "string";
    return `@vizejs/native-linux-${process.arch}-${isGlibc ? "gnu" : "musl"}`;
  }
  throw new Error(`Unsupported native loader test host: ${process.platform}-${process.arch}`);
}
function copyPackableNativePackage(destination: string): void {
  fs.mkdirSync(destination, { recursive: true });
  const manifest = JSON.parse(
    fs.readFileSync(path.join(nativePackageDir, "package.json"), "utf8"),
  ) as {
    files: string[];
    version: string;
    optionalDependencies?: object;
    devDependencies?: object;
    scripts?: object;
  };
  for (const file of [...manifest.files, "README.md"]) {
    const target = path.join(destination, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(nativePackageDir, file), target);
  }
  manifest.version = packageVersion;
  manifest.optionalDependencies = {};
  manifest.devDependencies = {};
  manifest.scripts = {};
  fs.writeFileSync(
    path.join(destination, "package.json"),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
}

export function installPackedNativePackage(
  tempDir: string,
  env: NodeJS.ProcessEnv = process.env,
): string {
  const packageSourceDir = path.join(tempDir, "package-source");
  const packDir = path.join(tempDir, "packed");
  const installDir = path.join(tempDir, "install");
  const npmCacheDir = path.join(tempDir, "npm-cache");
  fs.mkdirSync(packDir, { recursive: true });
  fs.mkdirSync(installDir, { recursive: true });
  copyPackableNativePackage(packageSourceDir);

  const npm = process.env.NPM_BIN ?? (process.platform === "win32" ? "npm.cmd" : "npm");
  const npmEnv = { ...env, npm_config_cache: npmCacheDir };
  const pack = run(npm, ["pack", "--ignore-scripts", "--pack-destination", packDir], {
    cwd: packageSourceDir,
    env: npmEnv,
  });
  assertSucceeded(pack, "npm pack");
  const tarballs = fs.readdirSync(packDir).filter((entry) => entry.endsWith(".tgz"));
  assert.equal(
    tarballs.length,
    1,
    `expected one native package tarball, got ${tarballs.join(", ")}`,
  );

  fs.writeFileSync(
    path.join(installDir, "package.json"),
    `${JSON.stringify({ name: "native-loader-smoke", private: true, version: "1.0.0" }, null, 2)}\n`,
  );
  const tarballPath = path.join(packDir, tarballs[0]);
  const install = run(
    npm,
    [
      "install",
      "--ignore-scripts",
      "--no-save",
      "--omit=optional",
      "--no-audit",
      "--no-fund",
      tarballPath,
    ],
    { cwd: installDir, env: npmEnv },
  );
  assertSucceeded(install, "npm install <packed @vizejs/native>");

  return installDir;
}

export function installNativeTarget(
  installDir: string,
  program: string,
  version = packageVersion,
): void {
  const name = currentTargetPackageName();
  const target = path.join(installDir, "node_modules", ...name.split("/"));
  fs.mkdirSync(target, { recursive: true });
  fs.writeFileSync(
    path.join(target, "package.json"),
    JSON.stringify({ name, main: "index.js", version }),
  );
  fs.copyFileSync(program, path.join(target, "index.js"));
}

export function runNativeLoaderProbe(
  installDir: string,
  program: string,
  args: string[] = [],
  env: NodeJS.ProcessEnv = process.env,
) {
  // Node resolves the simulated consumer's dependencies relative to its entry point.
  const entrypoint = path.basename(program);
  fs.copyFileSync(program, path.join(installDir, entrypoint));
  return run(process.execPath, [entrypoint, ...args], { cwd: installDir, env });
}
