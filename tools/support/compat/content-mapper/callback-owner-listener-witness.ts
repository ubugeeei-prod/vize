import assert from "node:assert/strict";

export type ListenerWitnessEdit = {
  readonly start: number;
  readonly end: number;
  readonly replacement: Buffer;
};

type WitnessLine = {
  readonly end: number;
  readonly indent: string;
  readonly key: string;
  readonly keyValue: string;
  readonly newlinePrefix: string;
  readonly start: number;
};

const witnessLinePattern = /^([ \t]*)\.\.\.\{\} as \{ ("(?:[^"\\]|\\.)*"): never \},(\r?)$/gmu;

function witnessLines(text: string): WitnessLine[] {
  return [...text.matchAll(witnessLinePattern)].map((match) => {
    assert.ok(match.index != null);
    const keyValue: unknown = JSON.parse(match[2]);
    assert.ok(typeof keyValue === "string", "listener witness keys are string literals");
    const lineEnd = match.index + match[0].length;
    assert.equal(text[lineEnd], "\n", "listener witness lines retain their newline");
    return {
      start: match.index,
      end: lineEnd + 1,
      indent: match[1],
      key: match[2],
      keyValue,
      newlinePrefix: match[3],
    };
  });
}

/**
 * Describe the exact per-key-to-per-usage listener witness transition.
 *
 * The candidate packet is still checked byte-for-byte by the caller. This
 * helper only admits complete consecutive legacy witness lines and derives
 * the one replacement line the current emitter must produce. Repeated keys
 * keep their final position, matching `append_listener_witnesses`.
 */
export function mergedListenerWitnessEdits(text: string): ListenerWitnessEdit[] {
  const lines = witnessLines(text);
  const edits: ListenerWitnessEdit[] = [];
  for (let index = 0; index < lines.length;) {
    const first = lines[index];
    const block = [first];
    index++;
    while (
      index < lines.length &&
      lines[index].start === block.at(-1)!.end &&
      lines[index].indent === first.indent &&
      lines[index].newlinePrefix === first.newlinePrefix
    ) {
      block.push(lines[index]);
      index++;
    }

    const keys: WitnessLine[] = [];
    for (const line of block) {
      const previous = keys.findIndex((seen) => seen.keyValue === line.keyValue);
      if (previous >= 0) keys.splice(previous, 1);
      keys.push(line);
    }
    const properties = keys.map(({ key }) => `${key}: never`).join("; ");
    const replacement = `${first.indent}...null! as { ${properties} },${first.newlinePrefix}\n`;
    edits.push({
      start: Buffer.byteLength(text.slice(0, first.start)),
      end: Buffer.byteLength(text.slice(0, block.at(-1)!.end)),
      replacement: Buffer.from(replacement),
    });
  }
  return edits;
}
