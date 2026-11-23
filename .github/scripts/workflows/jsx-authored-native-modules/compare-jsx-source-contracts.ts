import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv, writeJson } from "../shared/files.ts";

interface Packet {
  source: string;
  name: string;
  mode: string;
  stable: boolean;
  counters: unknown;
}
interface Role {
  source: string;
  components: unknown;
  module: string;
  diagnostics: { start: number; end: number }[];
}
interface Profile {
  source: string;
  ssr: boolean;
  map: boolean;
  iterationsPerSample: number;
  output: { diagnostics: unknown[]; map: string | null };
  samplesNs: number[];
  counters: { name: string; total: number }[];
}
const dir = join(requiredEnv("RUNNER_TEMP"), "jsx-authored-native");
const load = <T>(name: string) => readJson<T>(join(dir, name + ".json"));
const base = load<Profile[]>("base-profile"),
  head = load<Profile[]>("head-profile"),
  stable = load<Profile[]>("stability-base-profile");
assert.equal(base.length, 16);
assert.equal(head.length, 16);
assert.equal(stable.length, 16);
const beforePackets = load<Packet[]>("stability-base-packets"),
  afterPackets = load<Packet[]>("component-stability");
assert.equal(beforePackets.length, 28);
assert.equal(afterPackets.length, 28);
for (const [index, packet] of afterPackets.entries()) {
  const previous = beforePackets[index]!;
  for (const key of ["source", "name", "mode", "stable"] as const)
    assert.deepEqual(previous[key], packet[key], key);
  if (packet.mode === "Vdom" || !packet.stable) {
    const { counters: _before, ...a } = previous,
      { counters: _after, ...b } = packet;
    assert.deepEqual(a, b);
  }
}
const beforeRoles = load<Role[]>("base-functional-role"),
  afterRoles = load<Role[]>("head-functional-role");
assert.equal(beforeRoles.length, 6);
assert.equal(afterRoles.length, 6);
for (const [index, after] of afterRoles.entries()) {
  const before = beforeRoles[index]!;
  assert.equal(before.source, after.source);
  assert.deepEqual(before.components, after.components);
  if (index === 0) {
    assert.equal(after.module, "");
    assert.equal(after.diagnostics.length, 1);
    if (!before.diagnostics.length) {
      assert.ok(before.module.includes("function ssrRender"));
      assert.ok(!before.module.includes("const A ="));
    }
  } else if (index === 3) {
    assert.equal(after.diagnostics.length, 0);
    assert.ok(after.module.includes("__ssrInlineRender: true"));
  } else {
    assert.equal(after.diagnostics.length, 1);
    assert.equal(after.module, "");
    const { start, end } = after.diagnostics[0]!;
    assert.ok(["1", "`text`"].includes(Array.from(after.source).slice(start, end).join("")));
  }
}
function median(numbers: number[]) {
  assert.ok(numbers.length);
  const sorted = [...numbers].sort((a, b) => a - b),
    mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[mid]! : (sorted[mid - 1]! + sorted[mid]!) / 2;
}
function compare(previous: Profile, current: Profile) {
  for (const key of ["source", "ssr", "map", "iterationsPerSample"] as const)
    assert.deepEqual(previous[key], current[key], key);
  const before = median(previous.samplesNs),
    after = median(current.samplesNs);
  return {
    source: current.source,
    ssr: current.ssr,
    map: current.map,
    baseMedianNs: before,
    headMedianNs: after,
    ratio: after / before,
    counters: current.counters,
  };
}
const rows = head.map((current, index) => {
  const previous = base[index]!;
  assert.equal(current.output.diagnostics.length, 0);
  const counts = new Map(current.counters.map(({ name, total }) => [name, total]));
  function count(name: string) {
    const result = counts.get(name);
    assert.ok(result !== undefined, name);
    return result;
  }
  assert.ok((counts.get("atelier.jsx.lower.component_stability_builds") ?? 0) <= 1);
  const roots = count("atelier.jsx.module.native_roots");
  for (const kind of ["candidates", "semantic_builds", "refusals"])
    assert.equal(count("atelier.jsx.module.component_use_" + kind), 0);
  assert.equal(count("atelier.jsx.module.native_plans"), roots);
  assert.equal(count("atelier.jsx.module.native_generated_maps"), current.map ? roots : 0);
  if (current.ssr) assert.equal(count("l4.ssr.accepted"), roots);
  else
    assert.equal(
      count("atelier.vapor.transformed_root.origin_collections"),
      current.map ? roots : 0,
    );
  if (current.map) {
    assert.ok(current.output.map);
    assert.deepEqual(
      (JSON.parse(current.output.map) as { sourcesContent: string[] }).sourcesContent,
      [current.source],
    );
  } else assert.equal(current.output.map, null);
  assert.equal(previous.samplesNs.length, 9);
  assert.equal(current.samplesNs.length, 9);
  return {
    ...compare(previous, current),
    baseContract: previous.output.diagnostics.length ? "explicit-refusal" : "complete-module",
    headContract: "complete-module",
  };
});
writeJson(join(dir, "comparison.json"), {
  scope:
    "same sources with raw before/head outcomes retained; ratios describe the observed contracts and do not claim equivalent work or a speedup",
  rows,
});
const stableRows = head.map((current, index) => {
  const previous = stable[index]!;
  assert.equal(previous.output.diagnostics.length, 0);
  assert.equal(current.output.diagnostics.length, 0);
  return {
    ...compare(previous, current),
    intentionalStaticValueChange: current.source.startsWith("const Child={};") && !current.ssr,
  };
});
writeJson(join(dir, "stability-comparison.json"), {
  scope:
    "literal guarded664 source versus head; exact generated-output transitions are verified separately and raw timings are descriptive",
  rows: stableRows,
});
