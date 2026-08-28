import fs from "node:fs";
import path from "node:path";

type SkippableTest = { skip(reason: string): void };

const BIN_EXT = process.platform === "win32" ? ".exe" : "";

function dependencyIsAvailable(value: unknown): boolean {
  return value !== null && value !== undefined && value !== false;
}

export function stableTypeScriptRuntimePath(root: string): string {
  return path.join(
    root,
    "node_modules",
    "@typescript",
    `typescript-${process.platform}-${process.arch}`,
    "lib",
    `tsc${BIN_EXT}`,
  );
}

export function resolveTypecheckRuntime(
  root: string,
  extraCandidates: Array<string | null | undefined | false> = [],
): string | undefined {
  const candidates = [
    process.env.CORSA_BIN,
    process.env.CORSA_PATH,
    process.env.TSGO_PATH,
    process.env.TSGO_EXECUTABLE,
    process.env.CORSA_EXECUTABLE,
    ...extraCandidates,
    path.join(root, "../corsa-bind/.cache/tsgo"),
    stableTypeScriptRuntimePath(root),
    path.join(root, "node_modules/.bin/corsa"),
    path.join(root, "node_modules/.bin/tsgo"),
    path.join(root, "tests/node_modules/.bin/corsa"),
    path.join(root, "tests/node_modules/.bin/tsgo"),
  ];
  return candidates.find(
    (candidate): candidate is string => Boolean(candidate) && fs.existsSync(candidate),
  );
}

export function typecheckDependencySkip(
  value: unknown,
  label: string,
  skipReason: string,
  required = process.env.VIZE_TEST_REQUIRE_TSGO === "1",
): false | string {
  if (dependencyIsAvailable(value)) return false;

  if (required) {
    throw new Error(`${label} is required when VIZE_TEST_REQUIRE_TSGO=1`);
  }
  return skipReason;
}

export function requireTypecheckDependency<T>(
  t: SkippableTest,
  value: T | null | undefined | false,
  label: string,
  skipReason: string,
  required = process.env.VIZE_TEST_REQUIRE_TSGO === "1",
): T | undefined {
  const skip = typecheckDependencySkip(value, label, skipReason, required);
  if (skip !== false) {
    t.skip(skip);
    return undefined;
  }
  return value as T;
}
