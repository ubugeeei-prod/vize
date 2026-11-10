import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  applyCanonLeftoverPreamble,
  canonLeftoverPreamblePaths,
  slotCliPacket,
} from "../../tools/benchmarks/scripts/canon-slot-provider-cli.ts";
import {
  AUTHENTICATED_PREAMBLE_BYTES,
  AUTHENTICATED_PREAMBLE_SHA256,
  applyCurrentOverloadPreamble,
  applyIntegratedSourcePreambleTransitions,
  applyPreviousOverloadPreamble,
  applySourceOwnedPreambleTransitions,
  applyCurrentEmitPreamble,
} from "../../tools/benchmarks/scripts/canon-slot-provider-preamble.ts";
import {
  applyCurrentSlotPreamble,
  readPreviousEmitSource,
} from "../../tools/benchmarks/scripts/canon-slot-provider-current-preamble.ts";
import {
  beforeGenericEventPreamble,
  genericEventRecordPath,
} from "./support/generic-event-specialization-source.ts";
import {
  applyDiscriminatedJsxUnionIn,
  beforeDiscriminatedJsxUnionIn,
} from "../../tools/support/compat/content-mapper/discriminated-jsx-union-transition.ts";
import {
  beforeCanonLeftoversIn,
  applyStrictExtraKeysIn,
  beforeStrictExtraKeysIn,
} from "../../tools/support/compat/content-mapper/strict-extra-keys-transition.ts";
import {
  applyEmitMemberInferIn,
  beforeEmitMemberInferIn,
} from "../../tools/support/compat/content-mapper/emit-member-infer-transition.ts";
import {
  applyInstanceInputKeysIn,
  beforeInstanceInputKeysIn,
} from "../../tools/support/compat/content-mapper/instance-input-keys-transition.ts";
import {
  applyUnownedOwnedKeysIn,
  beforeUnownedOwnedKeysIn,
} from "../../tools/support/compat/content-mapper/unowned-owned-keys-transition.ts";
import {
  applyPropValueOrIn,
  beforePropValueOrIn,
} from "../../tools/support/compat/content-mapper/prop-value-or-transition.ts";
import { applyListenerContractIn } from "../../tools/support/compat/content-mapper/listener-contract-transition.ts";
import {
  applyUsageContractIn,
  beforeUsageContractIn,
} from "../../tools/support/compat/content-mapper/usage-contract-transition.ts";

test("quiet CLI debug output retains exact helper/file bytes and refuses unrelated stderr", () => {
  const record = JSON.parse(
    fs.readFileSync(
      "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/slot-provider-payload-update-11027.json",
      "utf8",
    ),
  ) as { logicalPreamble: { afterWholeUtf8: string } };
  const project = path.resolve("tests/slot-cli-format-control"),
    virtualTs = "export const value = 1;\n";
  const stdout = Buffer.from(
    JSON.stringify({
      errorCount: 0,
      files: [{ file: "src/Tiny.vue", virtualTs, diagnostics: [] }],
    }),
  );
  const historical = record.logicalPreamble.afterWholeUtf8;
  const oldAlias = historical
    .split("\n")
    .find((line) => line.startsWith("type $OverloadUnionRecursive<"));
  const currentAlias = fs
    .readFileSync(
      "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt",
      "utf8",
    )
    .split("\n")[1];
  assert.ok(oldAlias !== undefined && currentAlias !== undefined);
  const previousPreamble = applySourceOwnedPreambleTransitions(
    applyPreviousOverloadPreamble(historical),
  );
  assert.notEqual(previousPreamble, historical);
  assert.equal(Buffer.byteLength(previousPreamble), AUTHENTICATED_PREAMBLE_BYTES);
  assert.equal(
    createHash("sha256").update(previousPreamble).digest("hex"),
    AUTHENTICATED_PREAMBLE_SHA256,
  );
  const certificate = JSON.parse(
    fs.readFileSync(
      "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/canonical-loose-emit-sdk-update-11815.json",
      "utf8",
    ),
  ) as {
    preamble: {
      edits: { before: string; after: string }[];
      after: { bytes: number; sha256: string };
    };
  };
  assert.equal(certificate.preamble.edits.length, 1);
  const edit = certificate.preamble.edits[0]!;
  assert.equal(previousPreamble.split(edit.before).length, 2);
  const currentPreamble = previousPreamble.replace(edit.before, edit.after);
  assert.equal(Buffer.byteLength(currentPreamble), certificate.preamble.after.bytes);
  assert.equal(
    createHash("sha256").update(currentPreamble).digest("hex"),
    certificate.preamble.after.sha256,
  );
  assert.equal(applyCurrentEmitPreamble(previousPreamble, readPreviousEmitSource), currentPreamble);
  assert.throws(
    () => applyCurrentEmitPreamble(currentPreamble, readPreviousEmitSource),
    /certified previous preamble/u,
  );
  assert.throws(
    () => applyCurrentEmitPreamble(previousPreamble + "\n", readPreviousEmitSource),
    /certified previous preamble/u,
  );
  for (const changed of [
    "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/canonical-loose-emit-sdk-update-11815.json",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt",
  ]) {
    for (const mutation of ["missing", "changed"] as const) {
      assert.throws(
        () =>
          applyCurrentEmitPreamble(previousPreamble, (file) =>
            file === changed
              ? mutation === "missing"
                ? ""
                : readPreviousEmitSource(file) + "\n"
              : readPreviousEmitSource(file),
          ),
        /immutable emit preamble certificate|selected canonical emit source/u,
      );
    }
  }
  assert.ok(currentPreamble.includes("declare var __vizeTemplateChecker: true;"));
  assert.ok(
    currentPreamble.includes(
      "type __RuntimePropValue<T> = T extends (...args: any[]) => infer V ? V : T extends abstract new (...args: any[]) => infer V ? V : never;",
    ),
  );
  const successorPreamble = applyCurrentSlotPreamble(previousPreamble);
  assert.equal(
    beforeGenericEventPreamble((file) => fs.readFileSync(file, "utf8"), successorPreamble),
    currentPreamble,
  );
  const overloadPreamble = applyCurrentOverloadPreamble(successorPreamble);
  const integratedSourcePreamble = applyIntegratedSourcePreambleTransitions(overloadPreamble);
  for (const changed of [
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-listener-args.d.ts.txt",
  ]) {
    for (const mutation of ["missing", "changed"] as const) {
      assert.throws(
        () =>
          applyIntegratedSourcePreambleTransitions(overloadPreamble, (file) =>
            file === changed
              ? mutation === "missing"
                ? ""
                : fs.readFileSync(file, "utf8") + "\n"
              : fs.readFileSync(file, "utf8"),
          ),
        /authenticated preamble asset/u,
      );
    }
  }
  const discriminatedSourcePreamble = applyDiscriminatedJsxUnionIn(integratedSourcePreamble);
  const strictSourcePreamble = applyStrictExtraKeysIn(discriminatedSourcePreamble);
  const emitMemberSourcePreamble = applyEmitMemberInferIn(strictSourcePreamble);
  const canonSourcePreamble = applyCanonLeftoverPreamble(emitMemberSourcePreamble);
  const keysSourcePreamble = applyInstanceInputKeysIn(canonSourcePreamble);
  assert.equal(beforeInstanceInputKeysIn(keysSourcePreamble), canonSourcePreamble);
  const ownedSourcePreamble = applyUnownedOwnedKeysIn(keysSourcePreamble);
  assert.equal(beforeUnownedOwnedKeysIn(ownedSourcePreamble), keysSourcePreamble);
  const propValueOrSourcePreamble = applyPropValueOrIn(ownedSourcePreamble);
  assert.equal(beforePropValueOrIn(propValueOrSourcePreamble), ownedSourcePreamble);
  assert.throws(() => applyPropValueOrIn(propValueOrSourcePreamble), /PropValueOr/u);
  const listenerSourcePreamble = applyListenerContractIn(propValueOrSourcePreamble);
  const currentSourcePreamble = applyUsageContractIn(listenerSourcePreamble);
  assert.equal(beforeUsageContractIn(currentSourcePreamble), listenerSourcePreamble);
  assert.throws(() => applyUsageContractIn(currentSourcePreamble), /usage contract/u);
  const canonPredecessor = [...canonLeftoverPreamblePaths]
    .reverse()
    .reduce((current, file) => beforeCanonLeftoversIn(file, current), canonSourcePreamble);
  assert.equal(canonPredecessor, emitMemberSourcePreamble);
  assert.equal(beforeEmitMemberInferIn(canonPredecessor), strictSourcePreamble);
  assert.equal(beforeStrictExtraKeysIn(strictSourcePreamble), discriminatedSourcePreamble);
  assert.equal(
    beforeDiscriminatedJsxUnionIn(discriminatedSourcePreamble),
    integratedSourcePreamble,
  );
  assert.equal(currentSourcePreamble.split(currentAlias).length, 2);
  assert.equal(currentSourcePreamble.includes("declare var __vizeTemplateChecker: true;"), false);
  assert.ok(currentSourcePreamble.includes("type $ComponentInstanceFor<"));
  for (const changed of [
    genericEventRecordPath,
    "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-inference.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-inference.d.ts.txt",
  ]) {
    for (const mutation of ["missing", "changed"] as const) {
      assert.throws(() =>
        applyCurrentSlotPreamble(previousPreamble, (file) =>
          file === changed
            ? mutation === "missing"
              ? ""
              : fs.readFileSync(file, "utf8") + "\n"
            : fs.readFileSync(file, "utf8"),
        ),
      );
    }
  }
  const stderr = Buffer.from(
    `\n=== __vize_helpers.d.ts ===\n${currentSourcePreamble}\n\n=== ${path.join(project, "src/Tiny.vue")} ===\n${virtualTs}\n`,
  );
  assert.deepEqual(slotCliPacket({ stdout, stderr, status: 0 }, project).diagnostics, []);
  for (const altered of [
    Buffer.from(stderr.toString("utf8").replace(currentSourcePreamble, currentPreamble)),
    Buffer.from(stderr.toString("utf8").replace(currentSourcePreamble, successorPreamble)),
    Buffer.from(stderr.toString("utf8").replace(currentSourcePreamble, historical)),
    Buffer.from(stderr.toString("utf8").replace(currentSourcePreamble, previousPreamble)),
    Buffer.concat([Buffer.from("unknown warning\n"), stderr]),
    Buffer.concat([stderr, Buffer.from("unknown error\n")]),
    stderr.subarray(1),
    Buffer.concat([stderr, stderr]),
    Buffer.from(stderr.toString("utf8").replace(virtualTs, "export const value = 2;\n")),
  ])
    assert.throws(() => slotCliPacket({ stdout, stderr: altered, status: 0 }, project));
});
