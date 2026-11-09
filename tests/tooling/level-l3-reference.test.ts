import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { traceCompiledBackend } from "./support/level-runtime-trace.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtureRoot = path.join(repoRoot, "tests", "fixtures", "l3-reference");
const referenceRoot = path.join("tests", "level_test_support", "src", "l3_reference");
const bridgeRoot = path.join("levels", "vize_l2_to_l3", "tests");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

test("TS-28 fixture ladder is declared and non-vacuous", () => {
  const ladder = readRepoFile(referenceRoot, "matrix.rs");
  const bridge = readRepoFile(bridgeRoot, "l3_reference_fixture", "reference.rs");
  const dumps = fs
    .readdirSync(fixtureRoot)
    .filter((file) => file.endsWith(".s3.dump"))
    .sort();

  assert.ok(dumps.length >= 3, "TS-28 must start with a non-vacuous fixture ladder");
  assert.ok(
    dumps.some((dump) => dump.startsWith("rust-lowered-")),
    "TS-28 must include at least one fixture emitted by the Rust L2->L3 lowering path",
  );
  for (const dump of dumps) {
    const stem = dump.replace(/\.s3\.dump$/u, "");
    const traces = [`${stem}.trace`, `${stem}.vdom.trace`, `${stem}.vapor.trace`];
    for (const trace of traces) {
      assert.ok(fs.existsSync(path.join(fixtureRoot, trace)), `${trace} is missing`);
    }
    assert.ok(ladder.includes(`\n    "${stem}",\n`), `${stem} is not in the ladder`);
  }
  for (const suffix of ["trace", "vdom.trace", "vapor.trace"]) {
    assert.ok(ladder.includes(`("${suffix}", `), `the ladder does not check ${suffix}`);
  }
  assert.match(bridge, /check_ladder\(&fixtures\(\)\)/u);
});

test("TS-28 Rust lowering bridge is covered by an ordinary cargo test", () => {
  const bridge = readRepoFile(bridgeRoot, "l3_reference_fixture.rs");
  assert.match(bridge, /rust_lowered_fixtures_match_reference_inputs/u);
  assert.match(bridge, /tests\/fixtures\/l3-reference\/rust-lowered-static-dynamic\.s3\.dump/u);
  assert.match(bridge, /tests\/fixtures\/l3-reference\/rust-lowered-control-slots\.s3\.dump/u);
  assert.match(bridge, /L3Page::of\(&lowered\.program\)\.print_to_string\(DumpMode::Full\)/u);
  assert.match(bridge, /reference_trace_text\(&lowered\.program\)/u);
  assert.match(bridge, /backend_trace_text\(TraceBackend::Vdom, &lowered\.program\)/u);
  assert.match(bridge, /backend_trace_text\(TraceBackend::Vapor, &lowered\.program\)/u);
  assert.match(bridge, /ValuesPage::of\(&lowered\.program\)\.print_to_string\(DumpMode::Full\)/u);
  assert.match(bridge, /rust-lowered-static-dynamic\.values\.dump/u);
});

test("TS-28 stateful reference and mounted backends share full observations and steps", () => {
  const stem = "rust-lowered-static-dynamic";
  const scenario = JSON.parse(
    fs.readFileSync(path.join(fixtureRoot, `${stem}.scenario.json`), "utf8"),
  );
  const trace = JSON.parse(
    fs.readFileSync(path.join(fixtureRoot, `${stem}.behavior.json`), "utf8"),
  );
  assert.equal(trace.length, scenario.steps.length + 2);
  assert.deepEqual(trace[0].events, []);
  assert.deepEqual(trace.at(-1), { tree: [], events: ["save", "save"] });
  assert.deepEqual(
    trace.slice(0, -1).map((snapshot) => snapshot.tree[0].children[0].disabled),
    [true, true, false, false, false, false, false, false, false, true, true],
  );
  assert.deepEqual(
    trace.slice(0, -1).map((snapshot) => snapshot.tree[0].children[0].children),
    [
      ["Save"],
      ["Save"],
      ["Publish"],
      ["Publish"],
      [],
      ["42"],
      ["true"],
      ['\u96ea\n"ready"'],
      ['\u96ea\n"ready"'],
      ["Saved"],
      ["Saved"],
    ],
  );
  const mounted = readRepoFile(
    "crates",
    "vize_atelier_vapor",
    "tests",
    "level_mounted_behavior.rs",
  );
  assert.match(mounted, /rust-lowered-static-dynamic\.scenario\.json/u);
  assert.match(mounted, /rust-lowered-static-dynamic\.behavior\.json/u);
  assert.match(mounted, /assert_eq!\(\s*trace, expected,/u);
  const reference = readRepoFile(bridgeRoot, "l3_reference_fixture", "reference.rs");
  assert.match(reference, /witness::behavior\(&fixtures\(\)\)/u);
  assert.match(reference, /check\(&fixtures\(\)\.join\("rust-lowered-static-dynamic"\), run\)/u);
});

test("TS-28 control and slot stateful observations share one authored template", () => {
  const stem = "rust-lowered-control-slots";
  const scenario = JSON.parse(
    fs.readFileSync(path.join(fixtureRoot, `${stem}.scenario.json`), "utf8"),
  );
  const trace = JSON.parse(
    fs.readFileSync(path.join(fixtureRoot, `${stem}.behavior.json`), "utf8"),
  );
  assert.equal(trace.length, scenario.steps.length + 2);
  assert.deepEqual(trace.at(-1), { tree: [], events: [] });
  assert.ok(trace.every((snapshot) => snapshot.events.length === 0));
  assert.deepEqual(
    trace.slice(0, -1).map((snapshot) => snapshot.tree[0].children.map((child) => child.tag)),
    [
      ["p", "span"],
      ["span"],
      ["p", "span"],
      ["p", "span"],
      ["p", "span"],
      ["span"],
      ["p", "span"],
      ["span"],
      ["p", "span"],
    ],
  );
  assert.deepEqual(
    trace.slice(0, -1).map((snapshot) => snapshot.tree[0].children.at(-1).children),
    [
      ["fallback"],
      ["waiting"],
      ["done"],
      [],
      ["42"],
      ["false"],
      ['\u96ea\n"ready"'],
      ['\u96ea\n"ready"'],
      ["restored"],
    ],
  );
  const reference = readRepoFile(bridgeRoot, "l3_reference_fixture", "reference.rs");
  assert.match(reference, /witness::control::check\(&fixtures\(\)\)/u);
  assert.match(reference, /check\(&fixtures\(\)\.join\("rust-lowered-control-slots"\), run\)/u);
});

test("TS-28 compiled backend trace gate executes both emitted backends", async () => {
  const gate = readRepoFile("crates", "vize_atelier_vapor", "tests", "level_l3_compiled_trace.rs");
  assert.match(gate, /compiled_backend_runtime_traces_match_l3_reference_ladder/u);
  assert.match(gate, /runtime_backend_trace/u);
  assert.match(gate, /compile_template_with_options/u);
  assert.match(gate, /compile_vapor/u);
  assert.match(gate, /level-runtime-trace\.mjs/u);

  assert.deepEqual(
    await traceCompiledBackend({
      backend: "vdom",
      code: `
        import { createElementBlock, openBlock, toDisplayString } from "vue";
        export function render(_ctx, _cache) {
          return (
            openBlock(),
            createElementBlock("p", { id: "msg", textContent: toDisplayString(label) }, null, 9, ["id", "textContent"])
          );
        }
      `,
      context: { label: "ready" },
    }),
    ["create-element", "patch-prop", "set-text"],
  );
  assert.deepEqual(
    await traceCompiledBackend({
      backend: "vapor",
      code: `
        import { child, renderEffect, setText, template } from "vue";
        const t0 = template("<p></p>");
        export function render(_ctx) {
          const n0 = t0();
          const c0 = child(n0, 0);
          renderEffect(() => setText(c0, label));
          return n0;
        }
      `,
      context: { label: "ready" },
    }),
    ["create-node", "text-effect"],
  );
  await assert.rejects(
    traceCompiledBackend({
      backend: "vapor",
      code: `
        import { createForStatic } from "vue";
        export function render(_ctx) {
          return createForStatic(1, () => {});
        }
      `,
    }),
    /unsupported vue runtime helper: createForStatic/u,
  );
  assert.match(gate, /STATIC_DYNAMIC_VAPOR_COMPILED_KNOWN_GAP/u);
  assert.doesNotMatch(gate, /CONTROL_SLOTS_VAPOR_COMPILED_KNOWN_GAP/u);
  assert.match(gate, /rust-lowered-static-dynamic\.vdom\.trace/u);
  assert.match(gate, /rust-lowered-static-dynamic\.vapor\.trace/u);
  assert.match(gate, /rust-lowered-control-slots\.vdom\.trace/u);
  assert.match(gate, /rust-lowered-control-slots\.vapor\.trace/u);
});

test("TS-28 command in the suite registry names the executable runner", () => {
  const suites = readRepoFile("docs/davinci", "plan", "test-suites.md");
  const rows = suites
    .split("\n")
    .filter((line) => line.startsWith("|"))
    .map((line) =>
      line
        .split("|")
        .slice(1, -1)
        .map((cell) => cell.trim()),
    )
    .filter((cells) => cells[0] === "TS-28");
  assert.deepEqual(rows, [
    [
      "TS-28",
      "Reference differential",
      "`cargo test -p vize_l2_to_l3 --test l3_reference_fixture && cargo test -p vize_atelier_vapor --test level_l3_compiled_trace --test level_mounted_behavior`",
      "exact agreement on observable semantics; stateful subset and remaining operation-order gaps are explicit",
      "P3-4",
    ],
  ]);
});
