/** Reconstruct the predecessor of the two generated TypeScript edits in #13430. */
import assert from "node:assert/strict";
import type { Packet } from "./constructor-projection-parity.ts";
import { point, type Edit } from "./projection-byte-relocation.ts";

const suffix =
  " & ({ [key: string]: $ComponentInstance } & { input: unknown })[unknown extends $AuthoredProps ? unknown extends $JsxProps ? string : 'input' : 'input']";
const namedExport =
  /^declare const ([$_\p{ID_Start}](?:[$\p{ID_Continue}]|\u200c|\u200d)*): typeof __vize_component__;\n(export default \1)( as typeof \1);$/mu;

export function hasMetadataExport(text: string): boolean {
  return namedExport.test(text);
}

/** Producer-owned ignore gaps are authenticated before the caller passes the packet. */
export function beforeMetadataPacket(packet: Packet): Packet {
  const match = namedExport.exec(packet.text);
  if (!match) return packet;
  const bytes = Buffer.from(packet.text);
  const byte = (at: number) => Buffer.byteLength(packet.text.slice(0, at));
  const declaration = byte(match.index);
  const assignment = declaration + Buffer.byteLength(match[0].split("\n")[0]) + 1;
  const assignmentLength = Buffer.byteLength(match[2] + match[3] + ";");
  const edits: Edit[] = [];
  const constructor = packet.text.indexOf("\ntype $ComponentConstructor = new <$AuthoredProps");
  if (constructor >= 0) {
    const tail = ", $AuthoredProps, $JsxProps>>";
    const ending = tail + suffix + ";\n";
    const position = packet.text.indexOf(ending, constructor);
    assert.ok(
      position > constructor && position < match.index,
      "expected the metadata instance in the constructor return",
    );
    assert.equal(packet.text.indexOf(ending, position + 1), -1);
    const start = byte(position + tail.length);
    edits.push({ start, end: start + Buffer.byteLength(suffix), replacement: Buffer.alloc(0) });
  }
  const prefixEnd = assignment + Buffer.byteLength(match[2]);
  edits.push({
    start: prefixEnd,
    end: prefixEnd + Buffer.byteLength(match[3]),
    replacement: Buffer.alloc(0),
  });

  const previous = structuredClone(packet);
  for (const edit of [...edits].reverse()) {
    const text = Buffer.from(previous.text);
    previous.text = Buffer.concat([
      text.subarray(0, edit.start),
      text.subarray(edit.end),
    ]).toString();
  }
  const precedingRange = (start: number, length: number) => {
    const from = point(start, edits);
    return [from, point(start + length, edits) - from];
  };
  const exportRows = packet.mappings.filter(([start]) => start === assignment);
  assert.deepEqual(
    exportRows,
    [[assignment, assignmentLength, 0, 0, 1, 248]],
    "default export navigation",
  );
  previous.mappings = packet.mappings.map(([start, length, ...rest]) =>
    start === assignment
      ? [
          point(declaration, edits) + Buffer.byteLength("declare const "),
          Buffer.byteLength(match[1]),
          ...rest,
        ]
      : [...precedingRange(start, length), ...rest],
  );
  for (const link of previous.semanticLinks ?? []) {
    [link.sourceStart, link.sourceLength] = precedingRange(link.sourceStart, link.sourceLength);
    [link.targetStart, link.targetLength] = precedingRange(link.targetStart, link.targetLength);
  }
  for (const row of previous.diagnosticDirectives?.directives ?? []) {
    row[2] = point(row[2], edits);
    row[3] = point(row[3], edits);
  }
  assert.ok(
    bytes
      .subarray(assignment, assignment + assignmentLength)
      .equals(Buffer.from(match[2] + match[3] + ";")),
  );
  return previous;
}
