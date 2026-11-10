import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  JSX_ENTRY_ADAPTER,
  JSX_ENTRY_RUNNER,
  loadJsxEntryCases,
  newJsxEntryCases,
} from "../differential/compiler-jsx-entry.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const fixtures = "tests/_fixtures/differential/compiler/";
const manifestPath = `${fixtures}manifest.json`;
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const disk = (path: string) => (fs.existsSync(path) ? fs.readFileSync(path) : null);
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

const source = Buffer.from("export const App = () => <p>{label}</p>;\n");
const module = (target: string) => ({
  target,
  code: `/* ${target} */\n`,
  errors: [],
  warnings: [],
  scopedStyles: [],
});
const reference = Buffer.from(`${JSON.stringify([module("vdom"), module("vdom-ssr")], null, 2)}\n`);
const row = () => ({
  id: "compiler/jsx/entry-adapter-probe",
  state: "active",
  adapter: JSX_ENTRY_ADAPTER,
  compat: "native",
  targets: ["vdom", "vdom-ssr"],
  inputs: {
    root: "jsx-entry-adapter-probe",
    files: [{ path: "App.jsx.txt", role: "entry", sha256: hash(source) }],
  },
  reference: {
    path: "jsx-entry-adapter-probe/modules.expected.json",
    sha256: hash(reference),
    source: "tooling probe",
  },
});
type Row = ReturnType<typeof row>;

function files(rows: unknown[], overrides: Record<string, Buffer | null> = {}) {
  const head = new Map<string, Buffer | null>([
    [
      manifestPath,
      Buffer.from(
        JSON.stringify({ ...manifest, jsxEntryCases: [...manifest.jsxEntryCases, ...rows] }),
      ),
    ],
    [`${fixtures}jsx-entry-adapter-probe/App.jsx.txt`, source],
    [`${fixtures}jsx-entry-adapter-probe/modules.expected.json`, reference],
  ]);
  for (const [path, bytes] of Object.entries(overrides)) head.set(path, bytes);
  return (path: string) => (head.has(path) ? head.get(path)! : disk(path));
}

function report(rows: unknown[], paths: string[], overrides = {}) {
  const head = files(rows, overrides);
  return analyzeLegacyFix({
    title: "fix(jsx): reject an unbound list callback",
    paths,
    readFile: (side, path) => (side === "base" ? disk(path) : head(path)),
  }).products;
}

test("the registered JSX entry cases are valid and their runner is in the tree", () => {
  assert(Array.isArray(manifest.jsxEntryCases));
  loadJsxEntryCases(manifest.jsxEntryCases, disk);
  const runner = fs.readFileSync(JSX_ENTRY_RUNNER, "utf8");
  assert(runner.includes(`const ADAPTER: &str = "${JSX_ENTRY_ADAPTER}";`));
  assert(runner.includes('manifest["jsxEntryCases"]'));
});

test("a new JSX entry case registers a fix to the JSX compiler only", () => {
  const jsx = ["crates/vize_atelier_jsx/src/lower/control_flow.rs"];
  assert.deepEqual(report([row()], [...jsx, "crates/vize_atelier_jsx/tests/control_flow.rs"]), [
    { product: "compiler", state: "new-inputs-registered", cases: [row().id] },
  ]);
  assert.equal(report([], jsx)[0]!.state, "new-input-missing");
  const shared = [...jsx, "crates/vize_atelier_vapor/src/generate.rs"];
  assert.equal(report([row()], shared)[0]!.state, "new-input-missing");
});

test("JSX entry cases are closed, pinned and complete per target", () => {
  const jsx = ["crates/vize_atelier_jsx/src/lower/child.rs"];
  const mutations: ((value: Row) => void)[] = [
    (value) => void (value.id = "compiler/jsx/Upper"),
    (value) => void (value.adapter = "compiler-cli-ssr-v1"),
    (value) => void (value.compat = "vue-jsx-vapor"),
    (value) => void (value.targets = ["vdom-ssr", "vdom"]),
    (value) => void (value.targets = ["dom"]),
    (value) => void (value.targets = ["vdom"]),
    (value) => void (value.inputs.root = "entry-adapter-probe"),
    (value) => void (value.inputs.files[0]!.path = "App.vue.txt"),
    (value) => void (value.inputs.files[0]!.sha256 = "0".repeat(64)),
    (value) => void (value.reference.path = "jsx-entry-adapter-probe/module.expected.txt"),
    (value) => void (value.reference.source = ""),
    (value) => void Object.assign(value, { native: null }),
  ];
  for (const mutate of mutations) {
    const mutant = row();
    mutate(mutant);
    assert.equal(report([mutant], jsx)[0]!.state, "invalid-manifest-or-input", mutate.toString());
  }
  for (const bytes of [null, Buffer.from("changed")]) {
    const changed = { [`${fixtures}jsx-entry-adapter-probe/App.jsx.txt`]: bytes };
    assert.equal(report([row()], jsx, changed)[0]!.state, "invalid-manifest-or-input");
  }
  assert.equal(report([row(), row()], jsx)[0]!.state, "invalid-manifest-or-input");
});

test("a registered JSX entry case stays planned and grants no new credit", () => {
  const head = files([row()]);
  assert.deepEqual(newJsxEntryCases([row()], [row()], head), []);
  assert.throws(() => newJsxEntryCases([row()], [], head), /drops planned JSX entry case/);
  assert.throws(() => newJsxEntryCases([], undefined, head), /drops the JSX entry cases/);
});
