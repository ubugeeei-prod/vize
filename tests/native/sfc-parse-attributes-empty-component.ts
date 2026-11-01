import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

// This is the existing empty input's build-only successor. The original source,
// before packet and all parse modes remain unchanged; no SFC syntax is inferred.
const referenceBytes = readFileSync(
  new URL("../fixtures/sfc/parsed-attributes/empty-compile-successor.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(referenceBytes).digest("hex"),
  "98ae006b791e68b764b46780022bf132681822024b475be6c9ce799c12b74e55",
  "the complete authentic before/after packet record must remain unchanged",
);
export const emptyCompileReference = JSON.parse(referenceBytes.toString("utf8")) as {
  schema: string;
  fixSource: string;
  evidence: Record<string, unknown>;
  source: string;
  before: Record<string, unknown>;
  after: Record<string, unknown>;
};

export const emptyCompileSourceBlobs = {
  "crates/vize_atelier_sfc/src/compile.rs": "73dac21158fb72588268f617d9b439554f064fa1",
  "crates/vize_atelier_sfc/src/compile/empty_component.rs":
    "33ee8b8b1f6e8085cccfb609c1781ccecc6877b9",
  "crates/vize_vitrine/src/napi/sfc/compile.rs": "e8a1b34ac5268c71573347506dfdeb6d6fc9ccb0",
} as const;

export function validateEmptyCompileSources(sources: Record<string, Uint8Array>): void {
  assert.deepEqual(Object.keys(sources).sort(), Object.keys(emptyCompileSourceBlobs).sort());
  for (const [file, expected] of Object.entries(emptyCompileSourceBlobs)) {
    const bytes = sources[file];
    const blob = createHash("sha1")
      .update(Buffer.from(`blob ${bytes.length}\0`))
      .update(bytes)
      .digest("hex");
    assert.equal(blob, expected, `${file}: the complete reviewed build/error law must be retained`);
  }
}

validateEmptyCompileSources(
  Object.fromEntries(
    Object.keys(emptyCompileSourceBlobs).map((file) => [
      file,
      readFileSync(new URL(`../../${file}`, import.meta.url)),
    ]),
  ),
);
assert.equal(emptyCompileReference.schema, "vize.sfc-attributes-empty-compile-reference.v1");
assert.equal(emptyCompileReference.fixSource, "2645cb590a3f839494a8681a26cb305eaa34a9e9");
assert.equal(emptyCompileReference.source, "");

export function emptyCompilePacketReference(
  packet: Record<string, unknown>,
): Record<string, unknown> {
  // Reject unexpected baseline fields as well as unknown candidate differences.
  // The complete after packet comes from the retained actual failed comparison,
  // independently justified by the three authenticated production source files.
  assert.deepEqual(packet, emptyCompileReference.before);
  return structuredClone(emptyCompileReference.after);
}
