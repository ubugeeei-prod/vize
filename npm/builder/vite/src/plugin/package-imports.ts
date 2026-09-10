import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import {
  createViteBareImportBases,
  resolveViteRelativeImport,
  splitViteIdQuery,
} from "@vizejs/native";

import type { VizePluginState } from "./state.ts";

const PACKAGE_IMPORT_CONDITIONS = [
  "browser",
  "import",
  "module",
  "default",
  "node",
  "require",
  "types",
] as const;

type PackageImportMap = Record<string, unknown>;

export function resolvePackageJsonImportFromVizeImporter(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  if (!request.startsWith("#")) {
    return null;
  }

  const seen = new Set<string>();
  for (const base of createViteBareImportBases(state.root, importer)) {
    const packageRoot = findPackageScope(base);
    if (!packageRoot || seen.has(packageRoot)) {
      continue;
    }
    seen.add(packageRoot);

    const direct = tryRequireResolve(path.join(packageRoot, "package.json"), request, querySuffix);
    if (direct) {
      return direct;
    }

    const imports = readPackageImports(packageRoot);
    const target = imports ? resolvePackageImportTarget(imports, request) : null;
    const resolved = target
      ? resolvePackageImportTargetPath(packageRoot, target, importer, querySuffix)
      : null;
    if (resolved) {
      return resolved;
    }
  }

  return null;
}

function findPackageScope(base: string): string | null {
  let current = fs.existsSync(base) && fs.statSync(base).isDirectory() ? base : path.dirname(base);
  while (current !== path.dirname(current)) {
    if (fs.existsSync(path.join(current, "package.json"))) {
      return current;
    }
    current = path.dirname(current);
  }
  return null;
}

function tryRequireResolve(base: string, request: string, querySuffix: string): string | null {
  try {
    return `${createRequire(base).resolve(request)}${querySuffix}`;
  } catch {
    return null;
  }
}

function readPackageImports(packageRoot: string): PackageImportMap | null {
  try {
    const json = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8")) as {
      imports?: unknown;
    };
    return isPlainObject(json.imports) ? json.imports : null;
  } catch {
    return null;
  }
}

function resolvePackageImportTarget(imports: PackageImportMap, request: string): string | null {
  const exact = resolveConditionalTarget(imports[request]);
  if (exact) {
    return exact;
  }

  let best: { keyLength: number; target: string } | null = null;
  for (const [key, value] of Object.entries(imports)) {
    const starIndex = key.indexOf("*");
    if (starIndex === -1) {
      continue;
    }
    const prefix = key.slice(0, starIndex);
    const suffix = key.slice(starIndex + 1);
    if (!request.startsWith(prefix) || !request.endsWith(suffix)) {
      continue;
    }

    const target = resolveConditionalTarget(value);
    if (!target || (best && key.length <= best.keyLength)) {
      continue;
    }
    const matched = request.slice(prefix.length, request.length - suffix.length);
    best = { keyLength: key.length, target: target.replaceAll("*", matched) };
  }

  return best?.target ?? null;
}

function resolveConditionalTarget(value: unknown): string | null {
  if (typeof value === "string") {
    return value;
  }
  if (Array.isArray(value)) {
    for (const item of value) {
      const target = resolveConditionalTarget(item);
      if (target) {
        return target;
      }
    }
    return null;
  }
  if (!isPlainObject(value)) {
    return null;
  }
  for (const condition of PACKAGE_IMPORT_CONDITIONS) {
    const target = resolveConditionalTarget(value[condition]);
    if (target) {
      return target;
    }
  }
  return null;
}

function resolvePackageImportTargetPath(
  packageRoot: string,
  target: string,
  importer: string,
  querySuffix: string,
): string | null {
  const { request } = splitViteIdQuery(target);
  if (request.startsWith("./")) {
    const resolved = resolveViteRelativeImport(request, path.join(packageRoot, "package.json"));
    return resolved && isInsidePackage(packageRoot, resolved) ? `${resolved}${querySuffix}` : null;
  }
  if (request.startsWith("../")) {
    return null;
  }
  if (path.isAbsolute(request)) {
    return fs.existsSync(request) ? `${request}${querySuffix}` : null;
  }
  return tryRequireResolve(importer, request, querySuffix);
}

function isPlainObject(value: unknown): value is PackageImportMap {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isInsidePackage(packageRoot: string, resolved: string): boolean {
  const relative = path.relative(packageRoot, splitViteIdQuery(resolved).request);
  return (
    relative === "" || (!!relative && !relative.startsWith("..") && !path.isAbsolute(relative))
  );
}
