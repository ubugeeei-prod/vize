import assert from "node:assert/strict";
import { test } from "node:test";
import {
  range,
  rangeIncludingInsertions,
  type Edit,
} from "../../tools/support/compat/content-mapper/projection-byte-relocation.ts";
import { withoutProducerUnmappedIgnores } from "../../tools/support/compat/content-mapper/producer-unmapped-ignores.ts";
import { verbatimSetupRelocation } from "../../tools/support/compat/content-mapper/verbatim-setup-relocation.ts";
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
  diagnosticDirectives?: { directives: number[][]; unusedExpectDirectiveDiagnostics: unknown[] };
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

test("the canon train population changes each packet only by its three projection edits", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const name = "source-packets-current-b6b4bbb0a16a.json";
  const previousName = "source-packets-current-cd10508733d6.json";
  assert.equal(authority.currentSourcePackets, "source-packets-current-4faf0850f21f.json");
  const golden = selectPacketGoldens({ ...authority, currentSourcePackets: name }, files, cases)!;
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
    const spans = (result: Result) =>
      result.mappings.map(([g, length]) => [g!, length!] as [number, number]);
    const edits = verbatimSetupRelocation(
      before,
      after,
      [...constructorEdits(before), ...unwrap],
      spans(base),
      spans(head),
    );
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
