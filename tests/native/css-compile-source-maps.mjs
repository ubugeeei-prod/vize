import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadCompilerManifest } from "../differential/compiler.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const fixture = fileURLToPath(
  new URL("../fixtures/css/compile-source-maps/public-0.571.0.json", import.meta.url),
);
const frozen = JSON.parse(fs.readFileSync(fixture, "utf8"));
const family = loadCompilerManifest(
  path.join(root, "tests/_fixtures/differential/compiler/manifest.json"),
).cssApiCases;
assert.equal(family.length, 1);
const errors = JSON.parse(
  fs.readFileSync(
    new URL("../fixtures/css/compile-source-maps/public-error-neighbor.json", import.meta.url),
    "utf8",
  ),
);
const require = createRequire(import.meta.url);
const native = require("../../npm/native/index.js");
const output = path.join(root, "target/css-compile-source-maps");
fs.mkdirSync(output, { recursive: true });
const packet = {
  source: null,
  fixtureSha256: createHash("sha256").update(fs.readFileSync(fixture)).digest("hex"),
  calls: [],
};
const retain = () =>
  fs.writeFileSync(path.join(output, "observation.json"), JSON.stringify(packet, null, 2) + "\n");
const readSource = () => {
  assert.equal(process.env.VIZE_CSS_MAP_REQUIRE_SOURCE, "1");
  const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
  assert.equal(source, process.env.VIZE_CSS_MAP_SOURCE_SHA);
  const binaries = fs
    .readdirSync(path.join(root, "npm/native"))
    .filter((name) => name.endsWith(".node"));
  assert.equal(binaries.length, 1);
  const binary = path.join(root, "npm/native", binaries[0]);
  const token = native.getNativeBuildToken();
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
  return {
    source,
    binary,
    binarySha256: createHash("sha256").update(fs.readFileSync(binary)).digest("hex"),
    token,
  };
};

const unqualified = new Set([
  "scoped-flat",
  "scoped-nested",
  "scoped-slotted",
  "v-bind",
  "empty",
  "comment",
  "parse-error",
  "guarded",
  "embedded-map",
]);

await test("real source native CSS preserves every frozen public field and emits only admitted whole maps", () => {
  packet.source = readSource();
  retain();
  assert.equal(frozen.schema, "vize.css.compile-source-map.public-before");
  assert.equal(frozen.captures.length, 40);
  const failures = [];
  for (const entry of [...frozen.captures, ...errors.captures]) {
    for (const sourceMap of [false, true]) {
      const options = { ...entry.options, sourceMap };
      const result = native.compileCss(entry.source, options);
      packet.calls.push({ id: entry.id, source: entry.source, options, result });
      retain();
      try {
        const { map, ...preserved } = result;
        assert.deepEqual(preserved, entry.output);
        if (entry.id === "unicode-crlf" && options.minify) {
          assert.equal(entry.source, family[0].source);
          if (sourceMap) {
            assert.deepEqual(options, family[0].options);
            assert.deepEqual(result, family[0].after);
          } else assert.deepEqual(result, family[0].before);
        }
        if (!sourceMap || unqualified.has(entry.id)) {
          assert.equal(map, undefined, `${entry.id} must not claim transformed source ownership`);
          assert.deepEqual(result, entry.output);
        } else {
          assert.equal(typeof map, "string", entry.id);
          const value = JSON.parse(map);
          assert.deepEqual(Object.keys(value).sort(), [
            "file",
            "mappings",
            "names",
            "sources",
            "sourcesContent",
            "version",
          ]);
          assert.equal(value.version, 3);
          assert.equal(value.file, options.filename ?? "style.css");
          assert.deepEqual(value.sources, [options.filename ?? "style.css"]);
          assert.deepEqual(value.sourcesContent, [entry.source]);
          assert.deepEqual(value.names, []);
          assert.notEqual(value.mappings, "");
          if (entry.id === "unicode-crlf") {
            assert.deepEqual(value, {
              version: 3,
              file: options.filename,
              sources: [options.filename],
              sourcesContent: [entry.source],
              names: [],
              mappings: options.minify ? "AACA,eACA" : "AACA;;;;AACA",
            });
          }
          if (entry.id === "same-line-astral" && options.minify) {
            assert.equal(value.mappings, "AAAA,mBAAmB");
          }
        }
      } catch (error) {
        failures.push(
          `${entry.id} minify=${options.minify} sourceMap=${sourceMap}: ${error.stack}`,
        );
      }
    }
  }
  packet.failures = failures;
  retain();
  assert.deepEqual(failures, []);
  assert.equal(packet.calls.length, 84);
});
