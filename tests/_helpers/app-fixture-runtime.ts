import { execFileSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

export const NPMX_E2E_ENV = {
  NUXT_TEST_FIXTURES: "true",
  NUXT_SESSION_PASSWORD: "e2e-test-dummy-session-password-32chars!",
  VIZE_E2E_DISABLE_LUNARIA: "1",
} as const;
export const FRONTEND_PHPCON_E2E_API_BASE = "/__vize_e2e/api";
export const FRONTEND_PHPCON_STAFF_ROUTE_RELATIVE_PATH = path.join(
  "server",
  "routes",
  "__vize_e2e",
  "api",
  "staff.get.ts",
);
export const VUEFES_E2E_ENV = {
  AUTH_SECRET: "e2e-test-dummy-auth-secret-32chars!",
  NEXTAUTH_SECRET: "e2e-test-dummy-auth-secret-32chars!",
} as const;
export const FRONTEND_PHPCON_E2E_ENV = {
  NUXT_PUBLIC_API_BASE: FRONTEND_PHPCON_E2E_API_BASE,
  NUXT_TELEMETRY_DISABLED: "1",
} as const;

const HELPERS_DIR = path.dirname(fileURLToPath(import.meta.url));
const NPMX_REGISTRY_FIXTURE_SOURCE = path.resolve(
  HELPERS_DIR,
  "../_fixtures/npmx-e2e-registry-fixtures.ts",
);

export function readDotenvValue(filePath: string, key: string): string | undefined {
  if (!fs.existsSync(filePath)) {
    return undefined;
  }

  const prefix = `${key}=`;
  for (const line of fs.readFileSync(filePath, "utf-8").split(/\r?\n/)) {
    if (!line.startsWith(prefix)) {
      continue;
    }
    const value = line.slice(prefix.length);
    const quote = value.at(0);
    if ((quote === "'" || quote === '"') && value.endsWith(quote)) {
      return value.slice(1, -1);
    }
    return value;
  }
  return undefined;
}

export function execNpxCommand(
  args: string[],
  opts: { cwd: string; env?: NodeJS.ProcessEnv; timeout?: number },
): void {
  const executable = process.platform === "win32" ? (process.env.ComSpec ?? "cmd.exe") : "npx";
  const executableArgs =
    process.platform === "win32" ? ["/d", "/s", "/c", "npx.cmd", ...args] : args;
  execFileSync(executable, executableArgs, {
    cwd: opts.cwd,
    env: opts.env,
    stdio: "inherit",
    timeout: opts.timeout ?? 600_000,
  });
}

// The App E2E row itself runs inside a cacheable `vp run` task, so the runner exports the
// outer file-spy hooks (LD_PRELOAD/FSPY) to every child process. npmx.dev pins its own,
// different vite-plus version, and letting it attach a second spy layer on top of the outer
// one makes its task spawn fail with `Invalid argument (os error 22)`. Disabling the cache
// for these generator tasks keeps the spawn plain, matching the `cache: false` workaround
// upstream npmx.dev applies for the same failure.
export function npmxGeneratorTaskArgs(task: "generate:lexicons" | "generate:sprite"): string[] {
  return ["-y", "pnpm@10", "exec", "vp", "run", "--no-cache", task];
}

export function patchNpmxRegistryFixtures(npmxDir: string): void {
  const fixtureTarget = path.join(npmxDir, "shared", "utils", "__vize-e2e-npm-fixtures.ts");
  fs.mkdirSync(path.dirname(fixtureTarget), { recursive: true });
  fs.copyFileSync(NPMX_REGISTRY_FIXTURE_SOURCE, fixtureTarget);

  patchNpmxNpmPlugin(path.join(npmxDir, "app", "plugins", "npm.ts"));
  patchNpmxResolvedVersion(
    path.join(npmxDir, "app", "composables", "npm", "useResolvedVersion.ts"),
  );
  patchNpmxServerNpmUtils(path.join(npmxDir, "server", "utils", "npm.ts"));
}

function patchNpmxNpmPlugin(pluginPath: string): void {
  let source = fs.readFileSync(pluginPath, "utf-8");
  if (source.includes("resolveVizeE2ENpmRegistryCachedResponse")) {
    return;
  }

  source = addImport(
    source,
    "import { resolveVizeE2ENpmRegistryCachedResponse } from '#shared/utils/__vize-e2e-npm-fixtures'\n",
  );
  source = replaceOnce(
    source,
    `      ) => {
        return cachedFetch<T>(url, { baseURL: NPM_REGISTRY, ...options }, ttl)
      },
`,
    `      ) => {
        const baseURL = typeof options?.baseURL === 'string' ? options.baseURL : NPM_REGISTRY
        const fixture =
          import.meta.server && process.env.NUXT_TEST_FIXTURES === 'true'
            ? resolveVizeE2ENpmRegistryCachedResponse<T>(url, baseURL)
            : { handled: false }
        if (fixture.handled) {
          return Promise.resolve(fixture.data)
        }

        return cachedFetch<T>(url, { baseURL: NPM_REGISTRY, ...options }, ttl)
      },
`,
    pluginPath,
  );
  fs.writeFileSync(pluginPath, source);
}

function patchNpmxResolvedVersion(composablePath: string): void {
  let source = fs.readFileSync(composablePath, "utf-8");
  if (source.includes("resolveVizeE2EFastNpmMetaVersion")) {
    return;
  }

  source = addImport(
    source,
    "import { resolveVizeE2EFastNpmMetaVersion } from '#shared/utils/__vize-e2e-npm-fixtures'\n",
  );
  source = replaceOnce(
    source,
    `      const data = await $fetch<ResolvedPackageVersion>(url)
      return data.version
`,
    `      const fixture =
        import.meta.server && process.env.NUXT_TEST_FIXTURES === 'true'
          ? resolveVizeE2EFastNpmMetaVersion(url)
          : { handled: false }
      if (fixture.handled) {
        return fixture.data
      }

      const data = await $fetch<ResolvedPackageVersion>(url)
      return data.version
`,
    composablePath,
  );
  fs.writeFileSync(composablePath, source);
}

function patchNpmxServerNpmUtils(npmUtilsPath: string): void {
  let source = fs.readFileSync(npmUtilsPath, "utf-8");
  if (source.includes("resolveVizeE2ENpmPackument")) {
    return;
  }

  source = addImport(
    source,
    "import { resolveVizeE2ENpmPackument } from '#shared/utils/__vize-e2e-npm-fixtures'\n",
  );
  source = replaceOnce(
    source,
    `    const encodedName = encodePackageName(name)
    return await $fetch<Packument>(\`\${NPM_REGISTRY}/\${encodedName}\`)
`,
    `    const encodedName = encodePackageName(name)
    if (process.env.NUXT_TEST_FIXTURES === 'true') {
      const fixture = resolveVizeE2ENpmPackument<Packument>(encodedName, NPM_REGISTRY)
      if (fixture.handled) {
        return fixture.data as Packument
      }
    }

    return await $fetch<Packument>(\`\${NPM_REGISTRY}/\${encodedName}\`)
`,
    npmUtilsPath,
  );
  fs.writeFileSync(npmUtilsPath, source);
}

function addImport(source: string, importLine: string): string {
  return source.startsWith(importLine) ? source : `${importLine}${source}`;
}

function replaceOnce(
  source: string,
  search: string,
  replacement: string,
  filePath: string,
): string {
  if (!source.includes(search)) {
    throw new Error(`missing npmx registry fixture patch anchor in ${filePath}`);
  }
  return source.replace(search, replacement);
}

export function patchNuxtPrerenderForE2E(configPath: string): void {
  const source = fs.readFileSync(configPath, "utf-8");
  const nextSource = source
    .replace("'/' : { prerender: true }", "'/' : { prerender: false }")
    .replace("'/' : { prerender: true },", "'/' : { prerender: false },")
    .replace("'/': { prerender: true }", "'/': { prerender: false }")
    .replace("'/': { prerender: true },", "'/': { prerender: false },")
    .replace("crawlLinks: true", "crawlLinks: false");
  if (nextSource !== source) {
    fs.writeFileSync(configPath, nextSource);
  }
}

export function writeFrontendPhpconStaffRoute(
  frontendDir: string,
  writeFile: (filePath: string, content: string) => void,
): void {
  writeFile(
    path.join(frontendDir, FRONTEND_PHPCON_STAFF_ROUTE_RELATIVE_PATH),
    `export default defineEventHandler(() => ({
  staff_types: [
    {
      name: "実行委員長",
      name_en: "Chair",
      staff: [
        {
          id: "chair-1",
          name: "Hokkaido Chair",
          url: "https://example.com/staff/chair",
        },
      ],
    },
    {
      name: "コアスタッフ",
      name_en: "Core Staff",
      staff: [
        {
          id: "core-1",
          name: "Core Staff A",
          url: "https://example.com/staff/core-a",
        },
        {
          id: "core-2",
          name: "Core Staff B",
          url: "https://example.com/staff/core-b",
        },
      ],
    },
  ],
}));
`,
  );
}
