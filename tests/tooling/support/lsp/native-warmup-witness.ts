import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "../../../differential/harness.mjs";

export const WARMUP_WITNESS_PATH = "tests/tooling/lsp-native-diagnostic-warmup.test.ts";
export const HISTORICAL_WARMUP_SHA =
  "ba8d53759b8f5fdf771dc8ee0169477ef5bdfe8357c8fa7c5501346eef62031e";
const MAIN_WARMUP_SHA = "3f5f24dac1732cf8298d8ec03e5678eafd475581cd05d7910eff0b4c6c35854c";
const DIRECTORY = "tests/_fixtures/tooling/native-diagnostic-warmup-witness-11815";
const ORIGINAL_CHECK = `assert.equal(
  sha256(fs.readFileSync(path.join(root, fixture.provenance.warmupWitness.path))),
  fixture.provenance.warmupWitness.sha256,
  "the full actual stdio transition witness must retain its pinned source custody",
);`;
const RESUME_CHANGES = [
  [
    `      gate.assertStopped();
      gate.resume();
      assert.equal(
        nativeErrors(
          await wait(first, (packet) => packet.version === 2 && nativeErrors(packet).length === 8),
        ).length,
        8,
      );
      assert.equal(nativeErrors(await wait(second, (packet) => packet.version === 1)).length, 8);
`,
    `      gate.assertStopped();
      const resumedAt = notifications.length;
      gate.resume();
      assert.equal(
        nativeErrors(
          await wait(
            first,
            (packet) => packet.version === 2 && nativeErrors(packet).length === 8,
            resumedAt,
          ),
        ).length,
        8,
      );
      assert.equal(
        nativeErrors(await wait(second, (packet) => packet.version === 1, resumedAt)).length,
        8,
      );
`,
  ],
  [
    `      admittedGate.assertStopped();
      admittedGate.resume();
      await wait(
        first,
        (packet) => packet.version === 3 && nativeErrors(packet).length === 8,
        admittedPass,
      );
      await wait(
        second,
        (packet) => packet.version === 2 && nativeErrors(packet).length === 8,
        admittedPass,
      );
`,
    `      admittedGate.assertStopped();
      const admittedResumedAt = notifications.length;
      admittedGate.resume();
      await wait(
        first,
        (packet) => packet.version === 3 && nativeErrors(packet).length === 8,
        admittedResumedAt,
      );
      await wait(
        second,
        (packet) => packet.version === 2 && nativeErrors(packet).length === 8,
        admittedResumedAt,
      );
`,
  ],
] as const;

function replaceOnce(source: string, before: string, after: string): string {
  const at = source.indexOf(before);
  assert(
    at >= 0 && at === source.lastIndexOf(before),
    "only the reviewed source transition is accepted",
  );
  return source.slice(0, at) + after + source.slice(at + before.length);
}

/** Preserve the old law and admit only the two stronger post-resume cursors. */
export function currentWarmupWitness(archive: Buffer): Buffer {
  assert.equal(sha256(archive), HISTORICAL_WARMUP_SHA);
  let source = archive.toString("utf8");
  for (const [before, after] of RESUME_CHANGES) source = replaceOnce(source, before, after);
  assert.equal(
    sha256(Buffer.from(source)),
    MAIN_WARMUP_SHA,
    "the reviewed main successor must remain exact",
  );
  source = replaceOnce(
    source,
    'import { readPinnedArtifact, sha256 } from "../differential/harness.mjs";',
    'import { readPinnedArtifact } from "../differential/harness.mjs";\nimport { assertWarmupWitness } from "./support/lsp/native-warmup-witness.ts";',
  );
  source = replaceOnce(
    source,
    ORIGINAL_CHECK,
    "const warmupWitnessCustody = assertWarmupWitness(root, fixture.provenance.warmupWitness);",
  );
  source = replaceOnce(
    source,
    "            warmupWitness: fixture.provenance.warmupWitness,",
    "            warmupWitness: fixture.provenance.warmupWitness,\n            warmupWitnessSuccessor: warmupWitnessCustody,",
  );
  return Buffer.from(source);
}

export function warmupWitnessCustody(current: Buffer) {
  return {
    schema: "vize.lsp.native-warmup-witness-successor",
    version: 1,
    historical: {
      path: WARMUP_WITNESS_PATH,
      sha256: HISTORICAL_WARMUP_SHA,
      sourceRevision: "c4918d796969adae9a73a9ecbf382e0f79becf56",
    },
    archive: { path: "historical-witness.ts.txt", sha256: HISTORICAL_WARMUP_SHA },
    mainSuccessor: {
      sourceRevision: "68a68d44f5fe38f7aa09b3ff8d3a0e532328d9a1",
      sha256: MAIN_WARMUP_SHA,
      change: "two exact post-resume notification cursors; no diagnostic expectation change",
    },
    currentConsumer: {
      path: WARMUP_WITNESS_PATH,
      sha256: sha256(current),
      meaning: "current test executor custody only; no backend or stdio qualification",
    },
  };
}

export function validateWarmupWitness(
  witness: unknown,
  archive: Buffer,
  current: Buffer,
  custody: unknown,
) {
  assert.deepEqual(witness, { path: WARMUP_WITNESS_PATH, sha256: HISTORICAL_WARMUP_SHA });
  const expected = currentWarmupWitness(archive);
  assert(current.equals(expected), "unreviewed current warm-up consumer source");
  const authenticated = warmupWitnessCustody(expected);
  assert.deepEqual(
    custody,
    authenticated,
    "only the closed current-consumer provenance is accepted",
  );
  return authenticated;
}

export function assertWarmupWitness(root: string, witness: unknown) {
  const directory = path.join(root, DIRECTORY);
  return validateWarmupWitness(
    witness,
    fs.readFileSync(path.join(directory, "historical-witness.ts.txt")),
    fs.readFileSync(path.join(root, WARMUP_WITNESS_PATH)),
    JSON.parse(fs.readFileSync(path.join(directory, "custody.json"), "utf8")),
  );
}
