/** Materialize immutable #10845/#10682 audit cases without changing the corpus. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

interface AuditCase {
  readonly name: string;
  readonly bytes: number;
  readonly sha256: string;
  readonly require_admission: boolean;
  historical_id?: string;
}

interface HistoricalCase {
  readonly id: string;
  readonly input_sha256: string;
  readonly require_admission: boolean;
  readonly controls_sha256: string;
  readonly "program-observer_sha256": string;
}

const [root, destination] = process.argv.slice(2);
assert.ok(root && destination, "usage: expression-overlap-cases.ts ROOT DESTINATION");
mkdirSync(destination, { recursive: true });
const corpus = path.join(root, "tests/fuzz/regressions/js_ts_expression");
const expected = {
  "issue-10845.input": "60c49daaab7e06a062b0ffa7dced0eb4ddab9dbee1b52f1657fc4f1c08aa14a5",
  "issue-10845.slow-1.input": "de0029da1a229d4e336d73374bedd33e64649c741a28752d56b4dee40076f60a",
  "issue-10845.slow-2.input": "e73212cb497085f26e85b59ef09aabc4bbdb485965f71ebb8899dc666b89e293",
  "issue-10845.min.input": "73e51fe565438b9e18d8696f4bc57d368310898539e1f406387b7592a9e1a926",
  "issue-10401.input": "6d668844f745732cdc92bdafa21f72ba5d09d9c14305d4ea9174fb802493dcc8",
};
const manifest: AuditCase[] = [];

function sha256(data: Uint8Array): string {
  return createHash("sha256").update(data).digest("hex");
}

function emit(name: string, data: Uint8Array, admitted: boolean): AuditCase {
  writeFileSync(path.join(destination, name), data);
  const entry: AuditCase = {
    name,
    bytes: data.length,
    sha256: sha256(data),
    require_admission: admitted,
  };
  manifest.push(entry);
  return entry;
}

for (const [name, digest] of Object.entries(expected)) {
  const data = readFileSync(path.join(corpus, name));
  assert.equal(sha256(data), digest, name);
  emit(name, data, true);
}
assert.deepEqual(
  readFileSync(path.join(root, "vendor/oxc_parser/src/speculation/fixtures/issue-10845.input")),
  readFileSync(path.join(corpus, "issue-10845.input")),
);
const original = readFileSync(path.join(corpus, "issue-10401.input"));
for (const [label, suffix] of [
  ["x", "\n// x"],
  ["greater", "\n// >"],
] as const) {
  emit(`10401-append-${label}.input`, Buffer.concat([original, Buffer.from(suffix)]), true);
}
for (const size of [4096, 8192, 16384]) {
  // Historical #10682 raw size controls: admission rejects these prefixes.
  emit(
    `10401-prefix-${size}.input`,
    Buffer.concat([original.subarray(0, size), Buffer.from("\n// >")]),
    false,
  );
}
for (const [name, source] of [
  ["source-fact-jsdoc.input", "/** @type {number} */ const x = 1;"],
  ["source-fact-legacy.input", "077"],
  ["source-fact-generics.input", "const result = f<Map<string, Array<number>>>(value);"],
] as const) {
  emit(name, Buffer.from(source), true);
}
for (const count of [64, 128, 256]) {
  emit(`minimal-overlap-${count}.input`, Buffer.from("a < [a < [".repeat(count) + ">"), false);
}
const archive = path.join(root, "tools/support/compat/parser/historical-10682.tar.gz");
assert.equal(
  sha256(readFileSync(archive)),
  "780df7dfa9517de9155d5dffa8ec167a1f9c98391f1dd107f735be80d361185a",
);
assert.equal(
  sha256(readFileSync(path.join(root, "tools/support/compat/parser/frozen-10682-observer.rs"))),
  "9d095b94ec587d9f22cfa696a1bb5c6bd23813eced6a633d2989c6a270f05238",
);
const packets = path.join(destination, "historical-packets");
mkdirSync(packets);
function retained(name: string): Buffer {
  return execFileSync("tar", ["-xOzf", archive, name], { maxBuffer: 16 * 1024 * 1024 });
}
const indexBytes = retained("index.json");
// The archive hash above authenticates this schema and every retained packet.
const index = JSON.parse(indexBytes.toString("utf8")) as {
  readonly cases: readonly HistoricalCase[];
};
writeFileSync(path.join(packets, "index.json"), indexBytes);
for (const entry of index.cases) {
  const name = entry.id;
  assert.match(name, /^[a-z0-9-]+$/u);
  const data = retained(`cases/${name}.input`);
  assert.equal(sha256(data), entry.input_sha256);
  emit(`historical-10682-${name}.input`, data, entry.require_admission).historical_id = name;
  for (const kind of ["controls", "program-observer"] as const) {
    const packet = retained(`packets/${name}.${kind}.json`);
    assert.equal(sha256(packet), entry[`${kind}_sha256`]);
    writeFileSync(path.join(packets, `${name}.${kind}.json`), packet);
  }
}
writeFileSync(path.join(destination, "manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
