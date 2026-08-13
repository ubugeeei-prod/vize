import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import type { ApplicationMarquette } from "./model.js";
import {
  NATIVE_ENGINE_CAPABILITY_IDS,
  NATIVE_ENGINE_CAPABILITY_VERSION,
  compareAdapterCapabilities,
  nativeEngineCapabilityProfile,
  negotiateAdapterCapabilities,
  parseAdapterCapabilityManifest,
  validateAdapterCapabilityManifest,
  type AdapterCapabilityCompatibilityReport,
  type AdapterCapabilityDiagnostic,
  type AdapterCapabilityManifest,
  type AdapterCapabilityNegotiation,
} from "./adapter.js";

const fixture = (name: string) =>
  new URL(`../../../tests/fixtures/marquette/${name}`, import.meta.url);

async function read<T>(name: string): Promise<T> {
  return JSON.parse(await readFile(fixture(name), "utf8")) as T;
}

interface NegotiationFixture {
  readonly contract: ApplicationMarquette;
  readonly cases: readonly {
    readonly name: string;
    readonly required: readonly string[];
    readonly manifest: AdapterCapabilityManifest;
    readonly expected: AdapterCapabilityNegotiation;
  }[];
}

interface CompatibilityCase {
  readonly name: string;
  readonly previous: AdapterCapabilityManifest;
  readonly next: AdapterCapabilityManifest;
  readonly expected: AdapterCapabilityCompatibilityReport;
}

void test("native engine profile matches the shared contract and returns fresh values", async () => {
  const expected = await read<AdapterCapabilityManifest>("native-engine-capability-profile.json");
  const first = nativeEngineCapabilityProfile();
  const manifest = {
    formatVersion: 1,
    adapter: "fixture.native",
    capabilities: first,
  } as const satisfies AdapterCapabilityManifest;

  assert.deepEqual(manifest, expected);
  assert.deepEqual(
    first.map(({ id }) => id),
    NATIVE_ENGINE_CAPABILITY_IDS,
  );
  assert.equal(first.length, 8);
  assert.ok(
    first.every(
      ({ minVersion, maxVersion }) =>
        minVersion === NATIVE_ENGINE_CAPABILITY_VERSION &&
        maxVersion === NATIVE_ENGINE_CAPABILITY_VERSION,
    ),
  );
  assert.deepEqual(validateAdapterCapabilityManifest(manifest), []);

  const contract: ApplicationMarquette = {
    application: "native-profile",
    capabilities: Object.fromEntries(
      NATIVE_ENGINE_CAPABILITY_IDS.map((id) => [
        id,
        { id, description: "Native engine contract", version: 1 },
      ]),
    ),
  };
  assert.equal(
    negotiateAdapterCapabilities(contract, NATIVE_ENGINE_CAPABILITY_IDS, manifest).compatible,
    true,
  );

  const missingAnimation = {
    ...manifest,
    capabilities: manifest.capabilities.filter(({ id }) => id !== "native.animation"),
  };
  assert.deepEqual(
    negotiateAdapterCapabilities(contract, NATIVE_ENGINE_CAPABILITY_IDS, missingAnimation)
      .mismatches,
    [
      {
        code: "missing-capability",
        capability: "native.animation",
        path: "capabilities.native.animation",
        message: "adapter does not support the required capability",
        requiredVersion: 1,
      },
    ],
  );

  (first[0] as { id: string }).id = "mutated";
  assert.equal(nativeEngineCapabilityProfile()[0]?.id, "native.rendering");
});

void test("matches shared negotiation fixtures and input permutations", async () => {
  const fixture = await read<NegotiationFixture>("adapter-negotiation.json");
  const results = fixture.cases.map((case_) => {
    const contractBefore = structuredClone(fixture.contract);
    const manifestBefore = structuredClone(case_.manifest);
    const actual = negotiateAdapterCapabilities(fixture.contract, case_.required, case_.manifest);

    assert.deepEqual(actual, case_.expected, case_.name);
    assert.deepEqual(fixture.contract, contractBefore, `${case_.name}: contract mutated`);
    assert.deepEqual(case_.manifest, manifestBefore, `${case_.name}: manifest mutated`);
    return actual;
  });

  assert.equal(JSON.stringify(results[0]), JSON.stringify(results[1]));
  assert.equal(results[2]?.compatible, true, "inclusive bounds must pass");
  assert.equal(results[3]?.mismatches[0]?.code, "unknown-requirement");
  assert.equal(results[3]?.mismatches[0]?.path, "capabilities.unknown.capability");
});

void test("matches shared semantic validation diagnostics", async () => {
  const input = await read<unknown>("adapter-manifest-invalid.json");
  const expected = await read<readonly AdapterCapabilityDiagnostic[]>(
    "adapter-manifest-invalid.expected.json",
  );
  const manifest = parseAdapterCapabilityManifest(input);

  assert.deepEqual(validateAdapterCapabilityManifest(manifest), expected);
});

void test("strict parsing rejects unknown and malformed fields before negotiation", async () => {
  const unknown = await read<unknown>("adapter-manifest-unknown-field.json");
  assert.throws(
    () => parseAdapterCapabilityManifest(unknown),
    /capabilities\.0 has unknown field zUnexpected/,
  );
  assert.throws(
    () => parseAdapterCapabilityManifest({ formatVersion: "1", adapter: "fixture.adapter" }),
    /formatVersion must be a number/,
  );
  assert.throws(
    () =>
      parseAdapterCapabilityManifest({
        adapter: "fixture.adapter",
        capabilities: [{ id: "missing-min", maxVersion: 1 }],
      }),
    /capabilities\.0\.minVersion must be a safe integer/,
  );
  assert.throws(
    () =>
      parseAdapterCapabilityManifest({
        adapter: "fixture.adapter",
        capabilities: [{ id: "negative", minVersion: -1, maxVersion: 1 }],
      }),
    /capabilities\.0\.minVersion must be a safe integer/,
  );
});

void test("matches the shared adapter compatibility matrix", async () => {
  const cases = await read<readonly CompatibilityCase[]>("adapter-compatibility.json");
  for (const case_ of cases) {
    assert.deepEqual(
      compareAdapterCapabilities(case_.previous, case_.next),
      case_.expected,
      case_.name,
    );
  }
});

void test("invalid manifests fail closed without reporting adapter mismatches", () => {
  const marquette: ApplicationMarquette = {
    application: "invalid-manifest",
    capabilities: { known: { id: "known", description: "Known" } },
  };
  const result = negotiateAdapterCapabilities(marquette, ["known"], {
    adapter: "fixture.adapter",
    capabilities: [
      { id: "known", minVersion: 2, maxVersion: 1 },
      { id: "known", minVersion: 1, maxVersion: 2 },
    ],
  });

  assert.equal(result.compatible, false);
  assert.deepEqual(result.mismatches, []);
  assert.deepEqual(
    result.diagnostics.map(({ code }) => code),
    ["invalid-version-range", "duplicate-capability"],
  );
});
