/**
 * Real corpora of the type-check pair, copied from pinned submodules. Each
 * tsconfig restates the project's own type-check config with
 * `incremental: false`, so no run reads or writes a tsbuildinfo. Corpus
 * packages are vue and @types/node only (tools/benchmarks/typecheck-corpora):
 * every other import is unresolved in every lane alike.
 */

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
    tsconfig: {
      compilerOptions: shared,
      include: ["./src/**/*.ts", "./src/**/*.vue"],
      exclude: ["./src/**/*.test.ts", "./src/**/*.spec.ts", "./src/**/__tests__/**"],
    },
  },
  {
    // The library sources `vue-tsc --noEmit` checks at the root, with the
    // compiler options `nuxt-module-build prepare` writes to .nuxt/tsconfig.json
    // (@nuxt/kit 4.5); `#imports` and `#build/*` stay unresolved without it.
    id: "nuxt-ui",
    label: "nuxt/ui src",
    submodule: "tests/_fixtures/_git/nuxt-ui",
    source: "tests/_fixtures/_git/nuxt-ui",
    copy: ["src"],
    sfcs: 187,
    tsconfig: {
      compilerOptions: {
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
      include: ["./src/**/*"],
    },
  },
  {
    // packages/core `type-check`: tsconfig.check.json over tsconfig.app.json
    // over @vue/tsconfig 0.7's tsconfig.dom.json.
    id: "reka-ui",
    label: "reka-ui core",
    submodule: "tests/_fixtures/_git/reka-ui",
    source: "tests/_fixtures/_git/reka-ui/packages/core",
    copy: ["src", "env.d.ts"],
    sfcs: 454,
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
