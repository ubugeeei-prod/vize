import { existsSync } from "node:fs";
import { dirname, join } from "node:path";

type NuxtWithHooks = {
  options?: { buildDir?: string; typesDir?: string };
  hook(name: string, callback: (...args: unknown[]) => unknown): void;
};

const VIZE_CONTENT_MAPPER = {
  package: "vize",
  extensions: [".vue"],
} as const;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function mapsVueFiles(value: unknown): boolean {
  return (
    isRecord(value) &&
    Array.isArray(value.extensions) &&
    value.extensions.some((extension) => extension === ".vue")
  );
}

function findInstalledVize(fromDir: string): string | undefined {
  for (let dir = fromDir; ; dir = dirname(dir)) {
    const candidate = join(dir, "node_modules", VIZE_CONTENT_MAPPER.package);
    if (existsSync(join(candidate, "package.json"))) return candidate;
    if (dirname(dir) === dir) return undefined;
  }
}

/**
 * TypeScript resolves a mapper `package` from the tsconfig's directory. pnpm
 * links only direct dependencies there, so a `vize` installed through this
 * module is named by the directory this module itself resolves.
 */
export function resolveVizeMapperPackage(tsconfigDir: string, moduleDir: string): string {
  if (findInstalledVize(tsconfigDir) !== undefined) return VIZE_CONTENT_MAPPER.package;
  return findInstalledVize(moduleDir) ?? VIZE_CONTENT_MAPPER.package;
}

export function injectVizeContentMapper(
  tsConfig: unknown,
  mapperPackage: string = VIZE_CONTENT_MAPPER.package,
): boolean {
  if (!isRecord(tsConfig)) {
    return false;
  }

  const configured = tsConfig.contentMappers;
  if (configured !== undefined && !Array.isArray(configured)) {
    return false;
  }

  const contentMappers = configured ?? [];
  if (contentMappers.some(mapsVueFiles)) {
    return false;
  }

  tsConfig.contentMappers = [
    ...contentMappers,
    { package: mapperPackage, extensions: [...VIZE_CONTENT_MAPPER.extensions] },
  ];
  return true;
}

export function registerNuxtContentMapper(
  nuxt: NuxtWithHooks,
  nuxtMajor: number,
  moduleDir?: string,
): void {
  if (nuxtMajor === 2) {
    return;
  }

  nuxt.hook("prepare:types", (payload: unknown) => {
    if (!isRecord(payload)) return;
    const tsconfigDir = nuxt.options?.typesDir || nuxt.options?.buildDir;
    const mapperPackage =
      tsconfigDir && moduleDir
        ? resolveVizeMapperPackage(tsconfigDir, moduleDir)
        : VIZE_CONTENT_MAPPER.package;
    injectVizeContentMapper(payload.tsConfig, mapperPackage);
  });
}
