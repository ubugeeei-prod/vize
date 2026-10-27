/** Bind labels to independently built source artifacts before publishing a ratio. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import type { binaryEvidence } from "./pr-performance-observation-evidence.ts";

type Artifact = ReturnType<typeof binaryEvidence>;
export function verifyObservationBindings(
  directory: string | null,
  sources: { base: string; head: string },
  binaries: { base: Artifact; head: Artifact },
) {
  try {
    assert.ok(directory, "source build receipts not supplied");
    const receipts = Object.fromEntries(
      (["base", "head"] as const).map((side) => [
        side,
        JSON.parse(readFileSync(join(directory, `${side}.json`), "utf8")),
      ]),
    );
    for (const side of ["base", "head"] as const) {
      const receipt = receipts[side],
        binary = binaries[side];
      assert.equal(receipt.schemaVersion, 1);
      assert.equal(receipt.role, side, `${side}: wrong build role`);
      assert.equal(receipt.sourceSha, sources[side], `${side}: wrong source identity`);
      assert.match(receipt.treeSha, /^[a-f0-9]{40}$/u);
      assert.equal(resolve(receipt.binary), resolve(binary.path), `${side}: wrong binary path`);
      assert.equal(receipt.binarySha256, binary.sha256, `${side}: wrong binary hash`);
      assert.equal(receipt.binaryBytes, binary.bytes, `${side}: wrong binary bytes`);
      assert.equal(receipt.targetStartedEmpty, true, `${side}: target did not start empty`);
      assert.equal(receipt.cargoArtifact?.fresh, false, `${side}: executable was reused`);
      assert.equal(
        resolve(receipt.cargoArtifact.executable),
        join(resolve(receipt.targetDir), "ci-opt", "vize"),
      );
      assert.ok(Array.isArray(receipt.cargoArtifact.features));
      assert.ok(
        !receipt.cargoArtifact.features.includes("profiling"),
        "profiling binary cannot rank normal latency",
      );
      assert.equal(
        receipt.allocation,
        false,
        "allocation instrumentation cannot rank normal latency",
      );
      assert.equal(receipt.profile?.name, "ci-opt");
      assert.equal(typeof receipt.rustc, "string");
      assert.ok(Array.isArray(receipt.cargo?.args));
      assert.equal(typeof receipt.cargo.version, "string");
      assert.ok(receipt.compilerEnvironment && typeof receipt.compilerEnvironment === "object");
    }
    assert.notEqual(
      resolve(receipts.base.targetDir),
      resolve(receipts.head.targetDir),
      "shared Cargo target",
    );
    assert.equal(receipts.base.rustc, receipts.head.rustc, "different Rust compilers");
    assert.deepEqual(receipts.base.cargo?.args, receipts.head.cargo?.args, "different Cargo flags");
    assert.equal(receipts.base.cargo?.version, receipts.head.cargo?.version);
    assert.deepEqual(receipts.base.compilerEnvironment, receipts.head.compilerEnvironment);
    return { qualified: true, reason: null };
  } catch (error) {
    return { qualified: false, reason: String(error) };
  }
}
