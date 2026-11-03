import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import {
  completionLabels,
  lspLocations,
  workspaceEditLocations,
} from "../../tools/benchmarks/scripts/lsp-vs-volar-editors.ts";
import {
  curveMarkdown,
  fitLine,
  growth,
  planJobs,
  sizeCurve,
} from "../../tools/benchmarks/scripts/lsp-vs-volar-curve.ts";
import {
  FIXTURES,
  SCRIPT_BROKEN,
  TEMPLATE_CLEAN,
  catalogVersion,
  commonDirectory,
  plantProbes,
  projectDigest,
  type RequestSpec,
} from "../../tools/benchmarks/scripts/lsp-vs-volar-fixtures.ts";
import {
  monorepoProject,
  scaleProject,
} from "../../tools/benchmarks/scripts/lsp-vs-volar-scale.ts";
import { percentile, psTimeSeconds } from "../../tools/benchmarks/scripts/lsp-vs-volar-metrics.ts";
import { markdown, summarize } from "../../tools/benchmarks/scripts/lsp-vs-volar-report.ts";
import { judge, type SessionResult } from "../../tools/benchmarks/scripts/lsp-vs-volar-session.ts";
import { OpenDocument } from "../../tools/benchmarks/scripts/lsp-vs-volar-text.ts";
import { FrameReader } from "../../tools/benchmarks/scripts/lsp-vs-volar-wire.ts";

test("frames split across chunks and tsserver's counted newline both decode", () => {
  const messages: unknown[] = [];
  const reader = new FrameReader((message) => messages.push(message));
  const lsp = JSON.stringify({ id: 1, result: "ü" });
  const frame = `Content-Length: ${Buffer.byteLength(lsp)}\r\n\r\n${lsp}`;
  const ts = `${JSON.stringify({ seq: 0, type: "event", event: "x" })}\n`;
  const bytes = Buffer.from(`${frame}Content-Length: ${Buffer.byteLength(ts)}\r\n\r\n${ts}`);
  for (let at = 0; at < bytes.length; at += 7) reader.push(bytes.subarray(at, at + 7));
  assert.deepEqual(messages, [
    { id: 1, result: "ü" },
    { seq: 0, type: "event", event: "x" },
  ]);
});

test("anchors are unique text, and edits keep UTF-16 positions in step", () => {
  const doc = new OpenDocument("/a.vue", "file:///a.vue", "<template>\n  {{ ü.x }}\n</template>\n");
  assert.deepEqual(doc.resolve({ find: "ü.x", at: 2 }), { line: 1, character: 7 });
  assert.throws(() => doc.resolve({ find: "template" }), /not unique/);
  assert.throws(() => doc.resolve({ find: "missing" }), /not found/);
  const edit = doc.replace("ü.x", "ü.yy");
  assert.deepEqual(edit.range, {
    start: { line: 1, character: 5 },
    end: { line: 1, character: 8 },
  });
  assert.equal(doc.version, 2);
  assert.equal(doc.text, "<template>\n  {{ ü.yy }}\n</template>\n");
  assert.equal(doc.offsetAt(doc.positionAt(17)), 17);
});

test("probes plant once per fixture and refuse ambiguous anchors", () => {
  const [spec] = FIXTURES;
  const source = `${spec.scriptProbeAfter}const a = 1;\n</script>\n<template>\n${spec.templateProbeAfter}</template>\n`;
  const planted = plantProbes(spec, source);
  assert.equal(planted.split(SCRIPT_BROKEN).length, 2);
  assert.equal(planted.split(TEMPLATE_CLEAN).length, 2);
  assert.throws(() => plantProbes(spec, `${source}${spec.scriptProbeAfter}`), /exactly once/);
  for (const fixture of FIXTURES) {
    assert.equal(
      new Set(fixture.requests.map((request) => `${request.kind}:${request.name}`)).size,
      fixture.requests.length,
    );
  }
});

test("a fast empty or wrong answer is a failure, not a sample", () => {
  const hover: RequestSpec = { kind: "hover", name: "h", anchor: { find: "x" }, expect: ["x:"] };
  assert.equal(judge(hover, "", "/w").ok, false);
  assert.equal(judge(hover, "const x: number", "/w").ok, true);
  const definition: RequestSpec = {
    kind: "definition",
    name: "d",
    anchor: { find: "x" },
    expectPath: "src/B.vue",
  };
  assert.equal(
    judge(definition, [{ path: "/w/src/A.vue", line: 0, character: 0 }], "/w").ok,
    false,
  );
  assert.equal(judge(definition, [{ path: "/w/src/B.vue", line: 0, character: 0 }], "/w").ok, true);
  const references: RequestSpec = { kind: "references", name: "r", anchor: { find: "x" }, min: 2 };
  const same = { path: "/w/a", line: 1, character: 2 };
  assert.equal(
    judge(references, [same, { ...same }], "/w").ok,
    false,
    "duplicates from two providers count once",
  );
  assert.equal(judge(references, [same, { ...same, character: 9 }], "/w").ok, true);
});

test("protocol answers normalize across both editors' shapes", () => {
  assert.deepEqual(
    completionLabels({
      items: [
        { label: "a" },
        { label: { label: "b" } },
        { label: "channel?", filterText: "channel", insertText: "channel" },
      ],
    }),
    ["a", "b", "channel"],
  );
  const range = { start: { line: 3, character: 4 }, end: { line: 3, character: 5 } };
  assert.deepEqual(
    lspLocations([
      { targetUri: "file:///w/a.ts", targetRange: range, targetSelectionRange: range },
    ]),
    [{ path: "/w/a.ts", line: 3, character: 4 }],
  );
  assert.deepEqual(
    workspaceEditLocations({
      documentChanges: [
        { textDocument: { uri: "file:///w/b.vue" }, edits: [{ range, newText: "y" }] },
      ],
    }),
    [{ path: "/w/b.vue", line: 3, character: 4 }],
  );
});

test("statistics use nearest rank and ps CPU time parses every width", () => {
  assert.equal(percentile([5, 1, 3, 2, 4], 50), 3);
  assert.equal(percentile([5, 1, 3, 2, 4], 99), 5);
  assert.equal(percentile([], 50), null);
  assert.equal(psTimeSeconds("0:01.50"), 1.5);
  assert.equal(psTimeSeconds("1-02:03:04"), 93784);
});

function session(editor: string, editMs: number, ok: boolean): SessionResult {
  const reading = { cpuSeconds: 1, rssBytes: 1048576, pssBytes: null, processes: 2 };
  return {
    editor,
    kind: editor.startsWith("vize") ? "vize" : "volar",
    round: 1,
    cold: { initializeMs: 10, openToDiagnosticsMs: 20, totalMs: 30, ok: true, reading },
    warm: { wallMs: 1, cpuSeconds: 2, peakRssBytes: 2097152, peakPssBytes: null, end: reading },
    idle: { ms: 1, cpuSeconds: 0, end: reading },
    edits: [{ cycle: 1, probe: "script", phase: "break", ms: editMs, ok, cpuSeconds: 0.1 }],
    requests: [{ cycle: 1, kind: "hover", name: "h", ms: 3, ok: true, size: 4 }],
    answers: { "hover: h": "a | b" },
    series: [],
    failures: ok ? [] : ["cycle 1: script break diagnostics never converged"],
  };
}

test("reports exclude failed samples from latency and say so", () => {
  const sessions = [session("vize head", 5, true), session("volar 3", 1, false)];
  const [vize, volar] = summarize(sessions);
  assert.equal(vize.diagnostics.script.p50, 5);
  assert.equal(volar.diagnostics.script.p50, null);
  assert.equal(volar.diagnostics.script.ok, 0);
  const text = markdown("fixture", [vize, volar], sessions, { head: "abc" });
  assert.match(
    text,
    /\| diagnostics after script edit \| 5\.0 \/ 5\.0 \(1\) \| – \/ – \(\*\*0\/1 ok\*\*\) \|/,
  );
  assert.match(text, /a \\\| b/);
  assert.match(text, /- volar 3, round 1: cycle 1/);
});

test("the workflow keeps evidence after failure and measures the checked-out source", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/lsp-vs-volar.yml", import.meta.url),
    "utf8",
  );
  const upload = workflow
    .split(/^      - /m)
    .find((step) => step.startsWith("uses: actions/upload-artifact@"));
  assert.match(upload ?? "", /^        if: always\(\)$/m);
  assert.match(workflow, /--meta "head=\$\(git rev-parse HEAD\)"/);
  // Versions come from the workspace catalog, not from the workflow.
  assert.doesNotMatch(workflow, /@vue\/language-server@\d/);
});

const VUE = "3.5.42";
const scale = new Map([1000, 5000, 20000].map((sfcs) => [sfcs, scaleProject(sfcs, VUE)]));
const spec = (id: string) => FIXTURES.find((fixture) => fixture.id === id)!;

/** Relative imports only: each scale project is one package. */
function imports(files: Map<string, string>, from: string): string[] {
  const dir = from.split("/").slice(0, -1);
  const found: string[] = [];
  for (const [, specifier] of files.get(from)!.matchAll(/(?:from|import)\s*\(?"(\.[^"]+)"/g)) {
    const parts = [...dir];
    for (const part of specifier.split("/")) {
      if (part === "..") parts.pop();
      else if (part !== ".") parts.push(part);
    }
    const base = parts.join("/");
    const target = [base, `${base}.ts`, `${base}/index.ts`].find((path) => files.has(path));
    assert.ok(target, `${from} imports missing ${specifier}`);
    found.push(target);
  }
  return found;
}

function neighbourhood(files: Map<string, string>, focus: string) {
  const deps = new Map([...files.keys()].map((path) => [path, imports(files, path)]));
  const depth = new Map<string, number>();
  const longest = (path: string): number =>
    depth.get(path) ?? depth.set(path, 1 + Math.max(0, ...deps.get(path)!.map(longest))).get(path)!;
  const reach = (start: string, next: (path: string) => string[]) => {
    const seen = new Set<string>();
    const stack = [start];
    while (stack.length)
      for (const path of next(stack.pop()!))
        if (!seen.has(path) && seen.add(path)) stack.push(path);
    return seen;
  };
  const dependents = reach(focus, (path) =>
    [...deps].filter(([, to]) => to.includes(path)).map(([from]) => from),
  );
  const below = reach(focus, (path) => deps.get(path)!);
  const direct = [...deps].filter(([, to]) => to.includes(focus)).length;
  return {
    depth: longest(focus) - 1,
    direct,
    dependents,
    closure: new Set([focus, ...below, ...dependents]),
  };
}

test("scale projects are deterministic and grow only away from the edited SFC", () => {
  assert.equal(projectDigest(scaleProject(1000, VUE)), projectDigest(scale.get(1000)!));
  assert.notEqual(projectDigest(scaleProject(1000, "3.5.0")), projectDigest(scale.get(1000)!));
  const shapes = [...scale].map(([sfcs, files]) => {
    assert.equal([...files.keys()].filter((path) => path.endsWith(".vue")).length, sfcs);
    const id = `scale-${sfcs / 1000}k`;
    assert.equal(spec(id).curve?.sfcs, sfcs);
    const shape = neighbourhood(files, spec(id).file);
    const bytes = [...shape.closure, "src/components.d.ts", "tsconfig.json"].map(
      (path) => `${path}\n${files.get(path)}`,
    );
    return { ...shape, bytes: bytes.sort().join("\n") };
  });
  for (const shape of shapes) {
    assert.ok(shape.depth >= 20, `dependency depth ${shape.depth}`);
    assert.ok(shape.dependents.size >= 20 && shape.direct === 9, "the edited SFC sits mid-graph");
    // Same edited SFC, dependencies and dependents byte for byte; only unrelated files grow.
    assert.equal(shape.bytes, shapes[0].bytes);
    assert.equal(shape.dependents.size, shapes[0].dependents.size);
  }
});

test("every generated fixture plants once and resolves each anchor and target", () => {
  const monorepo = monorepoProject(VUE);
  assert.equal(projectDigest(monorepoProject(VUE)), projectDigest(monorepo));
  const projects = [
    ...[...scale].map(([sfcs, files]) => [`scale-${sfcs / 1000}k`, files] as const),
    ["scale-monorepo", monorepo] as const,
  ];
  for (const [id, files] of projects) {
    const fixture = spec(id);
    assert.equal(fixture.source.kind, "generated");
    const doc = new OpenDocument(
      `/${fixture.file}`,
      "file:///a.vue",
      plantProbes(fixture, files.get(fixture.file)!),
    );
    doc.resolve({ find: SCRIPT_BROKEN });
    doc.resolve({ find: TEMPLATE_CLEAN });
    for (const request of fixture.requests) {
      if ("anchor" in request)
        assert.doesNotThrow(() => doc.resolve(request.anchor), `${id} ${request.name}`);
      if (request.kind === "definition")
        assert.ok(files.has(request.expectPath), request.expectPath);
    }
    for (const tsconfig of fixture.tsconfigs) assert.ok(files.has(tsconfig), tsconfig);
  }
});

test("the monorepo joins its packages by references and declares the mapper where .vue lives", () => {
  const files = monorepoProject(VUE);
  const members = [...files.keys()]
    .filter((path) => /^(apps|packages)\/[^/]+\/package\.json$/.test(path))
    .map((path) => path.slice(0, -"/package.json".length));
  assert.ok(members.length >= 20 && members.length <= 50, `${members.length} packages`);
  for (const member of members) {
    const tsconfig = JSON.parse(files.get(`${member}/tsconfig.json`)!) as {
      references: Array<{ path: string }>;
    };
    for (const reference of tsconfig.references) {
      const target = new URL(reference.path, `file:///${member}/`).pathname.slice(1);
      assert.ok(files.has(`${target}/tsconfig.json`), `${member} references ${reference.path}`);
    }
  }
  const withVue = members.filter((member) =>
    [...files.keys()].some((path) => path.startsWith(`${member}/`) && path.endsWith(".vue")),
  );
  assert.deepEqual(
    new Set(spec("scale-monorepo").tsconfigs),
    new Set(withVue.map((member) => `${member}/tsconfig.json`)),
  );
  assert.equal(commonDirectory(["/w/apps/a", "/w/apps/ab", "/w/packages/b"]), "/w");
  assert.equal(commonDirectory(["/w/apps/a"]), "/w/apps/a");
});

test("generated projects pin Vue from the workspace catalog", () => {
  const yaml = readFileSync(new URL("../../pnpm-workspace.yaml", import.meta.url), "utf8");
  assert.match(catalogVersion(yaml, "vue-production", "vue"), /^3\.\d+\.\d+$/);
  const sample =
    'catalogs:\n  a:\n    vue: "1.0.0"\n\n  # b\n  b:\n    "@vue/x": "2.0.0"\n    vue: "3.0.0"\n';
  assert.equal(catalogVersion(sample, "b", "vue"), "3.0.0");
  assert.equal(catalogVersion(sample, "b", "@vue/x"), "2.0.0");
  assert.throws(() => catalogVersion(sample, "a", "@vue/x"), /pins no/);
});

test("a size curve shares one job; other fixtures get their own", () => {
  const ids = ["misskey", "scale-1k", "scale-monorepo", "scale-20k", "scale-5k"];
  assert.deepEqual(planJobs(ids.map(spec)), [
    { name: "misskey", ids: "misskey" },
    { name: "scale", ids: "scale-1k scale-20k scale-5k" },
    { name: "scale-monorepo", ids: "scale-monorepo" },
  ]);
});

test("slopes are least squares with a 95% interval", () => {
  const exact = fitLine([
    [1, 5],
    [5, 13],
    [20, 43],
  ])!;
  assert.ok(Math.abs(exact.slope - 2) < 1e-9 && Math.abs(exact.intercept - 3) < 1e-9);
  assert.equal(exact.halfWidth, 0);
  assert.equal(
    fitLine([
      [1, 5],
      [1, 6],
    ]),
    null,
    "one size has no slope",
  );
  assert.equal(
    fitLine([
      [1, 5],
      [5, 6],
    ])!.halfWidth,
    null,
    "two points leave no residual freedom",
  );
  const noisy = fitLine([
    [1, 10],
    [1, 12],
    [5, 12],
    [5, 10],
    [20, 10],
    [20, 12],
  ])!;
  assert.ok(Math.abs(noisy.slope) < 1e-9 && noisy.halfWidth! > 0);
  const perEdit = { unit: "ms" as const, perEdit: true };
  const sizes = [1, 5, 20];
  const none = [false, false, false];
  assert.equal(growth(perEdit, noisy, sizes, none, [11, 11, 11]), null);
  const rising = fitLine([
    [1, 10],
    [1, 11],
    [5, 18],
    [5, 19],
    [20, 49],
    [20, 51],
  ]);
  assert.equal(growth(perEdit, rising, sizes, none, [10, 18, 50]), "slope above zero");
  assert.equal(growth({ ...perEdit, perEdit: false }, rising, sizes, none, [10, 18, 50]), null);
  // Significant but under the resolution floor: 0.1 ms across the whole range.
  const tiny = fitLine([
    [1, 2],
    [1, 2],
    [5, 2.02],
    [5, 2.02],
    [20, 2.1],
    [20, 2.1],
  ]);
  assert.equal(growth(perEdit, tiny, sizes, none, [2, 2.02, 2.1]), null);
  assert.equal(
    growth(perEdit, null, sizes, [false, false, true], [10, 18, null]),
    "stops answering at 20k SFCs",
  );
  assert.equal(growth(perEdit, null, sizes, [true, true, true], [null, null, null]), null);
});

test("the curve section tabulates every editor per size and flags growth", () => {
  const at = (ms: number[]) => ms.map((value) => session("vize head", value, true));
  const points = [
    { id: "scale-1k", sfcs: 1000, sessions: at([10, 11, 12]) },
    { id: "scale-20k", sfcs: 20000, sessions: at([90, 91, 92]) },
    { id: "scale-5k", sfcs: 5000, sessions: at([26, 27, 28]) },
  ];
  const curve = sizeCurve(points);
  assert.deepEqual(curve.sfcs, [1000, 5000, 20000]);
  const metric = (label: string) => curve.editors[0].metrics.find((item) => item.label === label)!;
  assert.deepEqual(metric("diagnostics after script edit p50 ms").values, [11, 27, 91]);
  assert.equal(metric("diagnostics after script edit p50 ms").grows, "slope above zero");
  assert.equal(metric("hover p50 ms").grows, null);
  assert.equal(metric("cold start ms").grows, null);
  const text = curveMarkdown("scale", ["scale-1k", "scale-5k", "scale-20k"], curve);
  assert.match(
    text,
    /\| metric \| 1,000 SFCs \| 5,000 SFCs \| 20,000 SFCs \| slope per 1k SFCs \| per edit \|/,
  );
  assert.match(
    text,
    /\| diagnostics after script edit p50 ms \| 11 \| 27 \| 91 \| \+4\.2 ± 0\.\d+ ms \| \*\*grows\*\* \(slope above zero\) \|/,
  );
  assert.match(text, /- vize head: diagnostics after script edit p50 ms, \+4\.2/);
});
