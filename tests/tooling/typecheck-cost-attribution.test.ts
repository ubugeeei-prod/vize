import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test, type TestContext } from "node:test";

import {
  attributeLane,
  pairLanes,
} from "../../tools/benchmarks/scripts/typecheck-cost-attribution.ts";
import {
  enclosingDeclaration,
  helperDeclarations,
  localAlias,
  normalizeAlias,
  sectionStarts,
  siteFamily,
  sourceText,
} from "../../tools/benchmarks/scripts/typecheck-cost-families.ts";
import { renderAttribution } from "../../tools/benchmarks/scripts/typecheck-cost-report.ts";
import {
  checkerFrames,
  tickStacks,
  varianceSpans,
} from "../../tools/benchmarks/scripts/typecheck-cost-trace.ts";
import { censusTypes } from "../../tools/benchmarks/scripts/typecheck-cost-types.ts";

const GENERATED = [
  "// ========== Module Scope (imports) ==========",
  "import { ref } from 'vue';",
  "// ========== Setup Scope ==========",
  "function __setup() {",
  "  // User setup code",
  "  const count = ref(0);",
  "  // @vize-map: 0:10 -> 0:10",
  "  // ========== Template Scope (inherits from setup) ==========",
  "  const __vize_template = (function __template() {",
  "  const __vize_prop_check_0_to: __MkA_0_prop_to = to;",
  "  type __MkA_22_click_args = unknown[];",
  "  type __MkA_Props_0 = {};",
  "  type __U<T> = T;",
  "  })();",
  "}",
  "// Invoke setup to verify types",
  "__setup();",
  "// ========== Default Export ==========",
  "type $ComponentConstructor = new () => {};",
  "",
].join("\n");
const HELPERS = [
  "// Shared helpers",
  "export type $ComponentInstanceFor<T> = T & {};",
  "export type $ReplaceComponentProps<T> = Pick<T, keyof T>;",
  "",
].join("\n");

function scratch(t: TestContext): string {
  const dir = realpathSync(mkdtempSync(join(tmpdir(), "typecheck-cost-")));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}

const byte = (text: string, needle: string) => {
  const index = text.indexOf(needle);
  assert.notEqual(index, -1, needle);
  return Buffer.byteLength(text.slice(0, index));
};
/** types.json's 1-based full start of the line holding `needle`: the end of the line before. */
const fullStart = (text: string, needle: string) => {
  const lines = text.slice(0, text.indexOf(needle)).split("\n");
  const previous = lines[lines.length - 2] ?? "";
  return { line: lines.length - 1, character: previous.length + 1 };
};

void test("generated statements map to their emitter family and section", () => {
  const text = sourceText(Buffer.from(GENERATED));
  const sections = sectionStarts(text);
  const family = (needle: string) => siteFamily(text, sections, byte(GENERATED, needle));
  assert.deepEqual(family("import { ref }"), { section: "module", family: "module" });
  assert.deepEqual(family("const count"), { section: "setup-user", family: "setup-user" });
  assert.deepEqual(family("__vize_prop_check_0_to"), { section: "template", family: "prop-check" });
  assert.deepEqual(family("type __MkA_22_click_args"), { section: "template", family: "listener" });
  assert.deepEqual(family("type __MkA_Props_0"), { section: "template", family: "contract-alias" });
  assert.deepEqual(family("type __U<T>"), { section: "template", family: "template-context" });
  assert.deepEqual(family("__setup();"), {
    section: "component-surface",
    family: "component-surface",
  });
  assert.deepEqual(family("type $ComponentConstructor"), {
    section: "component-export",
    family: "component-constructor",
  });
  // A full start in the previous line's trivia classifies the statement after it.
  assert.deepEqual(siteFamily(text, sections, byte(GENERATED, "type __MkA_22") - 3), {
    section: "template",
    family: "listener",
  });
  assert.deepEqual(localAlias(text, byte(GENERATED, "type __U<T>")), {
    name: "__U",
    generic: true,
  });
  assert.deepEqual(localAlias(text, byte(GENERATED, "type __MkA_Props_0")), {
    name: "__MkA_Props_0",
    generic: false,
  });
  assert.equal(localAlias(text, byte(GENERATED, "const count")), null);
});

void test("per-usage alias names fold to their emitter shape", () => {
  assert.equal(normalizeAlias("__MkA_22_mousedown_args"), "__C_N_*_args");
  assert.equal(normalizeAlias("__MkButton_22__listener_args"), "__C_N__listener_args");
  assert.equal(normalizeAlias("__MkA_Props_0"), "__C_Props_N");
  assert.equal(normalizeAlias("__MkInput_FallthroughValue_3"), "__C_FallthroughValue_N");
  assert.equal(normalizeAlias("__MkSwitch_4_prop_modelValue"), "__C_N_prop_*");
  assert.equal(normalizeAlias("__MkA_0_emit_resolver"), "__C_N_emit_resolver");
  assert.equal(normalizeAlias("__MkButton_slot_contract"), "__C_slot_contract");
  assert.equal(normalizeAlias("__U2"), "__UN");
});

void test("compact generated names fold like the names they replace", () => {
  const compact = [
    "  // ========== Template Scope (inherits from setup) ==========",
    "  type __VProps0 = {};",
    "  type __VComp0 = typeof MkA;",
    "  type __VCheck0 = $PropChecker<__VComp0, __VProps0, {}>;",
    "  type __VProp0$to = string;",
    "  type __VModel0$value = string;",
    "  (undefined as unknown as __VCheck0)({",
    "",
  ].join("\n");
  const text = sourceText(Buffer.from(compact));
  const sections = sectionStarts(text);
  const family = (needle: string) => siteFamily(text, sections, byte(compact, needle)).family;
  assert.equal(family("type __VProps0"), "contract-alias");
  assert.equal(family("type __VComp0"), "contract-alias");
  assert.equal(family("type __VCheck0"), "contract-alias");
  assert.equal(family("type __VProp0$to"), "contract-alias");
  assert.equal(family("type __VModel0$value"), "model");
  assert.equal(family("(undefined as unknown as __VCheck0)"), "props-call");
  assert.equal(normalizeAlias("__VProps0"), normalizeAlias("__MkA_Props_0"));
  assert.equal(normalizeAlias("__VComp12"), "__C_Component_N");
  assert.equal(normalizeAlias("__VValueProps3"), "__C_ValueProps_N");
  assert.equal(normalizeAlias("__VCheckProps3"), "__C_CheckProps_N");
  assert.equal(normalizeAlias("__VCheckTail3"), "__C_CheckTail_N");
  assert.equal(normalizeAlias("__VCheck3"), "__C_Check_N");
  assert.equal(normalizeAlias("__VProp4$model_value"), "__C_N_prop_*");
  assert.equal(normalizeAlias("__VModel4$model_value"), "__C_N_model_*");
  assert.equal(normalizeAlias("__MkSwitch_4_model_model_value"), "__C_N_model_*");
});

void test("helper positions resolve to the enclosing helper declaration", () => {
  const text = sourceText(Buffer.from(HELPERS));
  const declarations = helperDeclarations(text);
  assert.deepEqual(declarations, [
    [1, "$ComponentInstanceFor"],
    [2, "$ReplaceComponentProps"],
  ]);
  assert.equal(enclosingDeclaration(declarations, 0), null);
  assert.equal(enclosingDeclaration(declarations, 2), "$ReplaceComponentProps");
});

void test("1 ms samples take every frame spanning a tick, B/E pairs per thread", () => {
  const frames = checkerFrames([
    {
      ph: "B",
      name: "checkSourceFile",
      ts: 500,
      pid: 1,
      tid: 2,
      args: { checkerId: 0, path: "/a.vue" },
    },
    {
      ph: "B",
      name: "checkSourceFile",
      ts: 400,
      pid: 1,
      tid: 3,
      args: { checkerId: 1, path: "/b.vue" },
    },
    {
      ph: "X",
      name: "checkExpression",
      ts: 900,
      dur: 200,
      pid: 1,
      tid: 2,
      args: { checkerId: 0, path: "/a.vue", pos: 1, end: 2 },
    },
    {
      ph: "B",
      name: "getVariancesWorker",
      ts: 1200,
      pid: 1,
      tid: 2,
      args: { checkerId: 0, id: 7 },
    },
    {
      ph: "B",
      name: "getVariancesWorker",
      ts: 1300,
      pid: 1,
      tid: 2,
      args: { checkerId: 0, id: 8 },
    },
    {
      ph: "E",
      name: "getVariancesWorker",
      ts: 1400,
      pid: 1,
      tid: 2,
      args: { checkerId: 0, id: 8 },
    },
    {
      ph: "E",
      name: "getVariancesWorker",
      ts: 2100,
      pid: 1,
      tid: 2,
      args: { checkerId: 0, id: 7 },
    },
    {
      ph: "E",
      name: "checkSourceFile",
      ts: 2500,
      pid: 1,
      tid: 2,
      args: { checkerId: 0, path: "/a.vue" },
    },
    {
      ph: "E",
      name: "checkSourceFile",
      ts: 1000,
      pid: 1,
      tid: 3,
      args: { checkerId: 1, path: "/b.vue" },
    },
  ]);
  assert.deepEqual(
    [...frames.keys()].sort((a, b) => a - b),
    [0, 1],
  );
  const stacks = tickStacks(frames.get(0) ?? []);
  assert.deepEqual(
    [...stacks].map(([tick, stack]) => [tick, stack.map((frame) => frame.name)]),
    [
      [1, ["checkSourceFile", "checkExpression"]],
      [2, ["checkSourceFile", "getVariancesWorker"]],
    ],
  );
  assert.deepEqual([...tickStacks(frames.get(1) ?? []).keys()], [1]);
  assert.deepEqual(
    varianceSpans(frames.get(0) ?? []).map((span) => [span.args.id, span.end - span.start]),
    [[7, 900]],
  );
});

type Types = Array<Record<string, unknown>>;

function lane(t: TestContext) {
  const dir = scratch(t);
  const root = join(dir, "project");
  const helper = join(root, "node_modules", "vize", "virtual-ts.d.ts");
  const sfc = join(root, "src", "App.vue");
  const generated = join(dir, "generated");
  const traceDir = join(dir, "trace");
  mkdirSync(join(root, "node_modules", "vize"), { recursive: true });
  mkdirSync(join(root, "src"), { recursive: true });
  mkdirSync(join(generated, "src"), { recursive: true });
  mkdirSync(traceDir);
  writeFileSync(helper, HELPERS);
  writeFileSync(sfc, '<template><MkA to="/" /></template>\n');
  writeFileSync(join(generated, "src", "App.vue.tsx"), GENERATED);
  const at = (path: string, text: string, needle: string) => ({
    path: path.toLowerCase(),
    start: fullStart(text, needle),
  });
  const lib = {
    path: join(dir, "typescript", "lib", "lib.es5.d.ts"),
    start: { line: 1, character: 1 },
  };
  const types: Types = [
    { id: 1, intrinsicName: "any", flags: ["Any"] },
    { id: 2, flags: ["Intersection"], intersectionTypes: [1] },
    {
      id: 3,
      symbolName: "Pick",
      aliasTypeArguments: [2],
      firstDeclaration: lib,
      flags: ["Object"],
    },
    {
      id: 4,
      symbolName: "$ReplaceComponentProps",
      aliasTypeArguments: [3],
      firstDeclaration: at(helper, HELPERS, "export type $ReplaceComponentProps"),
      flags: ["Object"],
      display: "$ReplaceComponentProps<{}>",
    },
    {
      id: 5,
      symbolName: "$ComponentConstructor",
      firstDeclaration: at(sfc, GENERATED, "type $ComponentConstructor"),
      flags: ["Object"],
    },
    {
      id: 6,
      symbolName: "__MkA_22_click_args",
      firstDeclaration: at(sfc, GENERATED, "type __MkA_22_click_args"),
      flags: ["Object"],
    },
    {
      id: 7,
      symbolName: "__U",
      aliasTypeArguments: [1],
      firstDeclaration: at(sfc, GENERATED, "type __U<T>"),
      flags: ["Object"],
    },
    {
      id: 8,
      symbolName: "__type",
      firstDeclaration: at(sfc, GENERATED, "const __vize_prop_check_0_to"),
      flags: ["Object"],
      display: "{ a: 1 }",
    },
    {
      id: 9,
      symbolName: "__type",
      firstDeclaration: at(sfc, GENERATED, "const __vize_prop_check_0_to"),
      flags: ["Object"],
      display: "{ b: 1 }",
    },
    { id: 10, flags: ["Union"], unionTypes: [8, 9] },
    {
      id: 11,
      symbolName: "__MkA_Props_0",
      intersectionTypes: [10],
      firstDeclaration: at(sfc, GENERATED, "type __MkA_Props_0"),
      flags: ["Intersection"],
    },
    { id: 12, flags: ["StringLiteral"], display: '"x"' },
  ];
  writeFileSync(
    join(traceDir, "types_0.json"),
    `[${types.map((type) => JSON.stringify(type)).join(",\n")}]\n`,
  );
  const args = (extra: Record<string, unknown>) => ({ checkerId: 0, ...extra });
  const container = byte(GENERATED, "(function __template");
  const check = byte(GENERATED, "  const __vize_prop_check_0_to") - 1;
  const events = [
    { ph: "B", name: "checkSourceFile", ts: 500, pid: 1, tid: 2, args: args({ path: sfc }) },
    {
      ph: "X",
      name: "checkDeferredNode",
      ts: 1500,
      dur: 6000,
      pid: 1,
      tid: 2,
      args: args({ path: sfc, pos: container - 1, end: byte(GENERATED, ")();") }),
    },
    {
      ph: "X",
      name: "checkVariableDeclaration",
      ts: 1800,
      dur: 1500,
      pid: 1,
      tid: 2,
      args: args({ path: sfc, pos: check, end: byte(GENERATED, "  type __MkA_22") - 1 }),
    },
    { ph: "B", name: "getVariancesWorker", ts: 2000, pid: 1, tid: 2, args: args({ id: 4 }) },
    { ph: "E", name: "getVariancesWorker", ts: 2600, pid: 1, tid: 2, args: args({ id: 4 }) },
    {
      ph: "X",
      name: "structuredTypeRelatedTo",
      ts: 4100,
      dur: 1000,
      pid: 1,
      tid: 2,
      args: args({ sourceId: 8, targetId: 9 }),
    },
    { ph: "E", name: "checkSourceFile", ts: 10500, pid: 1, tid: 2, args: args({ path: sfc }) },
  ];
  writeFileSync(join(traceDir, "trace.json"), JSON.stringify(events));
  writeFileSync(
    join(traceDir, "legend.json"),
    JSON.stringify([
      { tracePath: "/elsewhere/trace.json", typesPath: "/elsewhere/types_0.json", checkerId: 0 },
    ]),
  );
  return { root, generated, traceDir, dir };
}

void test("anonymous types count for their creator; lib types for the helper that built them", async (t) => {
  const { traceDir } = lane(t);
  const origin = (record: { id: number; symbolName?: string; firstDeclaration?: unknown }) =>
    record.firstDeclaration ? `${record.id > 3 ? "helper" : "lib"}\t${record.symbolName}` : null;
  const census = await censusTypes(
    join(traceDir, "types_0.json"),
    origin,
    (key) => key.startsWith("helper\t"),
    new Set([2, 10]),
    [
      [8, 9],
      [5, 6],
    ],
  );
  assert.equal(census.total, 12);
  assert.equal(census.created.get("lib\tPick"), 2);
  assert.equal(census.createdFor.get("helper\t$ReplaceComponentProps"), 3);
  assert.equal(census.created.get("(anonymous StringLiteral)"), 1);
  assert.deepEqual(census.described.get(2), {
    key: "lib\tPick",
    owner: "helper\t$ReplaceComponentProps",
    display: "",
  });
  assert.deepEqual([...census.siblings], [["8>9", "helper\t__MkA_Props_0"]]);
});

void test("a traced lane attributes samples, types, variance and union siblings", async (t) => {
  const { root, generated, traceDir } = lane(t);
  const report = await attributeLane({ traceDir, root, generated });
  assert.equal(report.checkers, 1);
  assert.equal(report.checkMs, 10);
  assert.equal(report.sfcMs, 10);
  assert.equal(report.typesCreated, 12);
  const third = 4 / 3;
  assert.deepEqual(report.families, {
    "setup-glue (unframed)": 1,
    "template/prop-check": 2,
    "template/listener (unframed)": third,
    "template/contract-alias (unframed)": third,
    "template/template-context (unframed)": third,
    "component-export/component-constructor (unframed)": 3,
  });
  assert.deepEqual(report.keys["helper\t$ReplaceComponentProps"], {
    created: 1,
    createdFor: 3,
    inclusiveMs: 0,
    varianceMs: 0.6,
  });
  assert.equal(report.keys["lib\tPick"]?.created, 2);
  assert.equal(report.keys["local-generic\ttemplate/template-context __U"]?.created, 1);
  assert.equal(report.keys["sfc\ttemplate/listener __C_N_*_args"]?.created, 1);
  assert.equal(
    report.keys["sfc\tcomponent-export/component-constructor $ComponentConstructor"]?.created,
    1,
  );
  assert.equal(report.keys["sfc\ttemplate/prop-check"]?.inclusiveMs, 1);
  assert.deepEqual(report.variance, [
    {
      key: "helper\t$ReplaceComponentProps",
      ms: 0.6,
      runs: 1,
      checkers: 1,
      example: "$ReplaceComponentProps<{}>",
    },
  ]);
  assert.deepEqual(report.siblings, { "sfc\ttemplate/contract-alias __C_Props_N": 1 });
  assert.deepEqual(report.unowned, [['StringLiteral: "x"', 1]]);
  assert.deepEqual(
    report.relations.map((row) => [row.key, row.ms, row.checkers]),
    [["sfc\ttemplate/prop-check → sfc\ttemplate/prop-check", 1, 1]],
  );
  assert.deepEqual(report.files, [
    { file: "src/App.vue", ms: 10, family: "component-export/component-constructor (unframed)" },
  ]);

  const head = {
    ...report,
    typesCreated: 10,
    keys: {
      ...report.keys,
      "helper\t$ReplaceComponentProps": {
        created: 1,
        createdFor: 1,
        inclusiveMs: 0,
        varianceMs: 0,
      },
    },
  };
  const markdown = renderAttribution("fixture", head, report);
  assert.match(markdown, /^### Check-cost attribution: fixture$/mu);
  assert.match(markdown, /10 types created \(base 12, -2\)/u);
  assert.match(
    markdown,
    /\| `\$ReplaceComponentProps` \| shared helper \| 1 \| -2 \| 1 \| 0 \| 0 \| 0 \|/u,
  );
  assert.match(
    markdown,
    /\| `template\/template-context __U` \| local generic alias \| 1 \| 0 \| 1 \|/u,
  );
});

void test("the corpus pair layout yields one traced Vize lane per corpus and build", (t) => {
  const dir = scratch(t);
  for (const name of [
    "misskey-tsc-vize-base",
    "misskey-tsc-vize-head",
    "misskey-tsc-vue",
    "n8n-tsrs-vize-head",
  ]) {
    mkdirSync(join(dir, "work", "traces", name), { recursive: true });
  }
  assert.deepEqual(pairLanes(join(dir, "work"), join(dir, "out")), [
    {
      corpus: "misskey",
      role: "base",
      traceDir: join(dir, "work", "traces", "misskey-tsc-vize-base"),
      root: join(dir, "work", "misskey-base"),
      generated: join(dir, "out", "generated", "misskey-tsc-vize-base"),
    },
    {
      corpus: "misskey",
      role: "head",
      traceDir: join(dir, "work", "traces", "misskey-tsc-vize-head"),
      root: join(dir, "work", "misskey-head"),
      generated: join(dir, "out", "generated", "misskey-tsc-vize-head"),
    },
  ]);
});
