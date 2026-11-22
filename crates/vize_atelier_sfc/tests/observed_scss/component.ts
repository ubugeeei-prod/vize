import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const require = createRequire(import.meta.url);
const { decode, encode } = require("@jridgewell/sourcemap-codec");
const { parse } = require("@vue/compiler-sfc");
const sass = require("sass");
const frozen = JSON.parse(
  fs.readFileSync(
    new URL("../fixtures/native_sfc_ts_annotation_setup_vue_3_5_35.json", import.meta.url),
    "utf8",
  ),
);
const preprocess = fileURLToPath(
  new URL("../../../vize/src/commands/build/runner/preprocess.mjs", import.meta.url),
);

type Point = [number, number];
type Segment = [number, number, number, number, number?];
type Scene = {
  name: string;
  filename: string;
  source: string;
  trim: boolean;
  code: string;
  map: object;
  css: string | null;
  cssMap: object | null;
};
const rows: Scene[] = JSON.parse(fs.readFileSync(0, "utf8"));

function starts(text: string, breaks = /\r\n|[\r\n\u2028\u2029]/g): number[] {
  return [0, ...Array.from(text.matchAll(breaks), (m) => m.index + m[0].length)];
}
function point(lines: number[], offset: number): Point {
  const line = lines.findLastIndex((start) => start <= offset);
  assert.ok(line >= 0);
  return [line, offset - lines[line]];
}

const jobs: { file: string; lang: string; source: string }[] = [];
const expectedBlocks: { css: string }[] = [];
let anchors = 0;
let modules = 0;
for (const row of rows) {
  const { descriptor, errors } = parse(row.source, {
    filename: row.filename,
    sourceMap: false,
    ignoreEmpty: false,
  });
  assert.deepEqual(errors, [], row.name);
  const inputLines = starts(row.source);
  let css = "";
  const links: Point[] = [];
  for (const block of descriptor.styles) {
    if (css.length) css += "\n";
    const start = block.loc.start.offset;
    assert.equal(row.source.slice(start, block.loc.end.offset), block.content);
    if (block.lang?.toLowerCase() === "scss") {
      const file = resolve(row.name);
      const result = sass.compileString(block.content, {
        url: pathToFileURL(file),
        sourceMap: true,
      });
      jobs.push({ file, lang: "scss", source: block.content });
      expectedBlocks.push({ css: result.css });
      const inLines = starts(block.content, /\r\n|[\r\n]/g);
      const outLines = starts(result.css);
      const map: Segment[][] = decode(result.sourceMap.mappings);
      for (const [line, segments] of map.entries()) {
        for (const [col, sourceIndex, sourceLine, sourceCol] of segments) {
          assert.equal(sourceIndex, 0);
          links.push([css.length + outLines[line] + col, start + inLines[sourceLine] + sourceCol]);
        }
      }
      css += result.css;
    } else {
      assert.ok(!block.scoped && !block.module && !block.src, row.name);
      const raw: string = block.content;
      const body = row.trim ? raw.trim() : raw;
      const offset = row.trim ? raw.length - raw.trimStart().length : 0;
      for (let line = 0; line < body.length;) {
        links.push([css.length + line, start + offset + line]);
        const lf = body.indexOf("\n", line);
        if (lf < 0) break;
        line = lf + 1;
      }
      css += body;
    }
  }
  const generated = starts(css);
  const mapped: Segment[][] = [];
  for (const [target, source] of links) {
    const [line, col] = point(generated, target);
    const [srcLine, srcCol] = point(inputLines, source);
    while (mapped.length <= line) mapped.push([]);
    mapped[line].push([col, 0, srcLine, srcCol]);
  }
  anchors += links.length;
  assert.equal(row.css, css || null, row.name);
  assert.deepEqual(
    row.cssMap,
    css
      ? {
          version: 3,
          file: row.filename,
          sources: [row.filename],
          sourcesContent: [row.source],
          names: [],
          mappings: encode(mapped),
        }
      : null,
    row.name,
  );
  for (const fixture of frozen.fixtures) {
    const prefix = row.source.indexOf(fixture.source);
    if (prefix < 0) continue;
    const fixtureLines = starts(fixture.source);
    const map: Segment[][] = decode(fixture.nativeMap.mappings);
    for (const segments of map) {
      for (const segment of segments) {
        const offset = prefix + fixtureLines[segment[2]] + segment[3];
        [segment[2], segment[3]] = point(inputLines, offset);
      }
    }
    assert.equal(row.code, fixture.code, row.name);
    assert.deepEqual(
      row.map,
      { ...fixture.nativeMap, sourcesContent: [row.source], mappings: encode(map) },
      row.name,
    );
    modules++;
  }
}
const stock = spawnSync(process.execPath, [preprocess], {
  input: JSON.stringify(jobs),
  encoding: "utf8",
  maxBuffer: 64 << 20,
});
assert.equal(stock.status, 0, stock.stderr);
assert.deepEqual(JSON.parse(stock.stdout), expectedBlocks);
assert.ok(anchors && modules && jobs.length);
console.log(
  `whole SCSS styles=${jobs.length} CSS anchors=${anchors} frozen JS modules and maps=${modules} Sass=${sass.info.split("\n")[0]}`,
);
