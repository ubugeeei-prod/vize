/** Authenticate the current cohorts and persist the complete packet comparison receipt. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { readAssets } from "./constructor-helper-assets.ts";
import { certifyCurrentProducerPopulation } from "./producer-unmapped-ignores.ts";
import type { comparePackets as ComparePackets, Row } from "./constructor-projection-parity.ts";
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

export function writeConstructorProjectionReceipt(
  comparePackets: typeof ComparePackets,
  comparatorPath: string,
) {
  const [directory, baseRoot, headRoot] = process.argv.slice(2);
  assert.ok(directory && baseRoot && headRoot && process.argv.length === 5);
  const baseBytes = readFileSync(path.join(directory, "base.json"));
  const headBytes = readFileSync(path.join(directory, "head.json"));
  const legacyBaseBytes = readFileSync(path.join(directory, "base-legacy.json"));
  const legacyHeadBytes = readFileSync(path.join(directory, "head-legacy.json"));
  const base = readFileSync(path.join(directory, "base-revision.txt"), "utf8").trim();
  assert.ok(process.env.HEAD_SHA, "selected head source SHA is required");
  const assets = readAssets(baseRoot, headRoot, {
    repository: headRoot,
    base,
    head: process.env.HEAD_SHA,
  });
  const certifyCurrent = (bytes: Buffer) => {
    const rows: Row[] = JSON.parse(bytes.toString());
    const count = certifyCurrentProducerPopulation(
      rows.filter((row) => row.target === "content-mapper").map((row) => row.packet),
    );
    assert.equal(count, 192, "all current mapper transforms must be certified");
    return count;
  };
  const currentProducerPopulation = {
    current: certifyCurrent(headBytes),
    legacy: certifyCurrent(legacyHeadBytes),
  };
  const legacyProof = comparePackets(
    JSON.parse(legacyBaseBytes.toString()),
    JSON.parse(legacyHeadBytes.toString()),
    assets,
  );
  const proof = comparePackets(
    JSON.parse(baseBytes.toString()),
    JSON.parse(headBytes.toString()),
    assets,
  );
  const receipt = {
    currentProducerPopulation,
    emitOverloads: assets.emitOverloads?.provenance,
    pullRequestBase: process.env.BASE_SHA,
    base: readFileSync(path.join(directory, "base-revision.txt"), "utf8").trim(),
    head: process.env.HEAD_SHA,
    observerSha256: digest(
      readFileSync(
        path.join(headRoot, "crates/vize_canon/tests/constructor_emission_projection.rs"),
      ),
    ),
    comparatorSha256: digest(readFileSync(comparatorPath)),
    comparatorModules: Object.fromEntries(
      [
        "constructor-projection-parity.ts",
        "constructor-helper-assets.ts",
        "constructor-projection-receipt.ts",
        "projection-byte-relocation.ts",
        "owned-emit-overload-transition.ts",
        "component-constructor-transition.ts",
        "component-export-transition.ts",
        "canonical-helper-transition.ts",
        "producer-unmapped-ignores.ts",
      ].map((file) => [file, digest(readFileSync(path.join(path.dirname(comparatorPath), file)))]),
    ),
    basePacketSha256: digest(baseBytes),
    headPacketSha256: digest(headBytes),
    legacyBasePacketSha256: digest(legacyBaseBytes),
    legacyHeadPacketSha256: digest(legacyHeadBytes),
    legacy: legacyProof,
    ...proof,
    scope:
      "Complete source packets under exact helper relocation; not native type-engine or publication acceptance",
  };
  writeFileSync(path.join(directory, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
  console.log(JSON.stringify(receipt));
}
