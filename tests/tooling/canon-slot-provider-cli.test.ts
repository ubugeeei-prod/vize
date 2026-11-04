import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { slotCliPacket } from "../../tools/benchmarks/scripts/canon-slot-provider-cli.ts";
import {
  AUTHENTICATED_PREAMBLE_BYTES,
  AUTHENTICATED_PREAMBLE_SHA256,
  applySourceOwnedPreambleTransitions,
  applyCurrentEmitPreamble,
} from "../../tools/benchmarks/scripts/canon-slot-provider-preamble.ts";

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
    historical.replace(oldAlias, currentAlias),
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
  assert.equal(applyCurrentEmitPreamble(previousPreamble), currentPreamble);
  assert.throws(() => applyCurrentEmitPreamble(currentPreamble), /certified previous preamble/u);
  assert.throws(
    () => applyCurrentEmitPreamble(previousPreamble + "\n"),
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
                : fs.readFileSync(file, "utf8") + "\n"
              : fs.readFileSync(file, "utf8"),
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
  const stderr = Buffer.from(
    `\n=== __vize_helpers.d.ts ===\n${currentPreamble}\n\n=== ${path.join(project, "src/Tiny.vue")} ===\n${virtualTs}\n`,
  );
  assert.deepEqual(slotCliPacket({ stdout, stderr, status: 0 }, project).diagnostics, []);
  for (const altered of [
    Buffer.from(stderr.toString("utf8").replace(currentPreamble, historical)),
    Buffer.from(stderr.toString("utf8").replace(currentPreamble, previousPreamble)),
    Buffer.concat([Buffer.from("unknown warning\n"), stderr]),
    Buffer.concat([stderr, Buffer.from("unknown error\n")]),
    stderr.subarray(1),
    Buffer.concat([stderr, stderr]),
    Buffer.from(stderr.toString("utf8").replace(virtualTs, "export const value = 2;\n")),
  ])
    assert.throws(() => slotCliPacket({ stdout, stderr: altered, status: 0 }, project));
});
