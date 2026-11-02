import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { slotCliPacket } from "../../tools/benchmarks/scripts/canon-slot-provider-cli.ts";

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
  const currentPreamble = historical.replace(oldAlias, currentAlias);
  assert.notEqual(currentPreamble, historical);
  const stderr = Buffer.from(
    `\n=== __vize_helpers.d.ts ===\n${currentPreamble}\n\n=== ${path.join(project, "src/Tiny.vue")} ===\n${virtualTs}\n`,
  );
  assert.deepEqual(slotCliPacket({ stdout, stderr, status: 0 }, project).diagnostics, []);
  for (const altered of [
    Buffer.from(stderr.toString("utf8").replace(currentPreamble, historical)),
    Buffer.concat([Buffer.from("unknown warning\n"), stderr]),
    Buffer.concat([stderr, Buffer.from("unknown error\n")]),
    stderr.subarray(1),
    Buffer.concat([stderr, stderr]),
    Buffer.from(stderr.toString("utf8").replace(virtualTs, "export const value = 2;\n")),
  ])
    assert.throws(() => slotCliPacket({ stdout, stderr: altered, status: 0 }, project));
});
