import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadCompilerManifest } from "../differential/compiler.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const fixture = new URL(
  "../fixtures/css/compile-source-maps/v-bind-public-neighbors.json",
  import.meta.url,
);
const raw = fs.readFileSync(fixture),
  frozen = JSON.parse(raw);
const family = loadCompilerManifest(
  path.join(root, "tests/_fixtures/differential/compiler/manifest.json"),
).cssApiCases.find((row) => row.id === "compiler/css/v-bind-source-map-10712");
assert.ok(family);
const native = createRequire(import.meta.url)("../../npm/native/index.js");
const directory = path.join(root, "target/css-compile-source-maps");
fs.mkdirSync(directory, { recursive: true });
const packet = {
  source: null,
  fixtureSha256: createHash("sha256").update(raw).digest("hex"),
  calls: [],
  failures: [],
};
const retain = () =>
  fs.writeFileSync(
    path.join(directory, "v-bind-observation.json"),
    JSON.stringify(packet, null, 2) + "\n",
  );
const digest = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
function mappings(tokens) {
  let sourceLine = 0,
    sourceColumn = 0,
    generatedLine = 0,
    generatedColumn = 0,
    first = true,
    result = "";
  const vlq = (value) => {
    let n = value < 0 ? -value * 2 + 1 : value * 2,
      s = "";
    do {
      let digit = n & 31;
      n = Math.floor(n / 32);
      if (n) digit |= 32;
      s += alphabet[digit];
    } while (n);
    return s;
  };
  for (const token of tokens) {
    while (generatedLine < token.generated[0]) {
      result += ";";
      generatedLine++;
      generatedColumn = 0;
      first = true;
    }
    if (!first) result += ",";
    result += [
      token.generated[1] - generatedColumn,
      0,
      token.original[0] - sourceLine,
      token.original[1] - sourceColumn,
    ]
      .map(vlq)
      .join("");
    [sourceLine, sourceColumn] = token.original;
    generatedColumn = token.generated[1];
    first = false;
  }
  return result;
}
function decode(text) {
  const tokens = [];
  let source = 0,
    sourceLine = 0,
    sourceColumn = 0;
  for (const [line, segments] of text.split(";").entries()) {
    let column = 0;
    for (const segment of segments.split(",").filter(Boolean)) {
      const fields = [];
      let value = 0,
        shift = 0;
      for (const char of segment) {
        const digit = alphabet.indexOf(char);
        assert.ok(digit >= 0);
        value += (digit & 31) * 2 ** shift;
        if (digit & 32) {
          shift += 5;
        } else {
          fields.push(value & 1 ? -(value >> 1) : value >> 1);
          value = 0;
          shift = 0;
        }
      }
      assert.equal(shift, 0);
      column += fields[0];
      assert.equal(fields.length, 4);
      source += fields[1];
      sourceLine += fields[2];
      sourceColumn += fields[3];
      assert.equal(source, 0);
      tokens.push({ generated: [line, column], original: [sourceLine, sourceColumn] });
    }
  }
  return tokens;
}

await test("actual source v-bind maps preserve complete outputs and literal authored UTF16 token coordinates", () => {
  assert.equal(process.env.VIZE_CSS_MAP_REQUIRE_SOURCE, "1");
  const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
  assert.equal(source, process.env.VIZE_CSS_MAP_SOURCE_SHA);
  const images = fs
    .readdirSync(path.join(root, "npm/native"))
    .filter((name) => name.endsWith(".node"));
  assert.equal(images.length, 1);
  const binary = path.join(root, "npm/native", images[0]),
    token = native.getNativeBuildToken();
  assert.match(token, /^[0-9a-f]{32}$/u);
  const changed = execFileSync("git", ["diff", "--name-only", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  })
    .trim()
    .split("\n")
    .filter(Boolean);
  assert.ok(
    changed.every((name) => ["npm/native/index.js", "npm/native/index.d.ts"].includes(name)),
  );
  packet.source = { source, binary, binarySha256: digest(binary), token };
  retain();
  assert.equal(frozen.schema, "vize.css.compile-binding-map.public-before");
  assert.equal(frozen.captures.length, 20);
  for (const entry of frozen.captures)
    for (const sourceMap of [false, true]) {
      const options = { ...entry.options, sourceMap },
        result = native.compileCss(entry.source, options);
      packet.calls.push({ id: entry.id, source: entry.source, options, result });
      retain();
      try {
        const { map, ...preserved } = result;
        assert.deepEqual(preserved, entry.output);
        if (!sourceMap || entry.tokens.length === 0) {
          assert.equal(map, undefined);
          assert.deepEqual(result, entry.output);
          continue;
        }
        const expected = {
          version: 3,
          file: "style.css",
          names: [],
          sources: ["style.css"],
          sourcesContent: [entry.source],
          mappings: mappings(entry.tokens),
        };
        assert.deepEqual(JSON.parse(map), expected);
        assert.deepEqual(decode(JSON.parse(map).mappings), entry.tokens);
        if (entry.id === "basic" && !options.minify) {
          assert.equal(entry.source, family.source);
          assert.deepEqual(options, family.options);
          assert.deepEqual(result, family.after);
        }
      } catch (error) {
        packet.failures.push(
          `${entry.id} minify=${options.minify} sourceMap=${sourceMap}: ${error.stack}`,
        );
      }
    }
  retain();
  assert.equal(packet.calls.length, 40);
  assert.deepEqual(packet.failures, []);
});
