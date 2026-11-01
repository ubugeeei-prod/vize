import assert from "node:assert/strict";
import type { TimingField } from "./model.ts";
const escaped = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
/** Whole-output template. Only five existing CLI renderer timing slots can vary;
 * diagnostics, paths, positions, counts, settings and all other bytes are literal. */
export function assertRenderedTimings(
  raw: Buffer,
  literals: string[],
  fields: TimingField[],
): void {
  assert(fields.length >= 1 && fields.length <= 5);
  assert.deepEqual(
    fields.map((f) => f.label),
    ["total", "collect", "imports", "gen", "corsa"],
    "only the complete ordered CLI summary timing slots may vary",
  );
  assert.equal(literals.length, fields.length + 1);
  assert.equal(new Set(fields.map((v) => v.label)).size, fields.length);
  let pattern = "^";
  assert(
    literals[1] === " (collect: " &&
      literals[2] === ", imports: " &&
      literals[3] === ", gen: " &&
      literals[4] === ", corsa: ",
  );
  assert(
    literals[5]?.startsWith(")\n"),
    "timing summary must close at its actual renderer placement",
  );
  for (let index = 0; index < fields.length; index++) {
    const field = fields[index]!,
      literal = literals[index]!;
    if (field.label === "total") assert(/(?:^|\n)Type checked [1-9]\d* files in $/u.test(literal));
    else
      assert(
        literal.endsWith(field.label === "collect" ? "(collect: " : `, ${field.label}: `),
        "timing cannot replace an arbitrary diagnostic field",
      );
    assert(
      Number.isFinite(field.maxNanoseconds) &&
        field.maxNanoseconds >= 0 &&
        field.maxNanoseconds <= 600_000_000_000,
    );
    pattern += escaped(literal) + "((?:0|[1-9]\\d*)(?:\\.\\d{1,6})?)(ns|µs|μs|ms|s)";
  }
  pattern += escaped(literals.at(-1)!) + "$";
  const value = new TextDecoder("utf-8", { fatal: true }).decode(raw),
    matched = new RegExp(pattern, "u").exec(value);
  assert(matched && matched[0] === value, "whole rendered CLI vector differs");
  for (let index = 0; index < fields.length; index++) {
    const amount = Number(matched[1 + index * 2]),
      unit = matched[2 + index * 2]!;
    const multiplier = (
      { ns: 1, µs: 1_000, μs: 1_000, ms: 1_000_000, s: 1_000_000_000 } as Record<string, number>
    )[unit];
    assert(multiplier);
    assert(amount * multiplier <= fields[index]!.maxNanoseconds);
  }
}
