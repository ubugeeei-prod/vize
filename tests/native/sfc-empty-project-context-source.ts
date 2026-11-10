import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

interface SourceChange {
  path: string;
  referenceBlob: string;
  currentBlob: string;
  replacements: { reference: string; current: string }[];
}

const bytes = readFileSync(
  new URL(
    "../fixtures/sfc/parsed-attributes/empty-compile-project-context-source.json",
    import.meta.url,
  ),
);
assert.equal(
  createHash("sha256").update(bytes).digest("hex"),
  "56fc4444a4074f0852134b83b2a33032a2bd0c12719dc99aeb3b2e6eeb4cb485",
  "the reviewed project-context transformation must remain complete",
);
const plan = JSON.parse(bytes.toString("utf8")) as {
  schema: string;
  sources: SourceChange[];
  supportingSources: Record<string, string>;
};
assert.equal(plan.schema, "vize.sfc-empty-compile-project-context-source-successor.v1");

export const emptyProjectContextSupportingBlobs = plan.supportingSources;

function blob(bytes: Uint8Array): string {
  return createHash("sha1")
    .update(Buffer.from(`blob ${bytes.length}\0`))
    .update(bytes)
    .digest("hex");
}

function supportingSourceForReference(file: string, bytes: Uint8Array): Uint8Array {
  if (
    file !== "crates/vize_atelier_sfc/src/compile/entry/root_match.rs" ||
    blob(bytes) !== "ebff50833e6b9b5fb2998bc7adf8a3874160eaf3"
  )
    return bytes;
  let text = Buffer.from(bytes).toString("utf8");
  for (const [current, reference] of [
    [
      "    Allocator, ParseMode, ParserOptions, PropNode, TemplateChildNode,",
      "    Allocator, ParserOptions, PropNode, TemplateChildNode,",
    ],
    ["        ParserOptions {\n            mode: ParseMode::Sfc,\n", "        ParserOptions {\n"],
  ]) {
    assert.equal(text.split(current!).length, 2, "unique reviewed SFC grammar edit");
    text = text.replace(current!, reference!);
  }
  const reference = Buffer.from(text);
  assert.equal(blob(reference), "7c678c8a0b4a29f173ad67ad298afd1cb92ede8b");
  return reference;
}

/** Authenticate only this reviewed integration, then retain the original law. */
export function emptyCompileSourcesForReference(
  sources: Record<string, Uint8Array>,
  referenceBlobs: Readonly<Record<string, string>>,
  supportingSources?: Record<string, Uint8Array>,
): Record<string, Uint8Array> {
  assert.deepEqual(Object.keys(sources).sort(), Object.keys(referenceBlobs).sort());
  // The original complete bundle still uses its original validation directly.
  // In particular it does not require any new project-context source files.
  if (Object.entries(referenceBlobs).every(([file, expected]) => blob(sources[file]) === expected))
    return sources;

  assert.deepEqual(
    plan.sources.map((source) => source.path).sort(),
    Object.keys(referenceBlobs).sort(),
  );
  const supporting =
    supportingSources ??
    Object.fromEntries(
      Object.keys(plan.supportingSources).map((file) => [
        file,
        readFileSync(new URL(`../../${file}`, import.meta.url)),
      ]),
    );
  assert.deepEqual(Object.keys(supporting).sort(), Object.keys(plan.supportingSources).sort());
  for (const [file, expected] of Object.entries(plan.supportingSources))
    assert.equal(
      blob(supportingSourceForReference(file, supporting[file])),
      expected,
      `reviewed project-context dependency: ${file}`,
    );

  const restored: Record<string, Uint8Array> = {};
  for (const source of plan.sources) {
    assert.equal(source.referenceBlob, referenceBlobs[source.path]);
    const current = sources[source.path];
    assert.equal(blob(current), source.currentBlob, `reviewed integration: ${source.path}`);
    let text = Buffer.from(current).toString("utf8");
    for (const replacement of [...source.replacements].reverse()) {
      assert.notEqual(replacement.current, "");
      const start = text.indexOf(replacement.current);
      assert(start >= 0, `missing reviewed integration: ${source.path}`);
      assert.equal(start, text.lastIndexOf(replacement.current), "ambiguous reviewed integration");
      text = text.replace(replacement.current, replacement.reference);
    }
    restored[source.path] = Buffer.from(text);
    assert.equal(blob(restored[source.path]), source.referenceBlob, "original law reconstruction");
  }
  return restored;
}
