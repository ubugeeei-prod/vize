import assert from "node:assert/strict";
import { createHash, webcrypto } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createContext, runInContext } from "node:vm";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = readFileSync(path.join(root, "docs/scripts/capture-page-render.ts"), "utf8");
type Rectangle = [number, number, number, number];
type Record = { type: number; source: string | null; bounds: Rectangle; rectangles: Rectangle[] };
type Snapshot = {
  content: unknown;
  before: (Record & { node: unknown })[];
  snapshot: () => (Record & { node: unknown })[];
};
type Proof = {
  descendants: number;
  elements: number;
  textNodes: number;
  rectangles: number;
  beforeSha256: string;
  afterSha256: string;
  identical: boolean;
};

function closure(marker: string): string {
  const start = source.indexOf(marker);
  assert(start >= 0 && source.indexOf(marker, start + marker.length) === -1, marker);
  const stop = source.indexOf("\n      });", start + marker.length);
  assert(stop > start, "literal evaluate callback must retain its source boundary");
  return stripTypeScriptTypes(`(${source.slice(start + marker.length, stop)}\n})`);
}
const producer = closure("original = await page.evaluateHandle(");
const current = closure("layout = await original.evaluate(");

// Original literal proof closure at 705d927623a91a7bdd930b34f3656ed2d1862874.
// The implementation under test is extracted above, rather than reimplemented here.
const original = `(async ({ content, snapshot, before }) => {
  const after = snapshot();
  const packets = (nodes) => nodes.map(({ node: _node, ...packet }) => packet);
  const beforePackets = packets(before);
  const afterPackets = packets(after);
  const hash = async (packet) => [
    ...new Uint8Array(await crypto.subtle.digest(
      "SHA-256", new TextEncoder().encode(JSON.stringify(packet)),
    )),
  ].map((byte) => byte.toString(16).padStart(2, "0")).join("");
  const sameNodes = document.querySelector(".content") === content &&
    before.length === after.length && before.every((entry, index) => entry.node === after[index].node);
  return {
    descendants: before.length - 1,
    elements: before.filter((entry) => entry.type === Node.ELEMENT_NODE).length,
    textNodes: before.filter((entry) => entry.type === Node.TEXT_NODE).length,
    rectangles: before.reduce((sum, entry) => sum + entry.rectangles.length, 0),
    beforeSha256: await hash(beforePackets),
    afterSha256: await hash(afterPackets),
    identical: sameNodes && JSON.stringify(beforePackets) === JSON.stringify(afterPackets),
  };
})`;

const literal = `日本語 😃 e\u0301 \u2028 \u2029 \ud800 \udfff \u0000\n\t"\\ <`;
const fixture: Record[] = [
  {
    type: 1,
    source: `<div z="last" a="first"><code>${literal}</code></div>`,
    bounds: [0, 0, 1440, 10000],
    rectangles: [[0, 0, 1440, 10000]],
  },
  {
    type: 3,
    source: literal,
    bounds: [-0, 0.125, 319.75, 24],
    rectangles: [
      [0, 0.125, 319.75, 24],
      [0, 24.125, 87.5, 24],
    ],
  },
  { type: 8, source: "literal comment", bounds: [0, 0, 0, 0], rectangles: [] },
  { type: 1, source: "<span></span>", bounds: [0, -1.5, 0, 0], rectangles: [] },
];

type Options = {
  replaceNode?: boolean;
  rejectDigest?: number;
  throwStringify?: number;
  mutateDocument?: boolean;
  inheritedToJson?: boolean;
};
async function exercise(code: string, after: Record[], options: Options = {}) {
  const events: string[] = [];
  const strings: string[] = [];
  const bytes: Buffer[] = [];
  let serializations = 0;
  let digests = 0;
  const context = createContext({
    fixture: structuredClone(fixture),
    afterRecords: structuredClone(after),
    TextEncoder,
    Uint8Array,
  });
  runInContext(
    `
    let records = fixture;
    class DocumentNode {
      constructor(index) { this.index = index; }
      get nodeType() { return records[this.index].type; }
      get nodeValue() { return records[this.index].source; }
    }
    class Element extends DocumentNode {
      get outerHTML() { return records[this.index].source; }
      getBoundingClientRect() { return rect(records[this.index].bounds); }
      getClientRects() { return records[this.index].rectangles.map(rect); }
    }
    const rect = ([x, y, width, height]) => ({ x, y, width, height });
    const makeNode = (index) => new (records[index].type === 1 ? Element : DocumentNode)(index);
    let nodes = records.map((_, index) => makeNode(index));
    const Node = { ELEMENT_NODE: 1, TEXT_NODE: 3 };
    const NodeFilter = { SHOW_ALL: -1 };
    const document = {
      querySelector: () => nodes[0],
      createTreeWalker: () => { let index = 1; return { nextNode: () => nodes[index++] ?? null }; },
      createRange: () => {
        let node;
        return {
          selectNodeContents: (value) => { node = value; },
          getBoundingClientRect: () => rect(records[node.index].bounds),
          getClientRects: () => records[node.index].rectangles.map(rect),
        };
      },
    };
  `,
    context,
  );
  const snapshot = (runInContext(producer, context) as () => Snapshot)();
  // The actual producer owns fresh data arrays; DOM references remain only in node.
  for (const entry of snapshot.before) {
    assert.deepEqual(Object.keys(entry), ["node", "type", "source", "bounds", "rectangles"]);
    const inspect = (value: unknown): void => {
      if (value === null || typeof value !== "object") {
        assert(["string", "number"].includes(typeof value) || value === null);
        return;
      }
      assert(Array.isArray(value));
      for (const descriptor of Object.values(Object.getOwnPropertyDescriptors(value))) {
        assert("value" in descriptor && !descriptor.get && !descriptor.set);
        inspect(descriptor.value);
      }
    };
    for (const key of ["type", "source", "bounds", "rectangles"] as const) {
      const descriptor = Object.getOwnPropertyDescriptor(entry, key);
      assert(descriptor && "value" in descriptor && !descriptor.get && !descriptor.set);
      inspect(descriptor.value);
    }
  }
  runInContext(
    "records = afterRecords; nodes = records.map((_, index) => nodes[index] ?? makeNode(index));",
    context,
  );
  if (options.replaceNode) runInContext("nodes[1] = makeNode(1);", context);
  const expectedPackets = [snapshot.before, snapshot.snapshot()].map((entries) =>
    entries.map(({ node: _node, ...packet }) => packet),
  );
  const expectedJson = expectedPackets.map((packet) => JSON.stringify(packet));
  if (options.inheritedToJson)
    runInContext(
      'let roots = 0; Array.prototype.toJSON = function(key) { return key === "" && ++roots > 2 ? [...this, roots] : this; };',
      context,
    );
  const nativeJson = runInContext("JSON.stringify", context) as typeof JSON.stringify;
  context.recordStringify = (value: unknown, ...extra: unknown[]) => {
    assert.equal(extra.length, 0, "packet serialization has no replacer or formatting argument");
    serializations += 1;
    events.push(`json:${serializations}`);
    if (serializations === options.throwStringify) throw new Error("stringify rejected");
    const text = nativeJson(value);
    strings.push(text);
    return text;
  };
  runInContext("JSON.stringify = recordStringify;", context);
  context.crypto = {
    subtle: {
      digest: async (algorithm: string, data: Uint8Array) => {
        assert.equal(algorithm, "SHA-256");
        const index = ++digests;
        events.push(`digest:${index}`);
        bytes.push(Buffer.from(data));
        await Promise.resolve();
        if (options.mutateDocument)
          runInContext(
            "records[0].source = 'changed during await'; records[1].bounds[2] += 1;",
            context,
          );
        if (index === options.rejectDigest) throw new Error(`digest ${index} rejected`);
        const hash = await webcrypto.subtle.digest(algorithm, data);
        events.push(`resolved:${index}`);
        return hash;
      },
    },
  };
  let proof: Proof | undefined;
  let error: unknown;
  try {
    const evaluate = runInContext(code, context) as (value: Snapshot) => Promise<Proof>;
    proof = JSON.parse(JSON.stringify(await evaluate(snapshot))) as Proof;
  } catch (failure) {
    error = failure instanceof Error ? failure.message : String(failure);
  }
  return { proof, error, events, strings, bytes, expectedJson };
}

test("actual capture proof reuses complete packet JSON without changing authority, bytes or await order", async () => {
  const inventory = readdirSync(path.join(root, "docs/scripts"), { recursive: true })
    .filter((file): file is string => typeof file === "string" && file.endsWith(".ts"))
    .map(
      (file) =>
        [
          file,
          (
            readFileSync(path.join(root, "docs/scripts", file), "utf8").match(
              /capturePageRender\s*\(/g,
            ) ?? []
          ).length,
        ] as const,
    )
    .filter(([, count]) => count > 0);
  assert.deepEqual(
    inventory.sort(([left], [right]) => left.localeCompare(right)),
    [
      ["capture-page-render.ts", 1],
      ["capture-render-controls.ts", 1],
      ["navigation-render-page.ts", 2],
    ],
  );
  const variants: { name: string; after: Record[]; replaceNode?: boolean }[] = [
    { name: "complete Unicode and source", after: structuredClone(fixture) },
    { name: "original node identity", after: structuredClone(fixture), replaceNode: true },
  ];
  for (const [name, change] of [
    [
      "attribute order",
      (records: Record[]) =>
        (records[0].source =
          records[0].source?.replace('z="last" a="first"', 'a="first" z="last"') ?? null),
    ],
    ["source whitespace", (records: Record[]) => (records[0].source += " ")],
    [
      "Unicode code unit",
      (records: Record[]) => (records[1].source = literal.replace("\ud800", "\ufffd")),
    ],
    ["one pixel element bound", (records: Record[]) => (records[0].bounds[2] += 1)],
    ["text Range only", (records: Record[]) => (records[1].rectangles[0][2] += 1)],
    ["rectangle order", (records: Record[]) => records[1].rectangles.reverse()],
    [
      "node array order",
      (records: Record[]) => ([records[1], records[2]] = [records[2], records[1]]),
    ],
    ["descendant count", (records: Record[]) => records.pop()],
  ] as const) {
    const after = structuredClone(fixture);
    change(after);
    variants.push({ name, after });
  }
  for (const variant of variants) {
    const baseline = await exercise(original, variant.after, variant);
    const cached = await exercise(current, variant.after, variant);
    assert.equal(cached.error, undefined, variant.name);
    assert.deepEqual(cached.proof, baseline.proof, variant.name);
    assert.equal(
      cached.proof?.identical,
      variant.name === "complete Unicode and source",
      variant.name,
    );
    assert.deepEqual(cached.strings, cached.expectedJson, variant.name);
    assert.deepEqual(cached.bytes, baseline.bytes, variant.name);
    assert.deepEqual(
      cached.bytes,
      cached.expectedJson.map((text) => Buffer.from(text, "utf8")),
      variant.name,
    );
    assert.equal(
      cached.proof?.beforeSha256,
      createHash("sha256").update(cached.bytes[0]).digest("hex"),
    );
    assert.equal(
      cached.proof?.afterSha256,
      createHash("sha256").update(cached.bytes[1]).digest("hex"),
    );
    assert.equal(
      baseline.strings.length,
      variant.replaceNode || variant.name === "descendant count" ? 2 : 4,
    );
    assert.deepEqual(cached.events, [
      "json:1",
      "digest:1",
      "resolved:1",
      "json:2",
      "digest:2",
      "resolved:2",
    ]);
  }
  const mutated = await exercise(current, fixture, { mutateDocument: true });
  const baselineMutated = await exercise(original, fixture, { mutateDocument: true });
  assert.deepEqual(
    mutated.proof,
    baselineMutated.proof,
    "DOM mutation does not mutate already captured private packets",
  );
  assert.equal(mutated.proof?.identical, true);
  for (const options of [
    { rejectDigest: 1, throwStringify: 2 },
    { rejectDigest: 2 },
    { throwStringify: 1 },
    { throwStringify: 2 },
  ]) {
    const baseline = await exercise(original, fixture, options);
    const cached = await exercise(current, fixture, options);
    assert.deepEqual(
      cached.events,
      baseline.events,
      "serialization remains on its original side of each await",
    );
    assert.equal(cached.error, baseline.error);
    assert.equal(cached.proof, undefined);
  }
  const outsideContract = { inheritedToJson: true };
  const hookedOriginal = await exercise(original, fixture, outsideContract);
  const hookedCurrent = await exercise(current, fixture, outsideContract);
  assert.equal(hookedOriginal.proof?.beforeSha256, hookedCurrent.proof?.beforeSha256);
  assert.equal(hookedOriginal.proof?.afterSha256, hookedCurrent.proof?.afterSha256);
  assert.equal(hookedOriginal.proof?.identical, false);
  assert.equal(
    hookedCurrent.proof?.identical,
    true,
    "stateful inherited toJSON is ineligible under the native-intrinsics contract",
  );
});
