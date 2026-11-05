import assert from "node:assert/strict";
import { createHash } from "node:crypto";

interface Routes {
  cheap: string[];
  direct: string[];
  subtypeWitness: string;
  inferred: string;
}

interface EmitGateOracle {
  version: 1;
  historical: Routes & {
    producer: string;
    inference: string;
    inferenceSha256: string;
    helpers: string;
    sourceHashes: Record<string, string>;
  };
  current: Routes & { source: string; producer: string; sourceHashes: Record<string, string> };
}

const fixture = "tests/_fixtures/tooling/emit-props-gate-11697.json";
const fixtureSha256 = "cf5afd61920c8a2c85fb680d701570f7d7572c988ff28b77f61608e4d543335d";
const successorFixture = "tests/_fixtures/tooling/emit-props-gate-current-source-20261105.json";
const successorSha256 = "0de58b21496792302fc4139a2acea9fa24d66f6972e7361a9e5e31ffc30fb824";
const producers = [
  "crates/vize_canon/src/virtual_ts/scope/component_events/generic_inference.rs",
  "crates/vize_canon/src/virtual_ts/scope/component_events.rs",
];
const resolver = "crates/vize_canon/src/virtual_ts/scope/component_events/emit_resolver_share.rs";
const arrays = "crates/vize_canon/src/virtual_ts/scope/component_events/prop_array_share.rs";
const sha256 = (text: string): string => createHash("sha256").update(text).digest("hex");

interface SourceIdentity {
  bytes: number;
  sha256: string;
}
interface CurrentSourceSuccessor {
  schema: string;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  predecessor: SourceIdentity & { path: string };
  sources: Array<{
    path: string;
    before: SourceIdentity;
    after: SourceIdentity;
    hunks: Array<{ before: string; after: string }>;
  }>;
  dependencies: Array<SourceIdentity & { path: string }>;
}

// This inverse authenticates source custody only. Callers keep reading the
// actual generator templates and helpers when checking the current candidate.
function historicalProducer(
  read: (path: string) => string,
  path: string,
  current: string,
  expected: string,
): string {
  // #11913 changes only the static loose-arity producer. Recover the exact
  // already reviewed resolver producer before entering its frozen custody chain.
  if (
    path === producers[0] &&
    sha256(current) === "17fec691f4f92415b730dd4ce4d684056719c3045f31dc4b749e5c480aa2b771"
  ) {
    assert.equal(Buffer.byteLength(current), 18626, "selected emit gate producer successor");
    const addition =
      '    // A static component\'s loose sentinel means undeclared runtime arity,\n    // not a declared zero-argument event. Dynamic targets retain the empty\n    // contextual tuple; their contextless marker widens runtime args later.\n    let (loose_marker, loose_args) = if dynamic_component {\n        ("infer __L extends unknown[]", "__L")\n    } else {\n        ("unknown[]", "unknown[]")\n    };\n    append!(\n        *ts,\n        "{indent}type {inferred_args} = {library} extends true ? (typeof {emit_props} extends {{ __ctx?: infer __X }} ? NonNullable<__X> extends {{ props: infer __P }} ? __P : {{}} : {{}}) extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[] : typeof {emit_props} extends {{ readonly __vizeLooseListener: {loose_marker} }} ? {loose_args} : typeof {emit_props} extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[];\\n",\n';
    assert.equal(current.split(addition).length, 2, "selected emit gate producer successor");
    current = current.replace(
      addition,
      '    append!(\n        *ts,\n        "{indent}type {inferred_args} = {library} extends true ? (typeof {emit_props} extends {{ __ctx?: infer __X }} ? NonNullable<__X> extends {{ props: infer __P }} ? __P : {{}} : {{}}) extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[] : typeof {emit_props} extends {{ readonly __vizeLooseListener: infer __L extends unknown[] }} ? __L : typeof {emit_props} extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[];\\n",\n',
    );
    assert.equal(
      sha256(current),
      "f231db13ede1284c8aa291ccd67bec0c50f7e68a510a1969b83e8e6c1fa70186",
      "selected emit gate producer successor",
    );
  }
  if (!producers.includes(path) || sha256(current) === expected) return current;
  const context = "selected emit gate producer successor";
  const raw = read(successorFixture);
  assert.equal(sha256(raw), successorSha256, context);
  const successor = JSON.parse(raw) as CurrentSourceSuccessor;
  assert.equal(successor.schema, "vize.emitPropsGateCurrentSource.v1", context);
  assert.deepEqual(
    successor.credit,
    { compiled: false, native: false, performance: false },
    context,
  );
  assert.equal(successor.predecessor.path, fixture, context);
  assert.equal(successor.predecessor.sha256, fixtureSha256, context);
  assert.equal(Buffer.byteLength(read(fixture)), successor.predecessor.bytes, context);
  assert.deepEqual(
    successor.sources.map((entry) => entry.path),
    producers,
    context,
  );
  const source = successor.sources.find((entry) => entry.path === path)!;
  assert.equal(source.before.sha256, expected, context);
  assert.equal(Buffer.byteLength(current), source.after.bytes, context);
  assert.equal(sha256(current), source.after.sha256, context);
  assert.deepEqual(
    successor.dependencies.map((entry) => entry.path),
    [resolver, arrays],
    context,
  );
  for (const dependency of successor.dependencies) {
    const body = read(dependency.path);
    assert.equal(Buffer.byteLength(body), dependency.bytes, context);
    assert.equal(sha256(body), dependency.sha256, context);
  }
  assert.equal(source.hunks.length, path === producers[0] ? 1 : 2, context);
  let before = current;
  for (const hunk of source.hunks) {
    assert.notEqual(hunk.before, hunk.after, context);
    assert.equal(before.split(hunk.after).length, 2, context);
    before = before.replace(hunk.after, hunk.before);
  }
  assert.equal(Buffer.byteLength(before), source.before.bytes, context);
  assert.equal(sha256(before), source.before.sha256, context);
  let forward = before;
  for (const hunk of [...source.hunks].reverse()) {
    assert.equal(forward.split(hunk.before).length, 2, context);
    forward = forward.replace(hunk.before, hunk.after);
  }
  assert.equal(forward, current, context);
  return before;
}

/** Select the authored successor only for its complete, physical producer image. */
export function emitGateOracle(read: (path: string) => string): EmitGateOracle {
  const raw = read(fixture);
  assert.equal(sha256(raw), fixtureSha256, "immutable emit gate oracle context");
  const oracle = JSON.parse(raw) as EmitGateOracle;
  assert.equal(sha256(oracle.historical.inference), oracle.historical.inferenceSha256);
  for (const [path, expected] of Object.entries(oracle.current.sourceHashes)) {
    assert.equal(
      sha256(historicalProducer(read, path, read(path), expected)),
      expected,
      `selected emit gate producer: ${path}`,
    );
  }
  return oracle;
}
