import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { patchNpmxRegistryFixtures } from "../_helpers/app-fixture-runtime.ts";
import {
  collectPatchedBehavior,
  installNuxtDoubles,
  NETWORK_CACHED_FETCH_RESULT,
  NETWORK_FETCH_RESULT,
  NPM_REGISTRY,
  writeNpmxFixtureApp,
} from "./npmx-e2e-registry-behavior.ts";

test("npmx registry fixture patch serves deterministic package metadata to server callers", async (t) => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npmx-registry-"));
  t.after(() => fs.rmSync(fixtureRoot, { recursive: true, force: true }));

  const originalFixturesEnv = process.env.NUXT_TEST_FIXTURES;
  t.after(() => {
    if (originalFixturesEnv === undefined) {
      delete process.env.NUXT_TEST_FIXTURES;
    } else {
      process.env.NUXT_TEST_FIXTURES = originalFixturesEnv;
    }
  });

  writeNpmxFixtureApp(fixtureRoot);
  patchNpmxRegistryFixtures(fixtureRoot);

  assert.equal(
    fs.existsSync(path.join(fixtureRoot, "shared/utils/__vize-e2e-npm-fixtures.ts")),
    true,
  );

  const doubles = installNuxtDoubles();
  t.after(() => doubles.restore());
  const behavior = await collectPatchedBehavior(fixtureRoot, 1, doubles);

  const ssrManifest = behavior["ssr-version-manifest"];
  assert.ok(ssrManifest);
  assert.deepEqual(ssrManifest.cachedFetchCalls, []);
  assert.deepEqual(ssrManifest.fetchCalls, []);
  const manifest = ssrManifest.result as {
    data: { version: string; dist: { unpackedSize: number; fileCount: number } };
    isStale: boolean;
    cachedAt: unknown;
  };
  assert.equal(manifest.isStale, false);
  assert.equal(manifest.cachedAt, null);
  assert.equal(manifest.data.version, "3.5.29");
  assert.equal(manifest.data.dist.unpackedSize, 2_600_000);
  assert.equal(manifest.data.dist.fileCount, 44);

  const ssrPackument = behavior["ssr-packument"];
  assert.ok(ssrPackument);
  assert.deepEqual(ssrPackument.cachedFetchCalls, []);
  const packument = ssrPackument.result as {
    data: { name: string; "dist-tags": { latest: string } };
  };
  assert.equal(packument.data.name, "vue");
  assert.equal(packument.data["dist-tags"].latest, "3.5.29");

  const ssrCustomBaseURL = behavior["ssr-custom-base-url"];
  assert.ok(ssrCustomBaseURL);
  assert.deepEqual(ssrCustomBaseURL.result, NETWORK_CACHED_FETCH_RESULT);
  assert.deepEqual(ssrCustomBaseURL.cachedFetchCalls, [
    { url: "/vue/3.5.29", options: { baseURL: "https://example.test" }, ttl: undefined },
  ]);

  const ssrUnhandled = behavior["ssr-unhandled-package"];
  assert.ok(ssrUnhandled);
  assert.deepEqual(ssrUnhandled.result, NETWORK_CACHED_FETCH_RESULT);
  assert.deepEqual(ssrUnhandled.cachedFetchCalls, [
    { url: "/unknown-fixture-package", options: { baseURL: NPM_REGISTRY }, ttl: 60 },
  ]);

  const ssrMalformed = behavior["ssr-malformed-encoding"];
  assert.ok(ssrMalformed);
  assert.deepEqual(ssrMalformed.result, NETWORK_CACHED_FETCH_RESULT);
  assert.deepEqual(ssrMalformed.cachedFetchCalls, [
    { url: "/%E0%A4%A", options: { baseURL: NPM_REGISTRY }, ttl: undefined },
  ]);

  const clientManifest = behavior["client-version-manifest"];
  assert.ok(clientManifest);
  assert.deepEqual(clientManifest.result, NETWORK_CACHED_FETCH_RESULT);
  assert.deepEqual(clientManifest.cachedFetchCalls, [
    { url: "/vue/3.5.29", options: { baseURL: NPM_REGISTRY }, ttl: undefined },
  ]);

  const disabledManifest = behavior["ssr-version-manifest-fixtures-disabled"];
  assert.ok(disabledManifest);
  assert.deepEqual(disabledManifest.result, NETWORK_CACHED_FETCH_RESULT);
  assert.deepEqual(disabledManifest.cachedFetchCalls, [
    { url: "/vue/3.5.29", options: { baseURL: NPM_REGISTRY }, ttl: undefined },
  ]);

  const ssrResolvedVersion = behavior["ssr-resolved-version"];
  assert.ok(ssrResolvedVersion);
  assert.equal(ssrResolvedVersion.result, "3.5.29");
  assert.deepEqual(ssrResolvedVersion.fetchCalls, []);

  const ssrResolvedVersionUnhandled = behavior["ssr-resolved-version-unhandled"];
  assert.ok(ssrResolvedVersionUnhandled);
  assert.equal(ssrResolvedVersionUnhandled.result, "0.0.0-network");
  assert.deepEqual(ssrResolvedVersionUnhandled.fetchCalls, [
    "https://npm.antfu.dev/unknown-fixture-package",
  ]);

  const ssrResolvedVersionMalformed = behavior["ssr-resolved-version-malformed-encoding"];
  assert.ok(ssrResolvedVersionMalformed);
  assert.equal(ssrResolvedVersionMalformed.result, "0.0.0-network");
  assert.deepEqual(ssrResolvedVersionMalformed.fetchCalls, ["https://npm.antfu.dev/%E0%A4%A"]);

  const clientResolvedVersion = behavior["client-resolved-version"];
  assert.ok(clientResolvedVersion);
  assert.equal(clientResolvedVersion.result, "0.0.0-network");
  assert.deepEqual(clientResolvedVersion.fetchCalls, ["https://npm.antfu.dev/vue"]);

  const serverPackument = behavior["server-packument"];
  assert.ok(serverPackument);
  assert.deepEqual(serverPackument.fetchCalls, []);
  const serverPackumentData = serverPackument.result as {
    name: string;
    versions: Record<string, { dist: { unpackedSize: number } }>;
  };
  assert.equal(serverPackumentData.name, "vue");
  assert.equal(serverPackumentData.versions["3.5.29"]?.dist.unpackedSize, 2_600_000);

  const serverScopedPackument = behavior["server-scoped-packument"];
  assert.ok(serverScopedPackument);
  assert.deepEqual(serverScopedPackument.fetchCalls, []);
  assert.equal((serverScopedPackument.result as { name: string }).name, "@vue/compiler-sfc");

  const serverScopedFullyEncoded = behavior["server-scoped-packument-fully-encoded"];
  assert.ok(serverScopedFullyEncoded);
  assert.deepEqual(serverScopedFullyEncoded.fetchCalls, []);
  assert.equal((serverScopedFullyEncoded.result as { name: string }).name, "@vue/compiler-sfc");

  const serverUnhandled = behavior["server-unhandled-package"];
  assert.ok(serverUnhandled);
  assert.deepEqual(serverUnhandled.result, NETWORK_FETCH_RESULT);
  assert.deepEqual(serverUnhandled.fetchCalls, [`${NPM_REGISTRY}/unknown-fixture-package`]);

  const serverDisabled = behavior["server-packument-fixtures-disabled"];
  assert.ok(serverDisabled);
  assert.deepEqual(serverDisabled.result, NETWORK_FETCH_RESULT);
  assert.deepEqual(serverDisabled.fetchCalls, [`${NPM_REGISTRY}/vue`]);

  patchNpmxRegistryFixtures(fixtureRoot);
  const repatchedBehavior = await collectPatchedBehavior(fixtureRoot, 2, doubles);
  assert.deepEqual(repatchedBehavior, behavior);
});
