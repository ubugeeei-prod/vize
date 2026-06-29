import path from "node:path";

export type NuxtMuseaStaticPublicAsset = {
  dir: string;
  baseURL: string;
};

export type NuxtMuseaStaticHookTarget = {
  hook(name: string, callback: (config: { publicAssets?: unknown[] }) => unknown): void;
  options: {
    rootDir: string;
    buildDir: string;
  };
};

export function registerNuxtMuseaStaticPublicAsset(
  nuxt: NuxtMuseaStaticHookTarget,
  basePath: string,
): void {
  nuxt.hook("nitro:config", (nitroConfig) => {
    nitroConfig.publicAssets = [
      ...(nitroConfig.publicAssets ?? []),
      resolveNuxtMuseaStaticPublicAsset(nuxt.options.rootDir, nuxt.options.buildDir, basePath),
    ];
  });
}

export function resolveNuxtMuseaStaticPublicAsset(
  rootDir: string,
  buildDir: string,
  basePath: string,
): NuxtMuseaStaticPublicAsset {
  const staticRoot = museaStaticRootFromBasePath(basePath);
  const resolvedBuildDir = path.isAbsolute(buildDir) ? buildDir : path.resolve(rootDir, buildDir);
  return {
    dir: path.join(resolvedBuildDir, "dist", "client", staticRoot),
    baseURL: normalizeMuseaBasePath(basePath),
  };
}

function museaStaticRootFromBasePath(basePath: string): string {
  return basePath.replace(/^\/+|\/+$/g, "");
}

function normalizeMuseaBasePath(basePath: string): string {
  const normalized = basePath.replace(/^\/+|\/+$/g, "");
  return normalized ? `/${normalized}` : "/";
}
