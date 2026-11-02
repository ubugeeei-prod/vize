import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { asArray, asRecord, asString } from "../../../support/json-value.ts";
import type { Expected, Profile, SourceFile } from "./contract.ts";
export const declarationFileName = "declaration-11364.json";
const value = asRecord(
  JSON.parse(readFileSync(new URL(declarationFileName, import.meta.url), "utf8")),
);
assert.equal(value.schema, "vize.q16.public.declaration11364.v1");
assert.equal(
  value.sourceProtocolSha256,
  "50e5431858b0e5e0aa360b7db5f0602572c211e4ce045a1d11e325d59fc38d55",
);
assert.equal(value.knownGoodSource, "07d1feb1bb19d862b30aea10fe3465a08a8b1239");
assert.equal(value.causalFix, null);
assert.equal(value.producerStreamsOwned, false);
assert.equal(value.sourceDeclarationsUsedAsPublicGolden, false);
export const declarationData = {
  knownGoodSource: asString(value.knownGoodSource),
  causalFix: null,
  protocolSha256: asString(value.sourceProtocolSha256),
  packageJson: asString(value.packageJsonOriginal),
  config: asString(value.configOriginal),
  producer: asString(value.producerSource),
  consumer: asString(value.consumerSource),
  producerArgs: asArray(value.producerArgs).map((item) => asString(item)),
  consumerArgs: asArray(value.consumerArgs).map((item) => asString(item)),
  outputFiles: asArray(value.outputFiles).map((item) => asString(item)),
  states: asArray(value.states).map((item) => {
    const row = asRecord(item),
      expected = asRecord(row.expected);
    assert(Number.isSafeInteger(expected.status));
    return {
      id: asString(row.id),
      phase: asString(row.phase),
      source: asString(row.source),
      expected: {
        status: Number(expected.status),
        stdout: asString(expected.stdout),
        stderr: asString(expected.stderr),
      },
    };
  }),
};
assert.deepEqual(declarationData.producerArgs, [
  "npx",
  "vize",
  "check",
  "--declaration",
  "--declaration-dir",
  "../out",
  "src/Foo.vue",
]);
assert.deepEqual(declarationData.consumerArgs, [
  "npx",
  "vue-tsc",
  "--noEmit",
  "-p",
  "tsconfig.json",
]);
assert.deepEqual(declarationData.outputFiles, ["Foo.vue.d.ts", "__vize_helpers.d.ts"]);
assert.deepEqual(
  declarationData.states.map((row) => row.expected.status),
  [0, 2, 2, 0],
);
assert.equal(declarationData.states[0]!.source, declarationData.states[3]!.source);
export type DeclarationState = { id: string; phase: string; source: string; expected: Expected };
export function declarationInputs(profile: Profile, producer: boolean): SourceFile[] {
  assert.equal(declarationData.packageJson.split('"vize": "0.638.0"').length, 2);
  return [
    {
      path: "package.json",
      source: declarationData.packageJson.replace(
        '"vize": "0.638.0"',
        '"vize": "' + profile.version + '"',
      ),
    },
    { path: "tsconfig.json", source: declarationData.config },
    {
      path: producer ? "src/Foo.vue" : "src/Probe.vue",
      source: producer ? declarationData.producer : declarationData.consumer,
    },
  ];
}
