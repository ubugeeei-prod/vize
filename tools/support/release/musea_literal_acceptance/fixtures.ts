import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { digest } from "../musea_static_acceptance/custody.ts";
export { digest } from "../musea_static_acceptance/custody.ts";

const corpusFile = "tests/_fixtures/differential/musea/literal-variant-keys.json";
export const fixtureRoots = ["tests/tooling/fixtures/musea/literal-variant-keys/", corpusFile];
export const drivers = [
  "npm/builder/vite-musea/src/literal-variant-browser.fixture.ts",
  "npm/builder/vite-musea/src/literal-variant-keys.browser.test.ts",
] as const;
const driverHashes = [
  "9bdacdaa5266f5fe21811f9ccb34dbe72408b6b0ed51577be952aeb1e99aa3aa",
  "409ee8af9a404c567a1c1cd47ad7fa21fe59dc9d4977d5d728b895fba5d39c4a",
];

/** Provider and isolated path adapters preserve the complete original assertions. */
export function adaptDriver(file: string, original: string, consumer: string) {
  let bytes = original;
  const replacements: { from: string; to: string }[] = [];
  const replace = (from: string, to: string) => {
    assert.equal(bytes.split(from).length, 2, `${file}: unique adapter source required`);
    bytes = bytes.replace(from, to);
    replacements.push({ from, to });
  };
  if (file.endsWith(".fixture.ts")) {
    replace(
      'import vize from "../../vite/src/index.ts";',
      'import vize from "@vizejs/vite-plugin";',
    );
    replace(
      'import { musea } from "./plugin/index.ts";',
      'import { musea } from "@vizejs/vite-plugin-musea";',
    );
    replace(
      'export const repository = fileURLToPath(new URL("../../../../", import.meta.url));',
      `export const repository = ${JSON.stringify(path.join(consumer, "authority"))};`,
    );
    replace(
      "url: frame.src,",
      "url: frame.src,\n        html: frame.contentDocument?.documentElement.outerHTML,",
    );
  } else {
    assert.equal(path.basename(file), "literal-variant-keys.browser.test.ts");
    // Independent identity contract: hash ordered path/NUL/name bytes, never load source plugin code.
    replace(
      'import { staticPreviewId } from "./static-data.ts";',
      'const staticPreviewId = (artPath: string, variantName: string): string => createHash("sha256").update(artPath).update("\\0").update(variantName).digest("hex").slice(0, 20);',
    );
  }
  return { bytes, replacements };
}

export function materialize(consumer: string, files: string[], read: (file: string) => Buffer) {
  const raw = read(corpusFile);
  assert.equal(digest(raw), "e83575aa334ed2071602df9595650b1116721a752379371324ac10292908fcf9");
  const corpus = JSON.parse(raw.toString()) as {
    variants: Record<string, string[]>;
    sources: { path: string; sha256: string }[];
  };
  assert.deepEqual(corpus.variants, {
    Controls: ["Default", "Custom theme"],
    Keys: ["__proto__", "constructor", "hasOwnProperty"],
  });
  assert.deepEqual(
    [...files].sort(),
    [corpusFile, ...corpus.sources.map((row) => row.path)].sort(),
  );
  const write = (file: string, bytes: Uint8Array | string) => {
    const target = path.join(consumer, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, bytes, { flag: "wx" });
    return { file, sha256: digest(typeof bytes === "string" ? Buffer.from(bytes) : bytes) };
  };
  const records = files.map((file) => ({
    source: file,
    ...write(`authority/${file}`, read(file)),
  }));
  for (const source of corpus.sources) assert.equal(digest(read(source.path)), source.sha256);
  const adapters = drivers.map((source, index) => {
    const original = read(source);
    assert.equal(digest(original), driverHashes[index], "complete original release H driver");
    records.push({ source, ...write(`originals/${source}`, original) });
    const adapted = adaptDriver(source, original.toString(), consumer);
    return {
      source,
      ...write(`drivers/${path.basename(source)}`, adapted.bytes),
      replacements: adapted.replacements,
    };
  });
  return { records, adapters };
}

export function assertSuiteCensus(log: string) {
  for (const [name, count] of Object.entries({
    tests: 2,
    pass: 2,
    fail: 0,
    cancelled: 0,
    skipped: 0,
    todo: 0,
  })) {
    const hits = [...log.matchAll(new RegExp(`^# ${name} ([0-9]+)$`, "gm"))].map((hit) => hit[1]);
    assert.deepEqual(hits, [String(count)], `unique complete ${name} census`);
  }
}
