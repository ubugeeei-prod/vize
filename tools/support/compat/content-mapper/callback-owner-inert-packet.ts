/** The event-navigation callback copies are inert; mapped owners stay exact. */
import assert from "node:assert/strict";

type Span = { start: number; end: number; text: string };
export type InertCallbackBinding = { name: Span; value: Span };

/** Derive the complete new rows from independently authored binding geometry. */
export function withInertCallbackRows(
  before: Record<string, unknown>,
  after: Record<string, unknown>,
  source: string,
  bindings: InertCallbackBinding[],
): Record<string, unknown> {
  assert.equal(before.text, after.text, "this certificate changes no generated text");
  assert.equal(typeof before.text, "string");
  assert.ok(Array.isArray(before.mappings));
  const text = before.text as string;
  const generated = Buffer.from(text);
  const authored = Buffer.from(source);
  const byteSpan = (span: Span) => {
    assert.ok(Number.isSafeInteger(span.start) && Number.isSafeInteger(span.end));
    assert.ok(0 <= span.start && span.start < span.end && span.end <= source.length);
    assert.equal(source.slice(span.start, span.end), span.text);
    return [Buffer.byteLength(source.slice(0, span.start)), Buffer.byteLength(span.text)];
  };
  const copies: { name: string; value: string; key: number; valueAt: number }[] = [];
  const blocks =
    /^([ \t]*)const __vize_events_resolved_(\d+) = \(undefined as unknown as __vize_events_resolver_\2\)\(\{\n([\s\S]*?)^\1\}\);\n/gmu;
  const marker =
    "// @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.";
  for (const block of text.matchAll(blocks)) {
    const prefix = `${block[1]}  `;
    const bodyAt = block.index + block[0].indexOf("\n") + 1;
    const lines = block[3].split("\n");
    let offset = bodyAt;
    for (let index = 0; index < lines.length; index++) {
      const line = lines[index];
      if (line === prefix + marker) {
        const entry = lines[index + 1];
        assert.ok(entry?.startsWith(prefix + '"'));
        const match = /^"([A-Za-z_$][A-Za-z0-9_$]*)": (.+),$/u.exec(entry.slice(prefix.length));
        assert.ok(match, "only a complete unchanged single-line callback entry is admitted");
        const key = Buffer.byteLength(text.slice(0, offset + line.length + 1 + prefix.length));
        copies.push({
          name: match[1],
          value: match[2],
          key,
          valueAt: key + Buffer.byteLength(`"${match[1]}": `),
        });
      }
      offset += line.length + 1;
    }
  }
  assert.equal(
    copies.length,
    bindings.length,
    "every authored callback has exactly one navigation copy",
  );
  assert.ok(copies.length > 0);
  const rows: number[][] = [];
  bindings.forEach((binding, index) => {
    const copy = copies[index];
    assert.equal(copy.name, binding.name.text);
    assert.equal(copy.value, binding.value.text);
    assert.ok(binding.value.text.includes("=>") && !/[\r\n]/u.test(binding.value.text));
    assert.equal(
      source.slice(binding.name.start - 1, binding.value.start),
      `:${binding.name.text}="`,
    );
    assert.equal(source[binding.value.end], '"');
    const [nameAt, nameLength] = byteSpan(binding.name);
    const [valueAt, valueLength] = byteSpan(binding.value);
    assert.deepEqual(authored.subarray(nameAt, nameAt + nameLength), Buffer.from(copy.name));
    assert.deepEqual(generated.subarray(copy.key, copy.valueAt - 2), Buffer.from(`"${copy.name}"`));
    assert.deepEqual(
      generated.subarray(copy.valueAt, copy.valueAt + valueLength),
      Buffer.from(copy.value),
    );
    rows.push([nameAt, nameLength, copy.key, copy.valueAt - 2, 0, 0]);
    rows.push([valueAt, valueLength, copy.valueAt, copy.valueAt + valueLength, 0, 0]);
  });
  for (const [, , start, end] of rows) {
    for (const tuple of before.mappings as number[][]) {
      assert.equal(tuple.length, 6);
      assert.ok(tuple.every(Number.isSafeInteger));
      assert.ok(
        tuple[0] + tuple[1] <= start || end <= tuple[0],
        "inert rows cannot hide any mapped owner",
      );
    }
  }
  const expected = structuredClone(before);
  const old = before.diagnosticDirectives as
    | { directives: number[][]; unusedExpectDirectiveDiagnostics: unknown[] }
    | undefined;
  if (old) {
    assert.ok(Array.isArray(old.directives));
    for (const row of rows)
      assert.ok(
        !old.directives.some((prior) => prior[2] === row[2] && prior[3] === row[3]),
        "an old row cannot be replaced or duplicated",
      );
  }
  expected.diagnosticDirectives = {
    ...(old ?? { unusedExpectDirectiveDiagnostics: [] }),
    directives: [...(old?.directives ?? []), ...rows].sort(
      (left, right) => left[2] - right[2] || left[3] - right[3],
    ),
  };
  return expected;
}
