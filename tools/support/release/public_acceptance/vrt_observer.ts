import assert from "node:assert/strict";
import { cp, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Browser } from "playwright";
import { publicNative, rejectOverrides } from "./installed.ts";
import { createVrtFixture, type VrtFixture } from "./vrt_fixtures.ts";
import { cleanup, evidence, inventory } from "./vrt_artifacts.ts";
import { observeVrtApi } from "./vrt_api.ts";
import { observeHostedVrt } from "./vrt_hosted.ts";

rejectOverrides();
const [version, output] = process.argv.slice(2);
assert.ok(version && output);
const e = await evidence(path.join(path.dirname(output), "musea-vrt"));
const owned: { browser?: Browser } = {};
let fixture: VrtFixture | undefined;
let provider: ReturnType<typeof publicNative> | undefined;
let failure: unknown;
let stage = "installed-native-custody";
try {
  provider = publicNative(version);
  await e.json("provider.json", { version, packages: provider.packages, loaded: provider.loaded });
  stage = "sealed-delivered-inputs";
  fixture = createVrtFixture(process.cwd());
  await e.json("inputs.json", {
    settings: fixture.settings,
    arts: fixture.arts,
    delay: fixture.delay,
  });
  stage = "public-chromium";
  const { chromium } = await import("playwright");
  const browser = (owned.browser = await chromium.launch({ headless: true }));
  stage = "configured-public-api";
  await observeVrtApi(browser, fixture, e);
  stage = "mandatory-public-hosted-cli-and-audits";
  await observeHostedVrt(browser, fixture, e);
  stage = "final-installed-custody";
  const final = publicNative(version);
  assert.deepEqual(final.packages, provider.packages);
  assert.deepEqual(final.loaded, provider.loaded);
} catch (error) {
  failure = error;
  await e.json("failure.json", {
    stage,
    error: String(error),
    stack: error instanceof Error ? error.stack : undefined,
  });
} finally {
  try {
    await cleanup(e, "observer-cleanup", [
      { name: "chromium", run: async () => owned.browser?.close() },
      {
        name: "physical-project",
        run: async () => {
          if (fixture) {
            await cp(fixture.root, path.join(e.output, "physical-project"), { recursive: true });
            await e.json("physical-project-inventory.json", await inventory(fixture.root));
          }
        },
      },
    ]);
  } catch (error) {
    e.records.push({ phase: "cleanup", error: String(error) });
    failure ??= error;
  }
  await e.json("observations.json", { version, stage, records: e.records, success: !failure });
}
if (failure) {
  console.error(failure);
  process.exitCode = 1;
} else {
  assert.ok(provider);
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(
    output,
    JSON.stringify(
      {
        schema: "vize-public-installed-musea-vrt-v1",
        version,
        packages: provider.packages,
        loaded: provider.loaded,
        observations: e.records,
        artifacts: e.output,
        success: true,
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  console.log(JSON.stringify({ version, scope: "installed-public-musea-vrt", success: true }));
}
