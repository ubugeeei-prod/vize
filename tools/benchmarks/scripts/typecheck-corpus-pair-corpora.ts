/**
 * Real corpora of the type-check pair, copied from pinned submodules. Each
 * tsconfig restates the project's own type-check config with
 * `incremental: false`, so no run reads or writes a tsbuildinfo. Each corpus
 * resolves its `packages` from tools/benchmarks/typecheck-corpora, copied as
 * their own lockfile closure: misskey and n8n get vue and @types/node, the two
 * libraries also the dependencies their types are written against. Every
 * other import is unresolved in every lane alike.
 */

import { spawnSync } from "node:child_process";
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptsDir, "..", "..", "..");
const CORPUS_PACKAGES_DIR = resolve(scriptsDir, "..", "typecheck-corpora");
const NUXT_UI_THEME_SCRIPT = join(scriptsDir, "typecheck-corpus-nuxt-ui-theme.ts");

/** The packages every corpus resolves: Vue and Node types. */
export const BASE_PACKAGES = ["vue", "@types/node"] as const;

const shared = {
  strict: true,
  target: "ES2022",
  module: "esnext",
  moduleResolution: "Bundler",
  resolveJsonModule: true,
  isolatedModules: true,
  useDefineForClassFields: true,
  skipLibCheck: true,
  noEmit: true,
  incremental: false,
  allowJs: true,
  paths: { "@/*": ["./src/*"] },
  types: ["node"],
  lib: ["esnext", "dom", "dom.iterable"],
  jsx: "preserve",
};

/** Pinned submodule sources; `sfcs` is what the host must hand the mapper. */
export const CORPORA = [
  {
    id: "misskey",
    label: "misskey frontend",
    submodule: "tests/_fixtures/_git/misskey",
    source: "tests/_fixtures/_git/misskey/packages/frontend",
    copy: ["src", "@types"],
    sfcs: 583,
    packages: BASE_PACKAGES,
    tsconfig: {
      compilerOptions: {
        ...shared,
        noImplicitAny: true,
        noImplicitReturns: true,
        experimentalDecorators: true,
        verbatimModuleSyntax: true,
      },
      include: ["./src/**/*.ts", "./src/**/*.vue", "./@types/**/*.ts"],
      exclude: ["./src/**/*.stories.ts"],
    },
  },
  {
    id: "n8n",
    label: "n8n editor-ui",
    submodule: "tests/_fixtures/_git/n8n",
    source: "tests/_fixtures/_git/n8n/packages/frontend/editor-ui",
    copy: ["src"],
    sfcs: 1113,
    packages: BASE_PACKAGES,
    tsconfig: {
      compilerOptions: shared,
      include: ["./src/**/*.ts", "./src/**/*.vue"],
      exclude: ["./src/**/*.test.ts", "./src/**/*.spec.ts", "./src/**/__tests__/**"],
    },
  },
  {
    // The library sources `vue-tsc --noEmit` checks at the root, with the
    // compiler options `nuxt-module-build prepare` writes to .nuxt/tsconfig.json
    // (@nuxt/kit 4.5). `#build/*` is .nuxt, where typecheck-corpus-nuxt-ui-theme.ts
    // writes the theme templates and app-config augmentations `nuxt prepare`
    // generates; `#imports` is the runtime's own Vue-mode stubs.
    id: "nuxt-ui",
    label: "nuxt/ui src",
    submodule: "tests/_fixtures/_git/nuxt-ui",
    source: "tests/_fixtures/_git/nuxt-ui",
    copy: ["src"],
    sfcs: 187,
    packages: [
      ...BASE_PACKAGES,
      "@floating-ui/dom",
      "@iconify/vue",
      "@internationalized/date",
      "@internationalized/number",
      "@nuxt/schema",
      "@standard-schema/spec",
      "@tanstack/vue-table",
      "@tanstack/vue-virtual",
      "@vueuse/core",
      "@vueuse/integrations",
      "@vueuse/shared",
      "colortranslator",
      "defu",
      "fuse.js",
      "hookable",
      "ohash",
      "reka-ui",
      "scule",
      "tailwind-merge",
      "tailwind-variants",
      "ufo",
      "vaul-vue",
      "vue-component-type-helpers",
      "vue-router",
    ],
    generate: NUXT_UI_THEME_SCRIPT,
    tsconfig: {
      compilerOptions: {
        paths: {
          "#build/*": ["./.nuxt/*"],
          "#imports": ["./src/runtime/vue/stubs/vue-router.ts"],
        },
        esModuleInterop: true,
        skipLibCheck: true,
        target: "ESNext",
        allowJs: true,
        allowImportingTsExtensions: true,
        resolveJsonModule: true,
        moduleDetection: "force",
        isolatedModules: true,
        verbatimModuleSyntax: true,
        allowArbitraryExtensions: true,
        strict: true,
        noUncheckedIndexedAccess: true,
        forceConsistentCasingInFileNames: true,
        noImplicitOverride: true,
        module: "preserve",
        noEmit: true,
        incremental: false,
        lib: ["ESNext", "dom", "dom.iterable", "webworker"],
        jsx: "preserve",
        jsxImportSource: "vue",
        types: [],
        moduleResolution: "Bundler",
        useDefineForClassFields: true,
        noImplicitThis: true,
        allowSyntheticDefaultImports: true,
      },
      include: ["./src/**/*", "./.nuxt/types/*.d.ts"],
    },
  },
  {
    // packages/core `type-check`: tsconfig.check.json over tsconfig.app.json
    // over @vue/tsconfig 0.7's tsconfig.dom.json. One excluded story SFC is
    // imported by an included one, so the host maps 455.
    id: "reka-ui",
    label: "reka-ui core",
    submodule: "tests/_fixtures/_git/reka-ui",
    source: "tests/_fixtures/_git/reka-ui/packages/core",
    copy: ["src", "env.d.ts"],
    sfcs: 455,
    packages: [
      ...BASE_PACKAGES,
      "@floating-ui/dom",
      "@floating-ui/vue",
      "@internationalized/date",
      "@internationalized/number",
      "@tanstack/vue-virtual",
      "@vueuse/core",
      "@vueuse/shared",
      "aria-hidden",
      "defu",
      "ohash",
      "vue-component-type-helpers",
    ],
    tsconfig: {
      compilerOptions: {
        noEmit: true,
        module: "ESNext",
        moduleResolution: "Bundler",
        resolveJsonModule: true,
        allowImportingTsExtensions: true,
        moduleDetection: "force",
        jsx: "preserve",
        jsxImportSource: "vue",
        noImplicitThis: true,
        strict: true,
        verbatimModuleSyntax: true,
        target: "ESNext",
        useDefineForClassFields: true,
        esModuleInterop: true,
        forceConsistentCasingInFileNames: true,
        skipLibCheck: true,
        incremental: false,
        lib: ["ESNext", "DOM", "DOM.Iterable"],
        types: [],
        paths: { "@/*": ["./src/*"] },
      },
      include: ["env.d.ts", "src/**/*", "src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
      exclude: ["**/*.test.ts", "src/**/*.story.vue", "src/**/story/*", "src/**/stories/*"],
    },
  },
] as const;
export type Corpus = (typeof CORPORA)[number];

/** The corpora a `--corpora a,b` list names, in pair order; an unknown id is an error. */
export function selectCorpora(list: string | undefined): Corpus[] {
  if (!list?.trim()) return [...CORPORA];
  const ids = list.split(",").map((id) => id.trim());
  for (const id of ids) {
    if (!CORPORA.some((corpus) => corpus.id === id)) throw new Error(`unknown corpus: ${id}`);
  }
  return CORPORA.filter((corpus) => ids.includes(corpus.id));
}

type Lockfile = {
  packages: Record<
    string,
    {
      dependencies?: Record<string, string>;
      optionalDependencies?: Record<string, string>;
      peerDependencies?: Record<string, string>;
    }
  >;
};

/** Where Node resolves `name` from the lockfile entry `from` (`node_modules/a/node_modules/b`). */
function resolveLocked(lock: Lockfile, from: string, name: string): string | undefined {
  for (let dir = from; ;) {
    const candidate = `${dir}/node_modules/${name}`;
    if (lock.packages[candidate]) return candidate;
    const parent = dir.lastIndexOf("/node_modules/");
    if (parent < 0) break;
    dir = dir.slice(0, parent);
  }
  return lock.packages[`node_modules/${name}`] ? `node_modules/${name}` : undefined;
}

/** Lockfile paths of `names` and everything they depend on; uninstalled optional and peer dependencies are skipped. */
export function packageClosure(lock: Lockfile, names: readonly string[]): string[] {
  const queue = names.map((name) => {
    const path = `node_modules/${name}`;
    if (!lock.packages[path]) throw new Error(`typecheck-corpora does not install ${name}`);
    return path;
  });
  const seen = new Set<string>();
  while (queue.length > 0) {
    const path = queue.pop()!;
    if (seen.has(path)) continue;
    seen.add(path);
    const entry = lock.packages[path]!;
    for (const name of Object.keys(entry.dependencies ?? {})) {
      const resolved = resolveLocked(lock, path, name);
      if (!resolved) throw new Error(`${path}: dependency ${name} is not installed`);
      queue.push(resolved);
    }
    for (const name of Object.keys({ ...entry.optionalDependencies, ...entry.peerDependencies })) {
      const resolved = resolveLocked(lock, path, name);
      if (resolved) queue.push(resolved);
    }
  }
  return [...seen].sort();
}

const corpusLock = (): Lockfile =>
  JSON.parse(readFileSync(join(CORPUS_PACKAGES_DIR, "package-lock.json"), "utf8")) as Lockfile;

/** The top-level packages (`vue`, `@vue/shared`) of the closure; nested copies live inside them. */
export function corpusPackageNames(names: readonly string[]): string[] {
  return packageClosure(corpusLock(), names)
    .filter((path) => path.lastIndexOf("node_modules/") === 0)
    .map((path) => path.slice("node_modules/".length));
}

/** The corpus's package closure from typecheck-corpora; nested node_modules come from their own entries. */
function copyCorpusPackages(dir: string, corpus: Corpus): void {
  for (const path of packageClosure(corpusLock(), corpus.packages)) {
    const source = join(CORPUS_PACKAGES_DIR, path);
    const nested = join(source, "node_modules");
    cpSync(source, join(dir, path), {
      recursive: true,
      verbatimSymlinks: true,
      filter: (entry) => entry !== nested && !entry.startsWith(`${nested}/`),
    });
  }
}

/** Corpus sources, its pinned packages and generated files, before any mapper is installed. */
export function prepareCorpus(dir: string, corpus: Corpus): void {
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  for (const entry of corpus.copy) {
    cpSync(join(repoRoot, corpus.source, entry), join(dir, entry), {
      recursive: true,
      verbatimSymlinks: true,
    });
  }
  writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(corpus.tsconfig, null, 2)}\n`);
  copyCorpusPackages(dir, corpus);
  if ("generate" in corpus) {
    const run = spawnSync(process.execPath, [corpus.generate, dir], { encoding: "utf8" });
    if (run.status !== 0) {
      throw new Error(`${corpus.id}: ${corpus.generate} failed\n${run.stderr}${run.stdout}`);
    }
  }
}
