// Extension metadata shared by the VS Code Marketplace and Open VSX publishers.
import { writeStderr } from "./command_io.ts";
import { nonEmptyEnv, parseMoonInt, readFileToString } from "./command_runtime.ts";
import { isJsonObject, parseJson, stringMember, type JsonValue } from "./ordered_json.ts";

export interface ExtensionInfo {
  extensionId: string;
  publisher: string;
  name: string;
  version: string;
}

function intAtLeastOr(value: string, minimum: number, fallback: number): number {
  const parsed = parseMoonInt(value);
  return parsed !== null && parsed >= minimum ? parsed : fallback;
}

/** Reads an integer setting that must be at least `minimum`, else the fallback. */
export function envIntAtLeast(name: string, minimum: number, fallback: number): number {
  const value = nonEmptyEnv(name);
  return value === undefined ? fallback : intAtLeastOr(value, minimum, fallback);
}

export function extensionInfoFromPackageJson(value: JsonValue): ExtensionInfo | null {
  if (!isJsonObject(value)) return null;
  const publisher = stringMember(value, "publisher");
  if (publisher === undefined) return null;
  const name = stringMember(value, "name");
  if (name === undefined) return null;
  const version = stringMember(value, "version");
  if (version === undefined) return null;
  if (publisher === "" || name === "" || version === "") return null;
  return { extensionId: `${publisher}.${name}`, publisher, name, version };
}

/** Reads the extension identity, reporting the publisher diagnostics on failure. */
export function readExtensionInfo(packageJsonPath: string): ExtensionInfo | null {
  const content = readFileToString(packageJsonPath);
  const value = content === null ? undefined : parseJson(content);
  if (value === undefined) {
    writeStderr(`Failed to read ${packageJsonPath}\n`);
    return null;
  }
  const info = extensionInfoFromPackageJson(value);
  if (info === null) {
    writeStderr(`Failed to read extension metadata from ${packageJsonPath}\n`);
  }
  return info;
}

/** Returns whether a metadata object lists `expected` in its `versions` array. */
export function jsonHasExtensionVersion(value: JsonValue, expected: string): boolean {
  if (!isJsonObject(value)) return false;
  const versions = value.get("versions");
  if (!Array.isArray(versions)) return false;
  return versions.some((item) => stringMember(item, "version") === expected);
}
