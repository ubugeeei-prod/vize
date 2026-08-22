import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  omitProgramEvidence,
  resolveTsgoBinary,
  runVizeCheck,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";

const appPath = "docs/app/components/content/examples/tabs/TabsRouteQueryExample.vue";
const cleanExpression = "route.query.tab";
const brokenExpression = "route.missing";

test("Nuxt best-effort checking works without a root tsconfig", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "nuxt-ui", includePaths: [appPath], outsideRepository: true },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("node_modules/@nuxt/ui/package.json", packageManifest);
      fixture.write("node_modules/@nuxt/ui/index.d.ts", nuxtUiDeclaration);
      fixture.write("node_modules/nuxt/package.json", nuxtPackageManifest);
      fixture.write("node_modules/nuxt/app.d.ts", nuxtAppDeclaration);
      fixture.write(".nuxt/imports.d.ts", generatedImportsDeclaration);
      fixture.write(".nuxt/components.d.cts", generatedComponentsDeclaration);
      fixture.write(".nuxt/tsconfig.json", json(generatedTsconfig));
      fixture.write("nuxt.config.ts", "export default defineNuxtConfig({})\n");

      assert.equal(nearestAncestorTsconfig(fixture.workspaceDir), null);
      assertClean(runWithoutTsconfig(fixture.workspaceDir, corsaPath));

      fixture.applyExactPatch(appPath, cleanExpression, brokenExpression);
      assertBroken(runWithoutTsconfig(fixture.workspaceDir, corsaPath));

      fixture.applyExactPatch(appPath, brokenExpression, cleanExpression);
      assertClean(runWithoutTsconfig(fixture.workspaceDir, corsaPath));
      assert.equal(nearestAncestorTsconfig(fixture.workspaceDir), null);
    },
  );
});

function runWithoutTsconfig(workspaceDir: string, corsaPath: string): VizeCheckResult {
  return runVizeCheck(workspaceDir, corsaPath, [appPath], null);
}

function assertClean(result: VizeCheckResult): void {
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.deepEqual(omitProgramEvidence(result.report), {
    errorCount: 0,
    fileCount: 1,
    files: [{ diagnostics: [], file: appPath }],
    warningCount: 0,
  });
}

function assertBroken(result: VizeCheckResult): void {
  assert.equal(result.status, 1, result.stderr || result.stdout);
  assert.equal(result.report.errorCount, 1, JSON.stringify(result.report));
  assert.equal(result.report.fileCount, 1, JSON.stringify(result.report));
  assert.equal(result.report.warningCount, 0, JSON.stringify(result.report));
  assert.equal(result.report.files.length, 1, JSON.stringify(result.report));
  assert.equal(result.report.files[0]?.file, appPath, JSON.stringify(result.report));
  assert.deepEqual(result.report.files[0]?.diagnostics, [
    "error:22:19 [TS2339] Property 'missing' does not exist on type 'NuxtRoute'.",
  ]);
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function nearestAncestorTsconfig(directory: string): string | null {
  let current = path.resolve(directory);
  while (true) {
    const candidate = path.join(current, "tsconfig.json");
    if (fs.existsSync(candidate)) return candidate;
    const parent = path.dirname(current);
    if (parent === current) return null;
    current = parent;
  }
}

const packageManifest = json({ name: "@nuxt/ui", type: "module", types: "./index.d.ts" });

const nuxtUiDeclaration = `export type TabsItem = {
  label: string
  icon: string
  value: string
}

export const UTabs: new () => {
  $props: {
    modelValue?: string
    'onUpdate:modelValue'?: (value: string) => void
    content?: boolean
    items?: TabsItem[]
    class?: unknown
  }
}
`;

const nuxtPackageManifest = json({
  name: "nuxt",
  type: "module",
  exports: { "./app": { types: "./app.d.ts", default: "./app.js" } },
});

const nuxtAppDeclaration = `export interface NuxtRoute {
  query: Record<string, unknown>
}

export interface NuxtRouter {
  push(to: { path: string; query: Record<string, unknown>; hash?: string }): Promise<void>
}

export function useRoute(): NuxtRoute
export function useRouter(): NuxtRouter
`;

const generatedImportsDeclaration = `
declare global {
  const computed: typeof import('vue')['computed']
  const useRoute: typeof import('nuxt/app')['useRoute']
  const useRouter: typeof import('nuxt/app')['useRouter']
}

export {}
`;

const generatedComponentsDeclaration = `declare module 'vue' {
  export interface GlobalComponents {
    UTabs: typeof import('@nuxt/ui')['UTabs']
  }
}

export {}
`;

const generatedTsconfig = {
  compilerOptions: {
    lib: ["ES2022", "DOM", "DOM.Iterable"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
  },
  include: ["./imports.d.ts", "./components.d.cts", `../${appPath}`],
};
