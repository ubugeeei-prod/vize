import assert from "node:assert/strict";
import { cp, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Browser } from "playwright";
import { publicNative, rejectOverrides } from "./installed.ts";
import { createVrtFixture, type VrtFixture } from "./vrt_fixtures.ts";
import { cleanup, evidence, inventory } from "./vrt_artifacts.ts";
import { observeVrtApi } from "./vrt_api.ts";
import { observeHostedVrt } from "./vrt_hosted.ts";
import { VrtSecrets, writeVrtDiagnostic, type VrtPhase } from "./vrt_secrets.ts";
import { observeTrustedInstalledVrt } from "./vrt_trusted_observation.ts";
import { assertTrustedVrtSurfacesClosed } from "./vrt_trusted_surface.ts";
import { assertPublishedVrtChildrenClosed } from "./vrt_service_process.ts";

rejectOverrides();
const [version, output] = process.argv.slice(2);
assert.ok(version && output);
const secrets = new VrtSecrets(writeVrtDiagnostic);
secrets.stage("observer-entry");
try {
  const e = secrets.evidence(await evidence(path.join(path.dirname(output), "musea-vrt")));
  const owned: { browser?: Browser } = {};
  let fixture: VrtFixture | undefined;
  let provider: ReturnType<typeof publicNative> | undefined;
  let failure: unknown;
  let stage: VrtPhase = "installed-native-custody";
  try {
    secrets.stage(stage);
    provider = publicNative(version);
    await e.json("provider.json", {
      version,
      packages: provider.packages,
      loaded: provider.loaded,
    });
    stage = "sealed-delivered-inputs";
    secrets.stage(stage);
    fixture = createVrtFixture(process.cwd());
    await e.json("inputs.json", {
      settings: fixture.settings,
      arts: fixture.arts,
      delay: fixture.delay,
    });
    stage = "public-chromium";
    secrets.stage(stage);
    const { chromium } = await import("playwright");
    const browser = (owned.browser = await chromium.launch({ headless: true }));
    stage = "configured-public-api";
    secrets.stage(stage);
    await observeVrtApi(browser, fixture, e);
    stage = "mandatory-public-hosted-cli-and-audits";
    secrets.stage(stage);
    await observeHostedVrt(browser, fixture, e, secrets);
    stage = "mandatory-normally-trusted-public-https";
    secrets.stage(stage);
    const assetDirectory = process.env.VRT_TUNNEL_DIRECTORY;
    assert.ok(
      assetDirectory && path.isAbsolute(assetDirectory),
      "Approved owned pinned asset required",
    );
    await observeTrustedInstalledVrt(
      browser,
      {
        consumer: process.cwd(),
        assetDirectory,
        binary: path.join(assetDirectory, "cloudflared-linux-amd64"),
      },
      e,
      secrets,
    );
    stage = "final-installed-custody";
    secrets.stage(stage);
    const final = publicNative(version);
    assert.deepEqual(final.packages, provider.packages);
    assert.deepEqual(final.loaded, provider.loaded);
  } catch (error) {
    secrets.failed();
    const sanitizedError = secrets.sanitizeError(error);
    failure = sanitizedError;
    await e.json("failure.json", {
      stage,
      error: String(sanitizedError),
      stack: sanitizedError instanceof Error ? sanitizedError.stack : undefined,
    });
  } finally {
    secrets.stage("observer-cleanup");
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
      secrets.failed("observer-cleanup");
      const sanitizedError = secrets.sanitizeError(error);
      e.records.push({ phase: "cleanup", error: String(sanitizedError) });
      failure ??= sanitizedError;
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
} catch (error) {
  secrets.failed();
  console.error(secrets.sanitizeError(error));
  process.exitCode ||= 1;
} finally {
  try {
    await secrets.closeBarrier(assertTrustedVrtSurfacesClosed, assertPublishedVrtChildrenClosed);
    await secrets.seal(path.join(path.dirname(output), "vrt.log"), process.env.VRT_RETAINED_OUTPUT);
  } catch {
    secrets.failed();
    console.error("VRT retained evidence refused; log publication withheld");
    process.exitCode ||= 1;
  } finally {
    secrets.erase();
  }
}
