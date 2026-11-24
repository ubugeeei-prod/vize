// Run against an independently built native addon with the opt-in bundler request.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { verifyNativeAddons } from "../../../commands/levels/native-addon-identity.ts";
import { verifyStockRuntimeImages } from "./stock-addon-qualification.ts";

interface Products {
  styles: unknown[];
  customBlocks: unknown[];
  src: { scriptSrc?: string; templateSrc?: string };
  hasScopedStyle: boolean;
}
interface Binding {
  extractSfcStyleBlocks(source: string, file?: string): unknown[];
  extractSfcCustomBlocks(source: string, file?: string): unknown[];
  extractSfcSrcInfo(source: string, file?: string): Products["src"];
  hasSfcScopedStyle(source: string, file?: string): boolean;
  inspectSfcBundlerBlocks(source: string, file?: string): Products;
}
const requireNative = createRequire(import.meta.url);
const stockRuntime = process.argv[2] === "--stock-runtime";
const stock = stockRuntime || process.argv[2] === "--stock";
function loadBindings(): [Binding, Binding] {
  if (!stock) {
    const candidate = requireNative(path.resolve(process.argv[2])) as Binding;
    return [candidate, candidate];
  }
  const images = stockRuntime
    ? verifyStockRuntimeImages(process.argv[3], process.argv[4])
    : verifyNativeAddons(process.argv[3]);
  const load = (file: string) => {
    const name = file + ".node";
    if (!fs.existsSync(name)) fs.linkSync(file, name);
    return requireNative(name) as Binding;
  };
  const base = load(images.base),
    head = load(images.head);
  assert.deepStrictEqual(
    Object.getOwnPropertyNames(base).sort(),
    Object.getOwnPropertyNames(head).sort(),
  );
  assert.equal(Object.hasOwn(head, "inspectSfcBundlerBlocks"), false);
  console.log(JSON.stringify({ stockExports: Object.getOwnPropertyNames(head).length }));
  return [base, head];
}
const [baseline, candidate] = loadBindings();
function read(joined: boolean, source: string, file: string): Products {
  if (joined && !stock) return candidate.inspectSfcBundlerBlocks(source, file);
  const binding = joined ? candidate : baseline;
  return {
    styles: binding.extractSfcStyleBlocks(source, file),
    customBlocks: binding.extractSfcCustomBlocks(source, file),
    src: binding.extractSfcSrcInfo(source, file),
    hasScopedStyle: binding.hasSfcScopedStyle(source, file),
  };
}

const cases = [
  ["empty", ""],
  ["template", "<template><p>😀</p></template>"],
  ["rejected", "<style scoped>p{}</style><script/><script/>"],
  [
    "mixed",
    '<template src="tpl.html"/><script setup src="setup.ts"/><script src="plain.ts"/><style scoped module/>' +
      '<style module="theme" src="雪.css"/><i18n lang="json" bare>{"ja":"😀"}</i18n>',
  ],
];
for (const file of [
  "npm/builder/rspack/src/test/fixtures/basic/App.vue",
  "npm/builder/rspack/src/test/fixtures/scoped-multi-blocks/App.vue",
])
  cases.push([file, fs.readFileSync(file, "utf8")]);

function bench(source: string, file: string, rounds: number, reverse: boolean): boolean {
  const measure = (joined: boolean) => {
    const start = process.hrtime.bigint();
    let count = 0;
    for (let i = 0; i < rounds; i++) {
      const result = read(joined, source, file);
      count += result.styles.length + result.customBlocks.length;
    }
    return { ns: Number((process.hrtime.bigint() - start) / BigInt(rounds)), count };
  };
  const before: number[] = [],
    after: number[] = [];
  let slower = 0;
  for (let i = 0; i < 35; i++) {
    const first = (i % 2 === 0) !== reverse;
    const a = measure(first),
      b = measure(!first);
    assert.equal(a.count, b.count);
    if (i < 4) continue;
    const [old, next] = first ? [b.ns, a.ns] : [a.ns, b.ns];
    before.push(old);
    after.push(next);
    slower += Number(next > old);
  }
  before.sort((a, b) => a - b);
  after.sort((a, b) => a - b);
  console.log(
    JSON.stringify({ file, rounds, separateNs: before[15], sharedNs: after[15], slower }),
  );
  return slower >= 23;
}
for (const [file, source] of cases) {
  assert.deepStrictEqual(read(true, source, file), read(false, source, file), file);
  if (!stock) assert.ok(!(bench(source, file, 96, false) && bench(source, file, 384, true)), file);
}

const manifestFile = process.argv[stockRuntime ? 5 : stock ? 4 : 3];
const manifest = manifestFile
  ? fs.readFileSync(manifestFile)
  : execFileSync("git", ["ls-files", "-z", "--", "*.vue", "*.vue.txt"], {
      maxBuffer: 32 * 1024 * 1024,
    });
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
let files = 0,
  invalid = 0,
  styles = 0,
  customBlocks = 0,
  scoped = 0,
  externalScripts = 0,
  externalTemplates = 0,
  bytes = 0;
for (const file of manifest.toString("utf8").split("\0").filter(Boolean)) {
  files++;
  const input = fs.readFileSync(file);
  let source: string;
  try {
    source = decoder.decode(input);
  } catch {
    invalid++;
    continue;
  }
  const actual = read(true, source, file);
  const expected = read(false, source, file);
  assert.deepStrictEqual(actual, expected, file);
  styles += actual.styles.length;
  customBlocks += actual.customBlocks.length;
  scoped += Number(actual.hasScopedStyle);
  externalScripts += Number(actual.src.scriptSrc != null);
  externalTemplates += Number(actual.src.templateSrc != null);
  bytes += Buffer.byteLength(JSON.stringify(actual));
}
assert.ok(files > invalid && styles && bytes);
console.log(
  JSON.stringify({
    files,
    invalid,
    styles,
    customBlocks,
    scoped,
    externalScripts,
    externalTemplates,
    bytes,
  }),
);
