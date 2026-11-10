import assert from "node:assert/strict";
import { test } from "node:test";
import {
  comments,
  point,
  range,
  rangeIncludingInsertions,
  type Edit,
} from "../../tools/support/compat/content-mapper/projection-byte-relocation.ts";
import { withoutProducerUnmappedIgnores } from "../../tools/support/compat/content-mapper/producer-unmapped-ignores.ts";
import { dedentEdits } from "../../tools/support/compat/content-mapper/setup-dedent.ts";
import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

// The canon train (#12572) changes every packet by three projection edits:
// verbatim setup lines (#12550), the NoInfer constructor return (#12523) and
// the shared template ref unwrap (#12470).
const CONSTRUCTOR = [
  [
    "...args: any[]) => $ComponentInstanceFor<$ComponentInstance, ",
    "...args: any[]) => ($IsAny<$AuthoredProps> extends true ? true : $IsUnknown<$AuthoredProps> extends true ? $UsePublicInstance<$JsxProps> : false) extends true ? $ComponentInstance : NoInfer<$ComponentInstanceFor<$ComponentInstance, ",
  ],
  [", $AuthoredProps, $JsxProps>;", ", $AuthoredProps, $JsxProps>>;"],
] as const;
const UNWRAP_DECLARATIONS = [
  `    type __U<T> = T extends { value: unknown } ? T extends import('vue').Ref ? import("vize/virtual-ts").$WidenTemplateRef<T['value']> : T : T;\n`,
  `    type __UN<T> = [T] extends [import('vue').Ref] ? __U<T> : [Extract<T, import('vue').Ref>] extends [never] ? __U<T> : (NonNullable<T> & import('vue').Ref<unknown> & { value: __U<T> })['value'];\n`,
];
const UNWRAP_REFERENCES = [
  ["__UN<__R_", 'import("vize/virtual-ts").$TemplateRefUnwrapNarrowed<__R_'],
  ["__U<__R_", 'import("vize/virtual-ts").$TemplateRefUnwrap<__R_'],
] as const;

type Result = Transform & {
  semanticLinks?: {
    sourceStart: number;
    sourceLength: number;
    targetStart: number;
    targetLength: number;
  }[];
};
type Labeled = Edit & { label: string };

const replace = (start: number, before: string, after: string, label: string): Labeled => ({
  start,
  end: start + Buffer.byteLength(before),
  replacement: Buffer.from(after),
  label,
});

function constructorEdits(text: Buffer): Labeled[] {
  const [[head, nextHead], [tail, nextTail]] = CONSTRUCTOR;
  const edits: Labeled[] = [];
  for (let at = text.indexOf(head); at >= 0; at = text.indexOf(head, at + 1)) {
    const end = text.indexOf(tail, at);
    assert.ok(end > at, "every constructor has its instance tail");
    edits.push(replace(at, head, nextHead, "NoInfer constructor"));
    edits.push(replace(end, tail, nextTail, "NoInfer constructor tail"));
  }
  assert.equal(edits.length, 2, "every packet holds one component constructor");
  return edits;
}

function unwrapEdits(text: Buffer): Labeled[] {
  const edits: Labeled[] = [];
  for (const declaration of UNWRAP_DECLARATIONS) {
    const at = text.indexOf(declaration);
    if (at >= 0) edits.push(replace(at, declaration, "", "per-file unwrap alias"));
  }
  for (const [from, to] of UNWRAP_REFERENCES)
    for (let at = text.indexOf(from); at >= 0; at = text.indexOf(from, at + 1))
      if (!edits.some((edit) => at < edit.end && at + Buffer.byteLength(from) > edit.start))
        edits.push(replace(at, from, to, "shared unwrap reference"));
  assert.ok(
    edits.length === 0 || edits.some((edit) => edit.label === "per-file unwrap alias"),
    "a shared reference replaces only a per-file alias",
  );
  return edits;
}

/** Apply the certified edits in order, rewriting each map comment they relocate. */
function relocate(
  before: Buffer,
  after: Buffer,
  certified: Labeled[],
  ranges: [number, number][],
  next: [number, number][],
) {
  const original = comments(before.toString(), ranges);
  const actual = comments(after.toString(), next);
  assert.equal(actual.length, original.length, "generated map-comment inventory changed");
  const events = [
    ...certified.map((edit) => ({ start: edit.start, edit, index: -1 })),
    ...original.map((comment, index) => ({ start: comment.start, edit: null, index })),
  ].sort((left, right) => left.start - right.start);
  const edits: Edit[] = [];
  for (const event of events) {
    if (event.edit) {
      assert.ok((edits.at(-1)?.end ?? 0) <= event.edit.start, "certified edits are disjoint");
      edits.push(event.edit);
      continue;
    }
    const comment = original[event.index]!;
    const [g0, g1, s0, s1] = comment.values as [number, number, number, number];
    assert.ok(g1 <= comment.start, "a map comment describes preceding code");
    const values = [point(g0, edits, true), point(g1, edits, true), s0, s1];
    assert.deepEqual(actual[event.index]!.values, values, "self-described coordinates relocate");
    const text = `${comment.indent}// @vize-map: ${values[0]}:${values[1]} -> ${s0}:${s1}${comment.text.endsWith("\r") ? "\r" : ""}`;
    edits.push({
      start: comment.start,
      end: comment.start + Buffer.byteLength(comment.text),
      replacement: Buffer.from(text),
    });
  }
  const parts: Buffer[] = [];
  let cursor = 0;
  for (const edit of edits) {
    parts.push(before.subarray(cursor, edit.start), edit.replacement);
    cursor = edit.end;
  }
  parts.push(before.subarray(cursor));
  assert.deepEqual(Buffer.concat(parts), after, "all other generated bytes are exact");
  return edits;
}

test("the canon train population changes each packet only by its three projection edits", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const name = "source-packets-current-b6b4bbb0a16a.json";
  const previousName = "source-packets-current-cd10508733d6.json";
  assert.equal(authority.currentSourcePackets, name);
  const golden = selectPacketGoldens(authority, files, cases)!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: previousName },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.pullRequest, 12572);
  assert.equal(provenance.producer.sourceRevision, "b6b4bbb0a16ab65e29ef7202b488215d8ea4b9a9");
  assert.deepEqual(provenance.previousPopulation, {
    path: previousName,
    bytes: Buffer.byteLength(files.get(previousName)!),
    sha256: sha(files.get(previousName)!),
  });
  assert.equal(golden.packets.length, 80);
  const unwrapped: string[] = [];
  for (const [index, packet] of golden.packets.entries()) {
    const old = previous.packets[index]!;
    assert.equal(packet.label, old.label);
    assert.equal(packet.sourceSha256, old.sourceSha256);
    assert.equal(packet.globalsSha256, old.globalsSha256);
    const base = withoutProducerUnmappedIgnores(old.result as Result);
    const head = withoutProducerUnmappedIgnores(packet.result as Result, { requireComplete: true });
    const before = Buffer.from(base.text),
      after = Buffer.from(head.text);
    const unwrap = unwrapEdits(before);
    if (unwrap.length > 0) unwrapped.push(packet.label);
    const certified = [...dedentEdits(before, after), ...constructorEdits(before), ...unwrap];
    assert.ok(
      certified.some((edit) => edit.label === "verbatim setup line"),
      packet.label,
    );
    const spans = (result: Result) =>
      result.mappings.map(([g, length]) => [g!, length!] as [number, number]);
    const edits = relocate(before, after, certified, spans(base), spans(head));
    const expected = structuredClone(base);
    expected.text = head.text;
    expected.mappings = base.mappings.map(([g, length, ...rest]) => [
      ...range(g!, length!, edits, before.length),
      ...rest,
    ]);
    if (expected.semanticLinks)
      expected.semanticLinks = expected.semanticLinks.map((link) => {
        const [sourceStart, sourceLength] = range(
          link.sourceStart,
          link.sourceLength,
          edits,
          before.length,
        );
        const [targetStart, targetLength] = range(
          link.targetStart,
          link.targetLength,
          edits,
          before.length,
        );
        return { ...link, sourceStart, sourceLength, targetStart, targetLength };
      });
    if (expected.diagnosticDirectives)
      expected.diagnosticDirectives.directives = expected.diagnosticDirectives.directives.map(
        (tuple) => {
          const [g, length] = rangeIncludingInsertions(
            tuple[2]!,
            tuple[3]! - tuple[2]!,
            edits,
            before.length,
          );
          return [tuple[0]!, tuple[1]!, g, g + length, ...tuple.slice(4)];
        },
      );
    assert.deepEqual(
      head,
      expected,
      `${packet.label}: only the certified edits move any byte or coordinate`,
    );
  }
  assert.deepEqual(unwrapped, [
    "lf/slot-shadow",
    "lf/mixed-slot-scope",
    "crlf-unicode/slot-shadow",
    "crlf-unicode/mixed-slot-scope",
  ]);
});
