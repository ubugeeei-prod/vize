import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { patchNpmxRegistryFixtures } from "../_helpers/app-fixture-runtime.ts";

test("npmx registry fixture patch stabilizes server-side package metadata", async (t) => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npmx-registry-"));
  t.after(() => fs.rmSync(fixtureRoot, { recursive: true, force: true }));

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

export function useResolvedVersion() {
  return useAsyncData(
    () => 'resolved-version:vue:latest',
    async () => {
      const url = 'https://npm.antfu.dev/vue'
      const data = await $fetch<ResolvedPackageVersion>(url)
      return data.version
    },
  )
}
`,
  );
  writeText(
    path.join(fixtureRoot, "server/utils/npm.ts"),
    `import { findMaxSatisfying } from 'verkit'

export const fetchNpmPackage = defineCachedFunction(
  async (name: string): Promise<Packument> => {
    const encodedName = encodePackageName(name)
    return await $fetch<Packument>(\`\${NPM_REGISTRY}/\${encodedName}\`)
  },
)

void findMaxSatisfying
`,
  );

  patchNpmxRegistryFixtures(fixtureRoot);
  const oncePatchedSources = readPatchedConsumers(fixtureRoot);
  patchNpmxRegistryFixtures(fixtureRoot);
  assert.deepEqual(readPatchedConsumers(fixtureRoot), oncePatchedSources);

  const copiedFixturePath = path.join(fixtureRoot, "shared/utils/__vize-e2e-npm-fixtures.ts");
  const copiedFixture = (await import(
    pathToFileURL(copiedFixturePath).href
  )) as typeof import("../_fixtures/npmx-e2e-registry-fixtures.ts");

  const cachedManifest = copiedFixture.resolveVizeE2ENpmRegistryCachedResponse<{
    dist: { unpackedSize: number };
  }>("/vue/3.5.29");
  assert.equal(cachedManifest.handled, true);
  if (!cachedManifest.handled) throw new Error("expected copied vue manifest fixture");
  assert.equal(cachedManifest.data.data.dist.unpackedSize, 2_600_000);
  assert.equal(cachedManifest.data.isStale, false);
  assert.equal(cachedManifest.data.cachedAt, null);
  assert.deepEqual(copiedFixture.resolveVizeE2ENpmRegistryCachedResponse("/missing"), {
    handled: false,
  });
  assert.deepEqual(
    copiedFixture.resolveVizeE2EFastNpmMetaVersion("https://npm.antfu.dev/@vue%2Fcompiler-sfc"),
    { handled: true, data: "3.5.29" },
  );
  const packument = copiedFixture.resolveVizeE2ENpmPackument<{
    versions: Record<string, { dist: { unpackedSize: number } }>;
  }>("vue");
  assert.equal(packument.handled, true);
  if (!packument.handled) throw new Error("expected copied vue packument fixture");
  assert.equal(packument.data.versions["3.5.29"]?.dist.unpackedSize, 2_600_000);
  assert.deepEqual(copiedFixture.resolveVizeE2ENpmPackument("missing-package"), {
    handled: false,
  });
  assert.deepEqual(
    copiedFixture.resolveVizeE2EFastNpmMetaFixture("https://npm.antfu.dev/%E0%A4%A"),
    {
      handled: false,
    },
  );
});

function readPatchedConsumers(fixtureRoot: string): string[] {
  return [
    fs.readFileSync(path.join(fixtureRoot, "app/plugins/npm.ts"), "utf-8"),
    fs.readFileSync(path.join(fixtureRoot, "app/composables/npm/useResolvedVersion.ts"), "utf-8"),
    fs.readFileSync(path.join(fixtureRoot, "server/utils/npm.ts"), "utf-8"),
  ];
}

function writeText(filePath: string, content: string): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}
