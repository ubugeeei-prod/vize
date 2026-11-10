import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { nativeVersionMismatchMessage } from "./native-version.js";

const require = createRequire(import.meta.url);

function readPackageVersion(filename: string): unknown {
  const parsed = JSON.parse(readFileSync(filename, "utf8")) as { version?: unknown };
  return parsed.version;
}

/** Refuse a CLI package paired with a different native package version. */
export function assertMatchingNativeVersion(): void {
  const packageRoot = path.resolve(fileURLToPath(new URL(".", import.meta.url)), "..");
  const cliVersion = readPackageVersion(path.join(packageRoot, "package.json"));
  // A joined specifier stays a runtime lookup. Pack must not inline the
  // build machine's @vizejs/native version over the installed package.
  const nativeSpecifier = ["@vizejs/native", "package.json"].join("/");
  let nativeVersion: unknown;
  try {
    nativeVersion = readPackageVersion(require.resolve(nativeSpecifier));
  } catch (error) {
    const code =
      typeof error === "object" && error !== null && "code" in error ? error.code : undefined;
    if (code === "MODULE_NOT_FOUND" || code === "ERR_PACKAGE_PATH_NOT_EXPORTED") return;
    throw error;
  }
  const message = nativeVersionMismatchMessage(cliVersion, nativeVersion);
  if (message !== null) throw new Error(message);
}
