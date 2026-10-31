/** --quiet removes progress; --show-virtual-ts still emits exact source dumps. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { hash } from "./canon-instance-directives-io.ts";
import { packet, isRecord } from "./canon-instance-directives-model.ts";
import { refFactoryCurrentPreamble } from "./ref-factory-authority.ts";

export function slotCliPacket(
  process: { stdout: Buffer; stderr: Buffer; status: number | null },
  project: string,
) {
  const observed = packet(process.stdout, process.status, project);
  const bytes = fs.readFileSync(
    path.resolve(
      "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/slot-provider-payload-update-11027.json",
    ),
  );
  assert.equal(hash(bytes), "0a71c5c21535e50d02df527e6c00267dfd8b44bfc0d63c37c49a55d67ab1166c");
  const source: unknown = JSON.parse(bytes.toString("utf8"));
  assert.ok(isRecord(source) && isRecord(source.logicalPreamble));
  const helper = source.logicalPreamble;
  assert.equal(typeof helper.afterWholeUtf8, "string");
  const text = helper.afterWholeUtf8 as string;
  assert.equal(Buffer.byteLength(text), helper.afterBytes);
  assert.equal(hash(text), helper.afterSha256);
  const files = observed.result.files.filter((file) => typeof file.virtualTs === "string");
  assert.ok(files.length > 0, "all actual virtual file bodies must remain in JSON");
  const expected =
    `\n=== __vize_helpers.d.ts ===\n${refFactoryCurrentPreamble(text)}\n` +
    files
      .map((file) => `\n=== ${path.resolve(project, file.file)} ===\n${file.virtualTs}\n`)
      .join("");
  assert.equal(
    process.stderr.equals(Buffer.from(expected)),
    true,
    "complete CLI stderr must equal its source-backed helper and JSON virtual-file dumps",
  );
  return observed;
}
