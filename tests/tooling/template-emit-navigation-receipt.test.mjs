import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyInverseEdits,
  historicalInterval,
  historicalOffset,
  loadRetainedProjectionFixture,
  qualifyRetainedProjectionHistory,
  sha256,
} from "../../tools/benchmarks/scripts/typechecker-template-emit-projection-history.ts";

function cloneFixture(fixture) {
  return {
    manifest: structuredClone(fixture.manifest),
    packets: Object.fromEntries(
      Object.entries(fixture.packets).map(([kind, files]) => [
        kind,
        new Map([...files].map(([path, bytes]) => [path, Buffer.from(bytes)])),
      ]),
    ),
  };
}

void test("retained successful and failed artifacts preserve the frozen projection proof", () => {
  const proof = qualifyRetainedProjectionHistory();
  assert.equal(proof.packetFiles, 194);
  assert.equal(proof.editProofs.length, 6);
  assert.equal(
    proof.editProofs.reduce((total, row) => total + row.edits, 0),
    144,
  );
  assert.equal(
    proof.coordinateProof.sha256,
    "612f87e159a42c96c895a440365a8fc6c312b096a37c6f926fb1b2e5f8e0a91e",
  );
  assert.deepEqual(
    proof.coordinateProof.surfaces.find(
      ({ path }) => path === "define-model/content-mapper/semantic-links.txt",
    ),
    {
      path: "define-model/content-mapper/semantic-links.txt",
      baselineRows: 2,
      incidentRows: 2,
      matchedRows: 1,
      addedRows: 0,
      rejectedRows: 1,
      missingRows: 1,
      matchedSha256: "14ca26fbe37ca11cdb2c80cebc14d30a517138e0fb559cf7540715c32837231f",
      addedSha256: "37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570",
      rejectedSha256: "6d9da8152cf59a01a11a6d345223f280eb43e489e2f2426bc538d1819bdbdf89",
      missingSha256: "a3cc67fdb7c691cf5b692051c883666ba2397b721e8f4fee2ff5b6806c82a5ac",
    },
  );
  assert.deepEqual(
    proof.frozenBaseline.map(({ name, initializerBytesAdded }) => [name, initializerBytesAdded]),
    [
      ["dual-scripts-emits", 50],
      ["define-model", 97],
    ],
  );
});

void test("the complete incident packet produces a module-custodied schema-2 receipt", () => {
  const fixture = loadRetainedProjectionFixture();
  const root = mkdtempSync(join(tmpdir(), "vize-projection-receipt-"));
  try {
    for (const [path, bytes] of fixture.packets.incident) {
      const target = join(root, path);
      mkdirSync(dirname(target), { recursive: true });
      writeFileSync(target, bytes);
    }
    const verifier = fileURLToPath(
      new URL(
        "../../tools/benchmarks/scripts/typechecker-template-emit-projection-receipt.mjs",
        import.meta.url,
      ),
    );
    const result = spawnSync(process.execPath, [verifier], {
      env: {
        ...process.env,
        SOURCE_SHA: fixture.manifest.artifacts.incident.sourceSha,
        VIZE_TEMPLATE_EMIT_TS40_CAPTURE: root,
      },
      encoding: "utf8",
    });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
    const receipt = JSON.parse(readFileSync(join(root, "receipts.json"), "utf8"));
    assert.equal(receipt.schema, 2);
    assert.deepEqual(
      receipt.historyVerifierModules.map(({ path }) => path),
      [
        "typechecker-template-emit-projection-history-baseline.ts",
        "typechecker-template-emit-projection-history-coordinates.ts",
        "typechecker-template-emit-projection-history-fixture.ts",
        "typechecker-template-emit-projection-history-live.ts",
        "typechecker-template-emit-projection-history-model.ts",
        "typechecker-template-emit-projection-history-shared.ts",
        "typechecker-template-emit-projection-history.ts",
      ],
    );
    for (const module of receipt.historyVerifierModules) {
      const url = new URL(`../../tools/benchmarks/scripts/${module.path}`, import.meta.url);
      assert.equal(module.sha256, sha256(readFileSync(url)));
    }
    assert.equal(
      receipt.historyVerifierSha256,
      sha256(Buffer.from(`${JSON.stringify(receipt.historyVerifierModules)}\n`)),
    );
    assert.equal(receipt.completeRawInputsAndOutputs.length, 194);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

void test("mutated generated helpers cannot reuse the retained edit scripts", () => {
  const fixture = cloneFixture(loadRetainedProjectionFixture());
  const path = "define-model/canon/text.ts";
  const bytes = fixture.packets.incident.get(path);
  const sentinel = Buffer.from("const $emit = (void 0)");
  const index = bytes.indexOf(sentinel);
  assert(index >= 0);
  bytes[index + sentinel.length - 2] = "1".charCodeAt(0);
  assert.throws(
    () => qualifyRetainedProjectionHistory(fixture),
    /Expected values to be strictly equal|deep-equal/u,
  );
});

void test("mutated generated coordinates cannot hide behind exact packet custody", () => {
  const fixture = cloneFixture(loadRetainedProjectionFixture());
  const path = "define-model/content-mapper/mappings.txt";
  const rows = fixture.packets.incident.get(path).toString("utf8").split("\n");
  const first = JSON.parse(rows[0]);
  first[0] += 1;
  rows[0] = `[${first.join(", ")}]`;
  fixture.packets.incident.set(path, Buffer.from(rows.join("\n")));
  assert.throws(
    () => qualifyRetainedProjectionHistory(fixture),
    /Expected values to be strictly deep-equal/u,
  );
});

void test("helper-internal and helper-crossing coordinates fail closed", () => {
  const fixture = loadRetainedProjectionFixture();
  const script = fixture.manifest.editScripts.find(({ id }) => id === "define-model/canon");
  assert(script);
  const replacement = script.edits.find((edit) => edit.afterEnd - edit.afterStart > 2);
  assert(replacement);
  assert.throws(
    () => historicalOffset(replacement.afterStart + 1, script.edits),
    /entered a changed helper range/u,
  );
  assert.throws(
    () => historicalInterval(replacement.afterStart - 1, 2, script.edits),
    /overlaps a changed helper range/u,
  );
  for (const value of [-1, 0.5, "offset", Number.MAX_SAFE_INTEGER + 1])
    assert.throws(() => historicalOffset(value, script.edits), /invalid generated offset/u);
  const before = fixture.packets.baseline.get(script.path);
  const after = fixture.packets.incident.get(script.path);
  assert.deepEqual(applyInverseEdits(after, before, script.edits), before);
});
