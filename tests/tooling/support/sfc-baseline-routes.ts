import { globSync, statSync } from "node:fs";
import path from "node:path";

export const sfcDialects = ["0.10", "0.11", "1", "2", "2.7", "3"] as const;
export type SfcDialect = (typeof sfcDialects)[number];

export type SfcDialectRoute = {
  id: string;
  dialect: SfcDialect;
  globs: string[];
};

export type DialectRoutedProject = {
  id: string;
  fixtureDir: string;
  vueGlobs: string[];
  sfcDialectRoutes?: SfcDialectRoute[];
};

export type ResolvedSfcDialect = {
  routeId: string;
  dialect: SfcDialect;
};

const dialectSet = new Set<string>(sfcDialects);

/**
 * Resolve a project's registered SFCs through an unordered exact partition.
 * Projects without operational legacy routes retain the existing Vue 3
 * baseline; the repository-wide capability inventory is owned by #4102.
 */
export function resolveSfcDialectPartition(
  project: DialectRoutedProject,
  registeredFiles: string[],
): Map<string, ResolvedSfcDialect> {
  const routes = project.sfcDialectRoutes;
  if (routes == null) {
    return new Map(
      registeredFiles.map((file) => [file, { routeId: "registry-vue3", dialect: "3" }]),
    );
  }
  validateRouteShapes(routes);
  const registered = new Set(registeredFiles);
  const matches = new Map<string, ResolvedSfcDialect[]>();

  for (const route of routes) {
    for (const pattern of route.globs) {
      const files = collectRouteFiles(project.fixtureDir, pattern);
      if (files.length === 0)
        throw new Error(`${project.id}:${route.id} glob matched no files: ${pattern}`);
      for (const file of files) {
        if (!registered.has(file)) {
          throw new Error(`${project.id}:${route.id} routed file is outside vueGlobs: ${file}`);
        }
        const selected = matches.get(file) ?? [];
        if (!selected.some((candidate) => candidate.routeId === route.id)) {
          selected.push({ routeId: route.id, dialect: route.dialect });
        }
        matches.set(file, selected);
      }
    }
  }

  const result = new Map<string, ResolvedSfcDialect>();
  for (const file of registeredFiles) {
    const selected = matches.get(file) ?? [];
    if (selected.length === 0) throw new Error(`${project.id} SFC has no dialect route: ${file}`);
    if (selected.length > 1) {
      throw new Error(
        `${project.id} SFC has overlapping dialect routes: ${file} ` +
          `(${selected.map((route) => route.routeId).join(", ")})`,
      );
    }
    result.set(file, selected[0]);
  }
  return result;
}

export function validateRouteShapes(routes: SfcDialectRoute[]): void {
  if (!Array.isArray(routes) || routes.length === 0) {
    throw new Error("sfcDialectRoutes must be a non-empty array");
  }
  const ids = new Set<string>();
  for (const route of routes) {
    if (route == null || typeof route !== "object")
      throw new Error("dialect routes must be objects");
    if (!/^[a-z][a-z0-9-]*$/.test(route.id))
      throw new Error(`invalid dialect route id: ${route.id}`);
    if (ids.has(route.id)) throw new Error(`duplicate dialect route id: ${route.id}`);
    ids.add(route.id);
    if (!dialectSet.has(route.dialect)) throw new Error(`invalid SFC dialect: ${route.dialect}`);
    if (!Array.isArray(route.globs) || route.globs.length === 0) {
      throw new Error(`${route.id} dialect route must declare globs`);
    }
    const globs = new Set<string>();
    for (const pattern of route.globs) {
      if (!isSafeVueGlob(pattern)) throw new Error(`invalid SFC dialect glob: ${pattern}`);
      if (globs.has(pattern)) throw new Error(`duplicate SFC dialect glob: ${pattern}`);
      globs.add(pattern);
    }
  }
}

function collectRouteFiles(fixtureDir: string, pattern: string): string[] {
  return globSync(pattern, { cwd: fixtureDir, exclude: [".yarn/**", "**/node_modules/**"] })
    .filter((entry) => statSync(path.resolve(fixtureDir, entry)).isFile())
    .map((entry) => entry.replaceAll("\\", "/"))
    .sort(codePointCompare);
}

function codePointCompare(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function isSafeVueGlob(pattern: string): boolean {
  return (
    pattern.endsWith(".vue") &&
    !pattern.startsWith("/") &&
    !/^[A-Za-z]:/.test(pattern) &&
    !pattern.includes("\\") &&
    !pattern.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
  );
}
