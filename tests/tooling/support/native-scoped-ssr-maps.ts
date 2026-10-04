import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { fromVue } from "./native-scoped-ssr-reference.ts";

const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
export const codec = createRequire(fromSfc.resolve("magic-string/package.json"))(
  "@jridgewell/sourcemap-codec",
);

export function coordinate(text: string, offset: number) {
  const bytes = Buffer.from(text).subarray(0, offset);
  const prefix = bytes.toString("utf8");
  assert(Buffer.from(prefix).equals(bytes), "map endpoints must retain UTF-8 boundaries");
  const lines = prefix.split(/\r\n|[\r\n\u2028\u2029]/);
  return [lines.length - 1, lines.at(-1)!.length];
}
// Counterfactual judge input only: equal bytes outside the real style body,
// with coherent source coordinates, must not gain original-style custody.
export function outsideStyleWindow(
  actual: any,
  originalCss: string,
  span: { start: number; end: number },
) {
  const forged = structuredClone(actual);
  const outside = Buffer.byteLength(forged.source + "<!--");
  forged.source += `<!--${originalCss}-->`;
  forged.cssMap.sourcesContent = [forged.source];
  for (const link of forged.cssLinks) {
    link.authored.start += outside - span.start;
    link.authored.end += outside - span.start;
  }
  const decoded = codec.decode(forged.cssMap.mappings);
  for (const link of forged.cssLinks.filter((link: any) => link.segment)) {
    const [line, column] = coordinate(forged.css, link.generated.start);
    const anchor = decoded[line].find((segment: number[]) => segment[0] === column);
    assert(anchor);
    const [sourceLine, sourceColumn] = coordinate(forged.source, link.authored.start);
    anchor[2] = sourceLine;
    anchor[3] = sourceColumn;
  }
  forged.cssMap.mappings = codec.encode(decoded);
  return forged;
}
export function checkMap(source: string, filename: string, text: string, map: any, links: any[]) {
  assert.equal(map.version, 3);
  assert.equal(map.file, filename);
  assert.deepEqual(map.sources, [filename]);
  assert.deepEqual(map.sourcesContent, [source]);
  const names: string[] = [];
  const expected = links
    .filter((link) => link.segment)
    .map((link) => {
      const [line, column] = coordinate(text, link.generated.start);
      const [sourceLine, sourceColumn] = coordinate(source, link.authored.start);
      const anchor = [line, column, 0, sourceLine, sourceColumn];
      if (link.name !== null) {
        if (!names.includes(link.name)) names.push(link.name);
        anchor.push(names.indexOf(link.name));
      }
      return anchor;
    })
    .sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  assert.deepEqual(map.names, names);
  const decoded = codec.decode(map.mappings);
  assert.equal(codec.encode(decoded), map.mappings);
  const actual: number[][] = [];
  const generatedLines = text.split(/\r\n|[\r\n\u2028\u2029]/);
  const originalLines = source.split(/\r\n|[\r\n\u2028\u2029]/);
  decoded.forEach((line: number[][], index: number) => {
    let previous = -1;
    for (const segment of line) {
      assert([4, 5].includes(segment.length));
      assert(segment.every(Number.isSafeInteger));
      const [column, sourceIndex, sourceLine, sourceColumn, name] = segment;
      assert.equal(sourceIndex, 0);
      assert(
        generatedLines[index] !== undefined &&
          column >= previous &&
          column <= generatedLines[index].length,
      );
      assert(
        originalLines[sourceLine] !== undefined &&
          sourceColumn >= 0 &&
          sourceColumn <= originalLines[sourceLine].length,
      );
      if (segment.length === 5) assert(name >= 0 && name < names.length);
      previous = column;
      actual.push([index, ...segment]);
    }
  });
  assert.deepEqual(
    actual,
    expected,
    "every original segment contributes its complete UTF-16 start anchor",
  );
  for (const link of links) {
    assert.equal(typeof link.segment, "boolean");
    for (const [content, span] of [
      [source, link.authored],
      [text, link.generated],
    ] as const) {
      assert(Number.isSafeInteger(span.start) && Number.isSafeInteger(span.end));
      assert(span.start >= 0 && span.start <= span.end && span.end <= Buffer.byteLength(content));
      coordinate(content, span.start);
      coordinate(content, span.end);
    }
  }
}
export function checkCssBytes(
  actual: any,
  originalCss: string,
  span: { start: number; end: number },
) {
  assert.equal(
    Buffer.from(actual.source).subarray(span.start, span.end).toString("utf8"),
    originalCss,
  );
  let generated = 0;
  let original = span.start;
  let authored = "";
  const gaps = [];
  for (const link of actual.cssLinks) {
    assert.equal(
      link.authored.start,
      original,
      "CSS links must cover the exact original style window in contiguous order",
    );
    assert(
      link.authored.end >= original && link.authored.end <= span.end,
      "CSS link must stay inside the exact original style window",
    );
    if (generated < link.generated.start)
      gaps.push(Buffer.from(actual.css).subarray(generated, link.generated.start).toString("utf8"));
    const copied = Buffer.from(actual.css)
      .subarray(link.generated.start, link.generated.end)
      .toString("utf8");
    assert.equal(
      Buffer.from(actual.source).subarray(link.authored.start, link.authored.end).toString("utf8"),
      copied,
    );
    authored += copied;
    generated = link.generated.end;
    original = link.authored.end;
  }
  assert.equal(generated, Buffer.byteLength(actual.css));
  assert.equal(authored, originalCss);
  assert.equal(original, span.end, "CSS links must exhaust the exact original style window");
  assert.deepEqual(gaps, [`[${actual.scopeId}]`]);
}
