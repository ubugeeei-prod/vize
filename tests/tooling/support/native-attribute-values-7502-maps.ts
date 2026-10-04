import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { exactKeys7502 } from "./native-attribute-values-7502-inputs.ts";

export function coordinate7502(text: string, offset: number) {
  assert(Number.isSafeInteger(offset) && offset >= 0 && offset <= Buffer.byteLength(text));
  const bytes = Buffer.from(text).subarray(0, offset);
  const prefix = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  assert(Buffer.from(prefix).equals(bytes), "authentic UTF-8 map endpoint");
  const lines = prefix.split(/\r\n|[\r\n\u2028\u2029]/);
  return [lines.length - 1, lines.at(-1)!.length];
}
export function codec7502() {
  const ui = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
  const vue = createRequire(ui.resolve("vue/package.json"));
  assert.equal(vue("@vue/compiler-sfc/package.json").version, "3.5.35");
  const sfc = createRequire(vue.resolve("@vue/compiler-sfc/package.json"));
  return createRequire(sfc.resolve("magic-string/package.json"))("@jridgewell/sourcemap-codec");
}
export function maps7502(packet: any, codec: any) {
  for (const row of packet.rows) {
    if (row.disposition !== "positive" || !row.sourceMap) continue;
    const { code, map, links } = row.result;
    assert.equal(map.file, row.filename);
    assert.deepEqual(map.sources, [row.filename]);
    assert.deepEqual(map.sourcesContent, [row.nativeSource]);
    const names: string[] = [];
    const expected = [];
    for (const link of links) {
      exactKeys7502(link, ["authored", "generated", "name", "segment"]);
      assert.equal(typeof link.segment, "boolean");
      assert(link.name === null || typeof link.name === "string");
      for (const [text, range] of [
        [row.nativeSource, link.authored],
        [code, link.generated],
      ] as const) {
        exactKeys7502(range, ["start", "end"]);
        assert(range.start <= range.end);
        coordinate7502(text, range.start);
        coordinate7502(text, range.end);
      }
      if (!link.segment) continue;
      const [line, column] = coordinate7502(code, link.generated.start);
      const [sourceLine, sourceColumn] = coordinate7502(row.nativeSource, link.authored.start);
      const anchor = [line, column, 0, sourceLine, sourceColumn];
      if (link.name !== null) {
        if (!names.includes(link.name)) names.push(link.name);
        anchor.push(names.indexOf(link.name));
      }
      expected.push(anchor);
    }
    expected.sort((left, right) => left[0] - right[0] || left[1] - right[1]);
    assert.deepEqual(map.names, names);
    const decoded = codec.decode(map.mappings);
    assert.equal(codec.encode(decoded), map.mappings, "complete canonical mappings retained");
    const generated = code.split(/\r\n|[\r\n\u2028\u2029]/);
    const original = row.nativeSource.split(/\r\n|[\r\n\u2028\u2029]/);
    const actual: number[][] = [];
    decoded.forEach((segments: number[][], line: number) => {
      let previous = -1;
      for (const segment of segments) {
        assert([4, 5].includes(segment.length) && segment.every(Number.isSafeInteger));
        const [column, source, sourceLine, sourceColumn, name] = segment;
        assert.equal(source, 0);
        assert(
          generated[line] !== undefined &&
            column >= previous &&
            column >= 0 &&
            column <= generated[line].length,
        );
        assert(
          original[sourceLine] !== undefined &&
            sourceColumn >= 0 &&
            sourceColumn <= original[sourceLine].length,
        );
        if (segment.length === 5) assert(name >= 0 && name < names.length);
        previous = column;
        actual.push([line, ...segment]);
      }
    });
    assert.deepEqual(
      actual,
      expected,
      "every original segment-bearing link has its complete exact start anchor",
    );
  }
}
