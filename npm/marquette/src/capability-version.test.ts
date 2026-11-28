import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import {
  compareAdapterCapabilities,
  negotiateAdapterCapabilities,
  parseAdapterCapabilityManifest,
  validateAdapterCapabilityManifest,
} from "./adapter.js";
import { validateApplicationMarquette } from "./validate.js";

interface VersionBoundary {
  readonly name: string;
  readonly version: number | null;
  readonly parsed: boolean;
  readonly valid: boolean;
}

const manifest = (version: number) => ({
  adapter: "native.fixture",
  capabilities: [{ id: "native.feature", minVersion: version, maxVersion: version }],
});

const application = (version: number) => ({
  application: "numeric-boundary",
  capabilities: {
    "native.feature": { id: "native.feature", description: "A native feature", version },
  },
});

void test("matches portable numeric fixtures shared with native contract consumers", async () => {
  const cases = JSON.parse(
    await readFile(
      new URL(
        "../../../tests/fixtures/marquette/capability-version-boundaries.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as VersionBoundary[];
  for (const fixture of cases) {
    // The fixture deliberately crosses the typed boundary with malformed JSON values.
    const version = fixture.version as number;
    const value = manifest(version);
    if (fixture.parsed) {
      assert.deepEqual(parseAdapterCapabilityManifest(value), value, fixture.name);
    } else {
      assert.throws(() => parseAdapterCapabilityManifest(value), TypeError, fixture.name);
    }
    assert.equal(
      validateAdapterCapabilityManifest(value).length === 0,
      fixture.valid,
      fixture.name,
    );
    assert.equal(
      validateApplicationMarquette(application(version)).length === 0,
      fixture.valid,
      fixture.name,
    );
    assert.equal(
      negotiateAdapterCapabilities(application(version), ["native.feature"], value).compatible,
      fixture.valid,
      fixture.name,
    );
    assert.equal(
      negotiateAdapterCapabilities(application(version), ["native.feature"], {
        adapter: "native.fixture",
        capabilities: [{ id: "native.feature", minVersion: 1, maxVersion: 0xffff_ffff }],
      }).compatible,
      fixture.valid,
      fixture.name,
    );
  }
});

void test("typed non-finite and fractional bounds cannot pass validation or compatibility", () => {
  for (const version of [
    Number.NaN,
    Number.POSITIVE_INFINITY,
    Number.NEGATIVE_INFINITY,
    1.5,
    0xffff_ffff + 1,
  ]) {
    const value = manifest(version);
    assert.equal(validateAdapterCapabilityManifest(value).length > 0, true);
    assert.equal(
      negotiateAdapterCapabilities(application(1), ["native.feature"], value).compatible,
      false,
    );
    const comparison = compareAdapterCapabilities(manifest(1), value);
    assert.ok(comparison.nextDiagnostics.length > 0);
    assert.deepEqual(comparison.changes, []);
  }
});

void test("invalid application requirement versions fail closed without serializing unsafe numbers", () => {
  for (const version of [Number.NaN, Number.POSITIVE_INFINITY, 0, -1, 1.5, 0xffff_ffff + 1]) {
    const result = negotiateAdapterCapabilities(
      application(version),
      ["native.feature"],
      manifest(1),
    );
    assert.equal(result.compatible, false);
    assert.deepEqual(result.mismatches, [
      {
        code: "invalid-requirement-version",
        capability: "native.feature",
        path: "capabilities.native.feature",
        message: "application capability version must be a positive unsigned 32-bit integer",
      },
    ]);
    assert.deepEqual(JSON.parse(JSON.stringify(result)), result);
  }
});
