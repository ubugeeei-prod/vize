import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

import { execNpxCommand } from "./app-fixture-runtime.ts";

/** Bind the real executable, rather than a version-manager shim. */
export function inspectPrepareNode(binary: string, env: NodeJS.ProcessEnv) {
  assert.ok(path.isAbsolute(binary), "preparation Node must be an absolute executable path");
  const executable = fs.realpathSync(binary);
  assert.ok(fs.statSync(executable).isFile(), "preparation Node must be a regular file");
  const bin = path.dirname(executable);
  const pathKey = Object.keys(env).find((key) => key.toLowerCase() === "path") ?? "PATH";
  const scoped = { ...env, [pathKey]: `${bin}${path.delimiter}${env[pathKey] ?? ""}` };
  const actual: { execPath: string; version: string } = JSON.parse(
    execFileSync(
      executable,
      ["-p", "JSON.stringify({execPath:process.execPath,version:process.version})"],
      { env: scoped, encoding: "utf8", timeout: 10_000 },
    ),
  );
  assert.equal(
    fs.realpathSync(actual.execPath),
    executable,
    "preparation Node shim changed runtime",
  );
  assert.match(actual.version, /^v\d+\.\d+\.\d+$/);
  scoped.VP_NODE_VERSION = actual.version.slice(1);
  scoped.VIZE_APP_CHECK_PREPARE_NODE = executable;
  return {
    env: scoped,
    observation: {
      executable,
      version: actual.version,
      sha256: createHash("sha256").update(fs.readFileSync(executable)).digest("hex"),
    },
  };
}

/** npm adds ancestor .bin entries before PATH; bind the runtime after that step. */
export function execNuxtPrepareNpx(
  args: string[],
  options: { cwd: string; env: NodeJS.ProcessEnv; timeout: number },
): void {
  const executable = options.env.VIZE_APP_CHECK_PREPARE_NODE;
  if (!executable) return execNpxCommand(args, options);
  const distribution = inspectPrepareNpx(executable);
  const windows = process.platform === "win32";
  const npxEntry = windows
    ? path.join(path.dirname(executable), "node_modules/npm/bin/npx-cli.js")
    : distribution.npxEntry;
  execBoundPrepareNpx(executable, npxEntry, args, options);
}

/** Also supports Actions' bare Node distribution with an authenticated npm CLI. */
export function execBoundPrepareNpx(
  executable: string,
  npxEntry: string,
  args: string[],
  options: { cwd: string; env: NodeJS.ProcessEnv; timeout: number },
): void {
  const windows = process.platform === "win32";
  assert.ok(fs.statSync(npxEntry).isFile(), "preparation distribution must own the npm CLI");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-command-shell-"));
  const shell = path.join(directory, windows ? "runtime.cmd" : "runtime.sh");
  fs.writeFileSync(
    shell,
    windows
      ? '@echo off\r\nset "PATH=%VIZE_APP_CHECK_PREPARE_BIN%;%PATH%"\r\n"%ComSpec%" /d /s /c %~2\r\n'
      : '#!/bin/sh\nPATH="$VIZE_APP_CHECK_PREPARE_BIN:$PATH"\nexport PATH\nexec /bin/sh "$@"\n',
    { mode: 0o755 },
  );
  try {
    execFileSync(executable, [npxEntry, "--script-shell", shell, ...args], {
      ...options,
      env: { ...options.env, VIZE_APP_CHECK_PREPARE_BIN: path.dirname(executable) },
      stdio: "inherit",
    });
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

/** Selected preparation runtimes must own the adjacent package-manager entry. */
export function inspectPrepareNpx(executable: string) {
  const npx = path.join(path.dirname(executable), process.platform === "win32" ? "npx.cmd" : "npx");
  const npxEntry = fs.realpathSync(npx);
  assert.ok(fs.statSync(npxEntry).isFile(), "preparation Node distribution must own npx");
  return {
    npx,
    npxEntry,
    npxSha256: createHash("sha256").update(fs.readFileSync(npxEntry)).digest("hex"),
  };
}

/** Only the pinned fixture's exact Node 22 contract selects this side runtime. */
export function selectNuxtPrepareRuntime(authoredEngine: unknown, env: NodeJS.ProcessEnv) {
  if (authoredEngine !== "22") {
    return { env, observation: { authoredEngine, node22Selected: false } };
  }
  const binary =
    env.VIZE_APP_CHECK_PREPARE_NODE22 ??
    (process.versions.node.startsWith("22.") ? process.execPath : undefined);
  assert.ok(
    binary,
    "authored Node 22 preparation requires VIZE_APP_CHECK_PREPARE_NODE22 from App runtime setup",
  );
  const selected = inspectPrepareNode(binary, env);
  assert.match(selected.observation.version, /^v22\./, "authored preparation requires Node 22");
  return {
    env: selected.env,
    observation: {
      authoredEngine,
      node22Selected: true,
      ...selected.observation,
      ...inspectPrepareNpx(selected.observation.executable),
    },
  };
}
