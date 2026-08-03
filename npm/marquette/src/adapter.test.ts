import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import type { ApplicationMarquette } from "./model.js";
import {
  compareAdapterCapabilities,
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
});

void test("matches shared semantic validation diagnostics", async () => {
  const manifest = await read<AdapterCapabilityManifest>("adapter-manifest-invalid.json");
  const expected = await read<readonly AdapterCapabilityDiagnostic[]>(
    "adapter-manifest-invalid.expected.json",
  );

  assert.deepEqual(validateAdapterCapabilityManifest(manifest), expected);
});

void test("strict parsing rejects unknown and malformed fields before negotiation", async () => {
  const unknown = await read<unknown>("adapter-manifest-unknown-field.json");
  assert.throws(
    () => parseAdapterCapabilityManifest(unknown),
    /capabilities\.0 has unknown field unexpected/,
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
