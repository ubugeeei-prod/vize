import { Buffer } from "node:buffer";
import { existsSync } from "node:fs";
import { join } from "node:path";

import { runTypecheckCommand } from "./typecheck-command-runner.mjs";

export function installArguments(
  manager,
  fixtureRoot,
  managerVersion = null,
  { nodeLinker = null, workspacePackages = null } = {},
) {
  const ignoreParentWorkspace =
    manager === "pnpm" && !existsSync(join(fixtureRoot, "pnpm-workspace.yaml"));
  return {
    npm: ["ci", "--ignore-scripts", "--prefer-offline", "--no-audit", "--no-fund"],
    pnpm: [
      "install",
      "--frozen-lockfile",
      "--ignore-scripts",
      "--prefer-offline",
      ...(nodeLinker === "hoisted" ? ["--config.node-linker=hoisted"] : []),
      ...(workspacePackages === "injected"
        ? ["--config.inject-workspace-packages=true", "--config.dedupe-injected-deps=false"]
        : []),
      ...(ignoreParentWorkspace ? ["--ignore-workspace"] : []),
    ],
    yarn:
      typeof managerVersion === "string" && managerVersion.startsWith("1.")
        ? [
            "install",
            "--frozen-lockfile",
            "--ignore-scripts",
            "--ignore-engines",
            "--prefer-offline",
            "--non-interactive",
          ]
        : ["install", "--immutable", "--mode=skip-build"],
  }[manager];
}

/**
 * @param {string} manager
 * @param {Buffer} sourceBytes
 * @param {{ workspacePackages?: string | null }} [options]
 * @returns {Buffer}
 */
export function qualifyLockfileBytes(manager, sourceBytes, { workspacePackages = null } = {}) {
  if (manager !== "pnpm" || workspacePackages !== "injected") return sourceBytes;
  const source = sourceBytes.toString("utf8");
  if (!Buffer.from(source, "utf8").equals(sourceBytes)) {
    throw new Error("pnpm lockfile must be valid UTF-8");
  }
  const settings = /^settings:\r?\n(?:[ \t][^\r\n]*(?:\r?\n|$))*/mu.exec(source)?.[0];
  if (settings == null) {
    throw new Error("pnpm lockfile must contain a top-level settings block");
  }
  const configured = [...settings.matchAll(/^[ \t]+injectWorkspacePackages: ([^\r\n]+)\r?$/gmu)];
  const validSetting =
    configured.length === 0 ||
    /^  injectWorkspacePackages: (true|false)\r?$/.test(configured[0][0]);
  if (configured.length > 1 || !validSetting) {
    throw new Error("pnpm lockfile has an invalid injectWorkspacePackages setting");
  }
  const eol = settings.includes("\r\n") ? "\r\n" : "\n";
  const qualifiedSettings =
    configured.length === 1
      ? settings.replace(
          /^  injectWorkspacePackages: false\r?$/mu,
          "  injectWorkspacePackages: true",
        )
      : `${settings}  injectWorkspacePackages: true${eol}`;
  return Buffer.from(source.replace(settings, qualifiedSettings), "utf8");
}

export function packageManagerRunner(manager, version) {
  if (manager === "pnpm" || manager === "yarn") {
    return {
      manager,
      command: "corepack",
      prefixArgs: [`${manager}@${version}`],
      label: `${manager}@${version}`,
    };
  }
  return { manager, command: manager, prefixArgs: [], label: manager };
}

export function runPackageManager(runner, args, options) {
  if (runner.prefixArgs.length === 0) {
    return runTypecheckCommand(runner.command, args, commandOptions(options));
  }
  // Fixtures intentionally carry the pinned package-manager contract in the
  // registry, not package.json. Prevent Corepack from walking up to the
  // repository root and applying Vize's own packageManager field instead.
  const env = { ...process.env, ...options.env, COREPACK_ENABLE_PROJECT_SPEC: "0" };
  return runTypecheckCommand(
    runner.command,
    [...runner.prefixArgs, ...args],
    commandOptions({ ...options, env }),
  );
}

export function commandOptions(options) {
  return {
    cwd: options.cwd,
    env: options.env ?? process.env,
    maxBuffer: options.maxBuffer,
    timeoutMs: options.timeout,
  };
}
