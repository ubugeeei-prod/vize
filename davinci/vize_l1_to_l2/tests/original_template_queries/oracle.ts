import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);
const { parse } = require("@vue/compiler-sfc");
assert.equal(require("@vue/compiler-sfc/package.json").version, "3.5.35");
const [paths, output] = process.argv.slice(2);
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const rows = [];
for (const filename of fs.readFileSync(paths, "utf8").split("\0").filter(Boolean)) {
  const bytes = fs.readFileSync(path.resolve(filename));
  let source;
  try {
    source = decoder.decode(bytes);
  } catch {
    continue;
  }
  const { descriptor, errors } = parse(source, { filename, sourceMap: false });
  const block = descriptor.template;
  let span = null;
  if (block) {
    const start = block.loc.start.offset;
    const end = block.loc.end.offset;
    if (!block.src && (!block.lang || block.lang === "html")) {
      assert.equal(source.slice(start, end), block.content, filename);
    }
    span = [Buffer.byteLength(source.slice(0, start)), Buffer.byteLength(source.slice(0, end))];
  }
  rows.push([filename, span, errors.length]);
}
fs.writeFileSync(output, JSON.stringify(rows));
