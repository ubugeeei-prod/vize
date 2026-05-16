import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import * as path from "node:path";

const require = createRequire(import.meta.url);
const WORKSPACE_BINDING_PATH = "../../vize-native";

interface NativeBinding {
  runCli: (args: string[]) => void;
}

function isMusl(): boolean {
  const report = process.report?.getReport();
  if (typeof report === "object" && report !== null && "header" in report) {
    const header = (report as { header: { glibcVersionRuntime?: string } }).header;
    return !header.glibcVersionRuntime;
  }

  try {
    const lddPath = require("child_process").execSync("which ldd").toString().trim();
    return readFileSync(lddPath, "utf8").includes("musl");
  } catch {
    return true;
  }
}

function getBindingPackageName(): string {
  const { platform, arch } = process;

  switch (platform) {
    case "darwin":
      switch (arch) {
        case "x64":
          return "@vizejs/native-darwin-x64";
        case "arm64":
          return "@vizejs/native-darwin-arm64";
        default:
          throw new Error(`Unsupported architecture on macOS: ${arch}`);
      }
    case "win32":
      switch (arch) {
        case "x64":
          return "@vizejs/native-win32-x64-msvc";
        case "arm64":
          return "@vizejs/native-win32-arm64-msvc";
        default:
          throw new Error(`Unsupported architecture on Windows: ${arch}`);
      }
    case "linux":
      switch (arch) {
        case "x64":
          return isMusl() ? "@vizejs/native-linux-x64-musl" : "@vizejs/native-linux-x64-gnu";
        case "arm64":
          return isMusl() ? "@vizejs/native-linux-arm64-musl" : "@vizejs/native-linux-arm64-gnu";
        default:
          throw new Error(`Unsupported architecture on Linux: ${arch}`);
      }
    default:
      throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
  }
}

function loadNative(): NativeBinding {
  const attemptedPackages = getAttemptedPackages();
  let lastError: unknown = null;

  for (const packageName of attemptedPackages) {
    try {
      const binding = require(packageName) as Partial<NativeBinding>;
      if (typeof binding.runCli !== "function") {
        throw new Error(`${packageName} does not expose the Rust CLI binding.`);
      }
      return binding as NativeBinding;
    } catch (error) {
      lastError = error;
    }
  }

  console.error(`Failed to load native binding. Tried: ${attemptedPackages.join(", ")}`);
  console.error("Try reinstalling: npm install vize");
  throw lastError instanceof Error ? lastError : new Error("Failed to load native binding");
}

function getAttemptedPackages(): readonly string[] {
  const platformBindingPackage = getBindingPackageName();
  return shouldPreferWorkspaceBinding(resolveWorkspaceBindingPath())
    ? [WORKSPACE_BINDING_PATH, platformBindingPackage]
    : [platformBindingPackage, WORKSPACE_BINDING_PATH];
}

function resolveWorkspaceBindingPath(): string | null {
  try {
    return require.resolve(WORKSPACE_BINDING_PATH);
  } catch {
    return null;
  }
}

export function shouldPreferWorkspaceBinding(resolvedPath: string | null): boolean {
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

export function sanitizeTerminalText(value: unknown): string {
  const text = String(value);
  let sanitized = "";
  let segmentStart = 0;

  for (let i = 0; i < text.length; i++) {
    const code = text.charCodeAt(i);
    let skipEnd = -1;
    if (code === 0x1b) {
      skipEnd = skipTerminalEscapeSequence(text, i);
    } else if (isUnsafeTerminalControl(code)) {
      skipEnd = i;
    }

    if (skipEnd !== -1) {
      if (segmentStart < i) {
        sanitized += text.slice(segmentStart, i);
      }
      i = skipEnd;
      segmentStart = i + 1;
    }
  }

  return segmentStart === 0 ? text : sanitized + text.slice(segmentStart);
}

function skipTerminalEscapeSequence(text: string, escapeIndex: number): number {
  const introducer = text.charCodeAt(escapeIndex + 1);
  if (introducer === 0x5b) {
    return skipUntilAnsiFinalByte(text, escapeIndex + 2);
  }
  if (introducer === 0x5d || introducer === 0x50 || introducer === 0x5e || introducer === 0x5f) {
    return skipUntilStringTerminator(text, escapeIndex + 2);
  }
  if (Number.isNaN(introducer)) {
    return escapeIndex;
  }
  return escapeIndex + 1;
}

function skipUntilAnsiFinalByte(text: string, index: number): number {
  for (let i = index; i < text.length; i++) {
    const code = text.charCodeAt(i);
    if (code >= 0x40 && code <= 0x7e) {
      return i;
    }
  }
  return text.length - 1;
}

function skipUntilStringTerminator(text: string, index: number): number {
  for (let i = index; i < text.length; i++) {
    const code = text.charCodeAt(i);
    if (code === 0x07) {
      return i;
    }
    if (code === 0x1b && text.charCodeAt(i + 1) === 0x5c) {
      return i + 1;
    }
  }
  return text.length - 1;
}

function isUnsafeTerminalControl(code: number): boolean {
  if (code === 0x09 || code === 0x0a || code === 0x0d) {
    return false;
  }
  return (code >= 0x00 && code <= 0x1f) || (code >= 0x7f && code <= 0x9f);
}

function main(): void {
  loadNative().runCli(process.argv.slice(2));
}

const isTestRuntime =
  Boolean(import.meta.vitest) || process.env.VITEST === "true" || process.env.NODE_ENV === "test";

if (!isTestRuntime) {
  try {
    main();
  } catch (error) {
    console.error(sanitizeTerminalText(error instanceof Error ? error.message : String(error)));
    process.exit(1);
  }
}
