import { readFileSync } from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

import type { PatinaBinding } from "./model.js";

const require = createRequire(import.meta.url);
const FALLBACK_BINDING_PACKAGE = "@vizejs/native";

let bindingCache: PatinaBinding | null = null;

function isMusl(): boolean {
  const report = process.report?.getReport();
  if (typeof report === "object" && report !== null && "header" in report) {
    const header = (report as { header: { glibcVersionRuntime?: string } }).header;
    return !header.glibcVersionRuntime;
  }

  try {
    const lddPath = require("node:child_process").execSync("which ldd").toString().trim();
    return readFileSync(lddPath, "utf8").includes("musl");
  } catch {
    return true;
  }
}

function getBindingPackageName(): string {
  const { arch, platform } = process;

  switch (platform) {
    case "darwin":
      switch (arch) {
        case "arm64":
          return "@vizejs/native-darwin-arm64";
        case "x64":
          return "@vizejs/native-darwin-x64";
        default:
          throw new Error(`Unsupported architecture on macOS: ${arch}`);
      }
    case "linux":
      switch (arch) {
        case "arm64":
          return isMusl() ? "@vizejs/native-linux-arm64-musl" : "@vizejs/native-linux-arm64-gnu";
        case "x64":
          return isMusl() ? "@vizejs/native-linux-x64-musl" : "@vizejs/native-linux-x64-gnu";
        default:
          throw new Error(`Unsupported architecture on Linux: ${arch}`);
      }
    case "win32":
      switch (arch) {
        case "arm64":
          return "@vizejs/native-win32-arm64-msvc";
        case "x64":
          return "@vizejs/native-win32-x64-msvc";
        default:
          throw new Error(`Unsupported architecture on Windows: ${arch}`);
      }
    default:
      throw new Error(`Unsupported OS: ${platform}`);
  }
}

function isPatinaBinding(binding: Partial<PatinaBinding>): binding is PatinaBinding {
  return (
    typeof binding.lintPatinaSfc === "function" && typeof binding.getPatinaRules === "function"
  );
}

export function loadBinding(): PatinaBinding {
  if (bindingCache) {
    return bindingCache;
  }

  const attemptedPackages = getAttemptedPackages();
  let lastError: unknown = null;

  for (const packageName of attemptedPackages) {
    try {
      const binding = require(packageName) as Partial<PatinaBinding>;
      if (!isPatinaBinding(binding)) {
        throw new Error(`${packageName} does not expose the Patina Oxlint bridge.`);
      }

      bindingCache = binding;
      return bindingCache;
    } catch (error) {
      lastError = error;
    }
  }

  const message = lastError instanceof Error && lastError.message ? ` ${lastError.message}` : "";
  throw new Error(
    `Failed to load the Vize native binding. Tried ${attemptedPackages.join(", ")}.${message}`,
  );
}

function getAttemptedPackages(): readonly string[] {
  const platformBindingPackage = getBindingPackageName();
  return shouldPreferWorkspaceBinding(resolveFallbackBindingPath())
    ? [FALLBACK_BINDING_PACKAGE, platformBindingPackage]
    : [platformBindingPackage, FALLBACK_BINDING_PACKAGE];
}

function resolveFallbackBindingPath(): string | null {
  try {
    return require.resolve(FALLBACK_BINDING_PACKAGE);
  } catch {
    return null;
  }
}

function shouldPreferWorkspaceBinding(resolvedPath: string | null): boolean {
  const override = process.env.VIZE_PREFER_WORKSPACE_BINDING;
  if (override === "1" || override === "true") {
    return true;
  }
  if (override === "0" || override === "false") {
    return false;
  }
  if (resolvedPath == null) {
    return false;
  }

  return resolvedPath.includes(`${path.sep}npm${path.sep}vize-native${path.sep}`);
}

if (import.meta.vitest) {
  const { describe, expect, it } = import.meta.vitest;

  describe("shouldPreferWorkspaceBinding", () => {
    it("detects the local workspace native package", () => {
      expect(
        shouldPreferWorkspaceBinding(
          `${path.sep}Users${path.sep}example${path.sep}repo${path.sep}npm${path.sep}vize-native${path.sep}index.js`,
        ),
      ).toBe(true);
    });

    it("ignores published platform packages", () => {
      expect(
        shouldPreferWorkspaceBinding(
          `${path.sep}repo${path.sep}node_modules${path.sep}.pnpm${path.sep}@vizejs+native-darwin-arm64${path.sep}node_modules${path.sep}@vizejs${path.sep}native-darwin-arm64${path.sep}index.js`,
        ),
      ).toBe(false);
    });

    it("returns false when the fallback package cannot be resolved", () => {
      expect(shouldPreferWorkspaceBinding(null)).toBe(false);
    });
  });
}
