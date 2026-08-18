import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

export const NPM_REGISTRY = "https://registry.npmjs.org";
export const NETWORK_CACHED_FETCH_RESULT = {
  data: { name: "network-cached-fetch" },
  isStale: true,
  cachedAt: "network",
};
export const NETWORK_FETCH_RESULT = { name: "network-packument", version: "0.0.0-network" };

const SSR_DOUBLE_GLOBAL = "__vizeE2ENpmxSsrDouble";

export interface CachedFetchCall {
  url: string;
  options: unknown;
  ttl: unknown;
}

export interface ScenarioResult {
  result: unknown;
  cachedFetchCalls: CachedFetchCall[];
  fetchCalls: string[];
}

export interface NuxtDoubles {
  cachedFetchCalls: CachedFetchCall[];
  fetchCalls: string[];
  setSsr: (value: boolean) => void;
  setEncodePackageName: (encode: (name: string) => string) => void;
  restore: () => void;
}

const SCOPE_SEPARATOR_ENCODING = (name: string) =>
  name.startsWith("@") ? name.replace("/", "%2F") : name;
const FULL_COMPONENT_ENCODING = (name: string) => encodeURIComponent(name);

interface NpmPluginModule {
  default: () => {
    provide: {
      npmRegistry: (
        url: string,
        options?: Record<string, unknown>,
        ttl?: number,
      ) => Promise<unknown>;
    };
  };
}

interface ResolvedVersionModule {
  useResolvedVersion: (name: string) => { key: string; handler: () => Promise<string> };
}

interface ServerNpmModule {
  fetchNpmPackage: (name: string) => Promise<unknown>;
}

export async function collectPatchedBehavior(
  fixtureRoot: string,
  revision: number,
  doubles: NuxtDoubles,
): Promise<Record<string, ScenarioResult>> {
  const pluginModule = await loadPatchedModule<NpmPluginModule>(
    fixtureRoot,
    "app/plugins/npm.ts",
    revision,
  );
  const resolvedVersionModule = await loadPatchedModule<ResolvedVersionModule>(
    fixtureRoot,
    "app/composables/npm/useResolvedVersion.ts",
    revision,
  );
  const serverNpmModule = await loadPatchedModule<ServerNpmModule>(
    fixtureRoot,
    "server/utils/npm.ts",
    revision,
  );

  const { npmRegistry } = pluginModule.default().provide;
  const resolveVersion = (name: string) => resolvedVersionModule.useResolvedVersion(name).handler();
  const behavior: Record<string, ScenarioResult> = {};

  doubles.setSsr(true);
  process.env.NUXT_TEST_FIXTURES = "true";
  behavior["ssr-version-manifest"] = await record(doubles, () => npmRegistry("/vue/3.5.29"));
  behavior["ssr-packument"] = await record(doubles, () => npmRegistry("/vue"));
  behavior["ssr-custom-base-url"] = await record(doubles, () =>
    npmRegistry("/vue/3.5.29", { baseURL: "https://example.test" }),
  );
  behavior["ssr-unhandled-package"] = await record(doubles, () =>
    npmRegistry("/unknown-fixture-package", undefined, 60),
  );
  behavior["ssr-malformed-encoding"] = await record(doubles, () => npmRegistry("/%E0%A4%A"));
  behavior["ssr-resolved-version"] = await record(doubles, () => resolveVersion("vue"));
  behavior["ssr-resolved-version-malformed-encoding"] = await record(doubles, () =>
    resolveVersion("%E0%A4%A"),
  );
  behavior["ssr-resolved-version-unhandled"] = await record(doubles, () =>
    resolveVersion("unknown-fixture-package"),
  );
  behavior["server-packument"] = await record(doubles, () =>
    serverNpmModule.fetchNpmPackage("vue"),
  );
  behavior["server-scoped-packument"] = await record(doubles, () =>
    serverNpmModule.fetchNpmPackage("@vue/compiler-sfc"),
  );
  doubles.setEncodePackageName(FULL_COMPONENT_ENCODING);
  behavior["server-scoped-packument-fully-encoded"] = await record(doubles, () =>
    serverNpmModule.fetchNpmPackage("@vue/compiler-sfc"),
  );
  doubles.setEncodePackageName(SCOPE_SEPARATOR_ENCODING);
  behavior["server-unhandled-package"] = await record(doubles, () =>
    serverNpmModule.fetchNpmPackage("unknown-fixture-package"),
  );

  doubles.setSsr(false);
  behavior["client-version-manifest"] = await record(doubles, () => npmRegistry("/vue/3.5.29"));
  behavior["client-resolved-version"] = await record(doubles, () => resolveVersion("vue"));

  doubles.setSsr(true);
  delete process.env.NUXT_TEST_FIXTURES;
  behavior["ssr-version-manifest-fixtures-disabled"] = await record(doubles, () =>
    npmRegistry("/vue/3.5.29"),
  );
  behavior["server-packument-fixtures-disabled"] = await record(doubles, () =>
    serverNpmModule.fetchNpmPackage("vue"),
  );
  process.env.NUXT_TEST_FIXTURES = "true";

  return behavior;
}

export function installNuxtDoubles(): NuxtDoubles {
  const cachedFetchCalls: CachedFetchCall[] = [];
  const fetchCalls: string[] = [];
  const globalScope = globalThis as unknown as Record<string, unknown>;
  const patchedKeys = [
    "NPM_REGISTRY",
    "defineNuxtPlugin",
    "defineCachedFunction",
    "encodePackageName",
    "useCachedFetch",
    "useAsyncData",
    "$fetch",
    SSR_DOUBLE_GLOBAL,
  ];
  const previous = patchedKeys.map((key) => ({
    key,
    present: key in globalScope,
    value: globalScope[key],
  }));
  let encodePackageName = SCOPE_SEPARATOR_ENCODING;

  globalScope.NPM_REGISTRY = NPM_REGISTRY;
  globalScope.defineNuxtPlugin = (setup: unknown) => setup;
  globalScope.defineCachedFunction = (fn: unknown) => fn;
  globalScope.encodePackageName = (name: string) => encodePackageName(name);
  globalScope.useCachedFetch = () => (url: string, options: unknown, ttl: unknown) => {
    cachedFetchCalls.push({ url, options, ttl });
    return Promise.resolve(structuredClone(NETWORK_CACHED_FETCH_RESULT));
  };
  globalScope.useAsyncData = (key: () => string, handler: () => Promise<unknown>) => ({
    key: key(),
    handler,
  });
  globalScope.$fetch = (url: string) => {
    fetchCalls.push(String(url));
    return Promise.resolve(structuredClone(NETWORK_FETCH_RESULT));
  };

  return {
    cachedFetchCalls,
    fetchCalls,
    setSsr: (value: boolean) => {
      globalScope[SSR_DOUBLE_GLOBAL] = value;
    },
    setEncodePackageName: (encode: (name: string) => string) => {
      encodePackageName = encode;
    },
    restore: () => {
      for (const entry of previous) {
        if (entry.present) {
          globalScope[entry.key] = entry.value;
        } else {
          delete globalScope[entry.key];
        }
      }
    },
  };
}

// Mirrors the npmx.dev sources the patch anchors target, kept executable under Node so the
// patched adapters can run against controlled fetch and Nuxt doubles.
export function writeNpmxFixtureApp(fixtureRoot: string): void {
  writeText(
    path.join(fixtureRoot, "package.json"),
    `${JSON.stringify(
      {
        name: "vize-npmx-registry-fixture-app",
        private: true,
        type: "module",
        imports: {
          "#shared/utils/__vize-e2e-npm-fixtures": "./shared/utils/__vize-e2e-npm-fixtures.ts",
        },
      },
      null,
      2,
    )}\n`,
  );
  writeText(
    path.join(fixtureRoot, "app/plugins/npm.ts"),
    `export default defineNuxtPlugin(() => {
  const cachedFetch = useCachedFetch()

  return {
    provide: {
      npmRegistry: <T>(
        url: Parameters<CachedFetchFunction>[0],
        options?: Parameters<CachedFetchFunction>[1],
        ttl?: Parameters<CachedFetchFunction>[2],
      ) => {
        return cachedFetch<T>(url, { baseURL: NPM_REGISTRY, ...options }, ttl)
      },
    },
  }
})
`,
  );
  writeText(
    path.join(fixtureRoot, "app/composables/npm/useResolvedVersion.ts"),
    `import type { ResolvedPackageVersion } from 'fast-npm-meta'

export function useResolvedVersion(name: string) {
  return useAsyncData(
    () => \`resolved-version:\${name}:latest\`,
    async () => {
      const url = \`https://npm.antfu.dev/\${name}\`
      const data = await $fetch<ResolvedPackageVersion>(url)
      return data.version
    },
  )
}
`,
  );
  writeText(
    path.join(fixtureRoot, "server/utils/npm.ts"),
    `import type { Packument } from '#shared/types/npm-registry'

export const fetchNpmPackage = defineCachedFunction(
  async (name: string): Promise<Packument> => {
    const encodedName = encodePackageName(name)
    return await $fetch<Packument>(\`\${NPM_REGISTRY}/\${encodedName}\`)
  },
)
`,
  );
  writeText(
    path.join(fixtureRoot, "modules/runtime/server/cache.ts"),
    `function getMockForUrl(url: string): unknown {
  const urlObj = URL.parse(url)
  if (!urlObj) return null

  const { host, pathname } = urlObj

  // npm API: downloads range → synthetic daily data for sparklines
  if (host === 'api.npmjs.org') {
    return null
  }

  return null
}
`,
  );
}

async function record(doubles: NuxtDoubles, run: () => Promise<unknown>): Promise<ScenarioResult> {
  doubles.cachedFetchCalls.length = 0;
  doubles.fetchCalls.length = 0;
  const result = await run();
  return {
    result,
    cachedFetchCalls: [...doubles.cachedFetchCalls],
    fetchCalls: [...doubles.fetchCalls],
  };
}

async function loadPatchedModule<T>(
  fixtureRoot: string,
  relativePath: string,
  revision: number,
): Promise<T> {
  const patchedPath = path.join(fixtureRoot, relativePath);
  const executablePath = patchedPath.replace(/\.ts$/, `.__exec${revision}.ts`);
  const patchedSource = fs.readFileSync(patchedPath, "utf-8");
  fs.writeFileSync(
    executablePath,
    patchedSource.replaceAll("import.meta.server", `globalThis.${SSR_DOUBLE_GLOBAL}`),
  );
  return (await import(pathToFileURL(executablePath).href)) as T;
}

function writeText(filePath: string, content: string): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}
