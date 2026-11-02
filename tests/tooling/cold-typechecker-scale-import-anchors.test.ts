import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";
import {
  removeOwnedChildAnchor,
  verifyImportAnchorTransition,
  verifyAnchorMaterializedRows,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-import-anchors.ts";
import {
  verifyCandidateCommandPopulation,
  candidateCommandIds,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-law.ts";
const source = 'import Child from "./components/Component1.vue";\n';
const code =
  'import Child from "./components/Component1.vue.ts";\n(function(){\n  // Mark used components as referenced\n  void Child;\n\n  // Reference setup bindings (used in template/CSS v-bind)\n  void (null as unknown as keyof typeof Child); void count; void (null as unknown as keyof typeof State);\n  })();\n';
const next = code.replace("void (null as unknown as keyof typeof Child); ", "");
const cwd = "/owned/project/apps/web",
  file = "src/Control.vue",
  header = "\n=== __vize_helpers.d.ts ===\n",
  helper = "declare const helper: unique symbol;\n";
function packet(text: string, extra = {}) {
  const value = {
    programs: [{ root: ".", files: [file] }],
    files: [{ file, diagnostics: [], virtualTs: text }],
    errorCount: 0,
    opaque: { retained: [0, "雪"] },
    ...extra,
  };
  return {
    status: 0,
    stdout: Buffer.from(JSON.stringify(value, null, 2) + "\n"),
    stderr: Buffer.from(header + helper + `\n=== ${cwd}/${file} ===\n${text}\n`),
  };
}
const inputs = [{ path: file, body: source }];

test("exact source/read-owned Child anchor disappears while all other statements remain", () => {
  assert.deepEqual(removeOwnedChildAnchor(code, source), { code: next, removed: 1 });
  assert.deepEqual(removeOwnedChildAnchor(code, 'import * as UI from "./components";'), {
    code,
    removed: 0,
  });
  const namespace = code.replace("  void Child;", "  void UI.Child;");
  assert.deepEqual(removeOwnedChildAnchor(namespace, source), { code: namespace, removed: 0 });
  const noRead = code.replace("  void Child;\n", "");
  assert.deepEqual(removeOwnedChildAnchor(noRead, source), { code: noRead, removed: 0 });
  const only = code.replace(" void count; void (null as unknown as keyof typeof State);", "");
  assert.equal(
    removeOwnedChildAnchor(only, source).code.includes("Reference setup bindings"),
    false,
  );
});

test("whole JSON and debug reconstruction preserves fields, helper, order and opaque bytes", () => {
  const proof = verifyImportAnchorTransition(packet(code), packet(next), cwd, inputs);
  assert.equal(proof.removedAnchors, 1);
  assert.deepEqual(proof.modules, [{ file, before: code, after: next }]);
  for (const mutant of [
    packet(next + "// drift\n"),
    packet(next, { opaque: { retained: [1, "雪"] } }),
    packet(next, { errorCount: 1 }),
  ])
    assert.throws(() => verifyImportAnchorTransition(packet(code), mutant, cwd, inputs));
  const helperDrift = packet(next);
  helperDrift.stderr = Buffer.from(
    helperDrift.stderr.toString().replace("unique symbol", "number"),
  );
  assert.throws(() => verifyImportAnchorTransition(packet(code), helperDrift, cwd, inputs));
  assert.throws(() => verifyImportAnchorTransition(packet(code), packet(next), cwd, []));
  assert.throws(() =>
    verifyImportAnchorTransition(packet(code), packet(next), cwd, [...inputs, ...inputs]),
  );
});

test("zero actual deletions is explicit ineligible population, never an assumed affected corpus", () => {
  const proof = verifyImportAnchorTransition(packet(code), packet(code), cwd, [
    { path: file, body: "const Child = {};\n" },
  ]);
  assert.equal(proof.removedAnchors, 0);
  assert.deepEqual(proof.modules, []);
  assert.throws(() =>
    verifyImportAnchorTransition(packet(code), packet(next), cwd, [
      { path: file, body: "const Child = {};\n" },
    ]),
  );
});

const digest = (body: string) => createHash("sha256").update(body).digest("hex");
function seal(text: string) {
  const rows = [
    {
      path: "/owned/provider/package.json",
      kind: "file",
      bytes: 2,
      sha256: digest("{}"),
      mtimeNs: "1",
    },
    {
      path: "/owned/cache/src/Control.vue.ts",
      kind: "file",
      bytes: Buffer.byteLength(text),
      sha256: digest(text),
      mtimeNs: "2",
      mode: 420,
    },
  ];
  return {
    roots: ["/owned/project", "/owned/provider", "/owned/cache"],
    rows,
    sha256: digest(JSON.stringify(rows)),
    files: 2,
    bytes: 2 + Buffer.byteLength(text),
  };
}
test("complete native input closure joins exact changed module, refuses provider/path/body/extra row changes", () => {
  const changes = [{ file, before: code, after: next }];
  assert.equal(
    verifyAnchorMaterializedRows(seal(code), seal(next), "/owned/cache", changes).changedModules
      .length,
    1,
  );
  for (const change of [
    (s: ReturnType<typeof seal>) => {
      s.rows[0].sha256 = digest("[]");
    },
    (s: ReturnType<typeof seal>) => {
      s.rows[1].path += ".tsx";
    },
    (s: ReturnType<typeof seal>) => {
      s.rows[1].sha256 = digest(next + "x");
    },
    (s: ReturnType<typeof seal>) => {
      s.rows[1].mode = 511;
    },
  ]) {
    const mutant = seal(next);
    change(mutant);
    mutant.sha256 = digest(JSON.stringify(mutant.rows));
    assert.throws(() => verifyAnchorMaterializedRows(seal(code), mutant, "/owned/cache", changes));
  }
  assert.throws(() => verifyAnchorMaterializedRows(seal(code), seal(next), "/owned/cache", []));
});

test("normal-only cohort is exactly 20 outer commands/eight primaries in the original balanced order", () => {
  const ids = candidateCommandIds.filter(
    (id) => !id.startsWith("allocation-") && !id.startsWith("native-"),
  );
  const proof = verifyCandidateCommandPopulation(ids, true, true);
  assert.equal(proof.plannedOuterCommands, 20);
  assert.equal(proof.primaryCommandsObserved, 8);
  assert.deepEqual(
    ids.filter((id) => id.startsWith("primary-")),
    [
      "primary-1-base",
      "primary-1-head",
      "primary-2-head",
      "primary-2-base",
      "primary-3-head",
      "primary-3-base",
      "primary-4-base",
      "primary-4-head",
    ],
  );
  assert.throws(() => verifyCandidateCommandPopulation(ids.toReversed(), true, true));
  assert.throws(() => verifyCandidateCommandPopulation(ids.slice(0, -1), true, true));
  assert.equal(
    verifyCandidateCommandPopulation([...candidateCommandIds], true).plannedOuterCommands,
    30,
  );
});
