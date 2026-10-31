/** Pure admission/archive controls; these grant no installed-public or runtime credit. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { parse as parseYaml } from "yaml";
import { writeTarGz } from "../../../tools/support/archive/tar.ts";
import {
  cachedArchive,
  checkSources,
  completedProducer,
  lockEntry,
  read,
  request,
  validateArchive,
  type Custody,
} from "../../../tools/support/release/public_jsx_child_values/contract.ts";
import {
  nativeVdomMerge,
  nativeVdomTen,
  qualificationTarget,
  qualifyVdomBrowser,
  qualifyVdomNode,
  targetCustody,
  vaporNine,
} from "../../../tools/support/release/public_jsx_child_values/target.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const environment = {
  GITHUB_ACTIONS: "true",
  GITHUB_REPOSITORY: "ubugeeei-prod/vize",
  GITHUB_REF: "refs/heads/main",
  GITHUB_EVENT_NAME: "workflow_dispatch",
  GITHUB_WORKFLOW_REF:
    "ubugeeei-prod/vize/.github/workflows/public-jsx-child-values.yml@refs/heads/main",
};
const candidate = "4e07a16e9807e4f711a649bbe38759c8a9f61933";
test("only an actually completed successful original Release producer is admitted", () => {
  const facts = {
    id: 123,
    head_sha: candidate,
    head_branch: "main",
    path: ".github/workflows/release.yml",
    event: "workflow_dispatch",
    run_attempt: 1,
    repository: { full_name: "ubugeeei-prod/vize" },
    status: "completed",
    conclusion: "success",
  };
  assert.deepEqual(completedProducer(facts, "123"), facts);
  for (const changed of [
    { status: "queued", conclusion: null },
    { status: "in_progress", conclusion: null },
    { conclusion: "failure" },
    { conclusion: "cancelled" },
    { conclusion: "neutral" },
    { id: 124 },
    { head_branch: "feature" },
    { head_sha: candidate.slice(0, 12) },
    { run_attempt: 0 },
  ])
    assert.throws(() => completedProducer({ ...facts, ...changed }, "123"));
});
test("public admission requires full authority and actual main-only dispatch context", () => {
  request("0.999.0", candidate, "123", environment);
  for (const changed of [
    { GITHUB_ACTIONS: "false" },
    { GITHUB_REPOSITORY: "another/vize" },
    { GITHUB_REF: "refs/heads/feature" },
    { GITHUB_EVENT_NAME: "pull_request" },
    { GITHUB_WORKFLOW_REF: environment.GITHUB_WORKFLOW_REF.replace("main", "feature") },
    { NODE_PATH: "/tmp/provider" },
    { NAPI_RS_FORCE_WASI: "1" },
    { VIZE_NATIVE_BINDING_PATH: "/tmp/provider.node" },
  ])
    assert.throws(() => request("0.999.0", candidate, "123", { ...environment, ...changed }));
  for (const [version, head, run] of [
    ["latest", candidate, "123"],
    ["0.999.0", candidate.slice(0, 12), "123"],
    ["0.999.0", candidate, "0"],
  ])
    assert.throws(() => request(version!, head!, run!, environment));
});
test("the copied original observers and full ten-phase inputs retain merged byte custody", () => {
  const custody = read<Custody>(
    path.join(root, "tools/support/release/public_jsx_child_values/source-custody.json"),
  );
  assert.equal(custody.merge, candidate);
  assert.equal(Object.keys(custody.pins).length, 7);
  checkSources(root, custody);
  assert.deepEqual(custody.referenceDependencies, {
    "@babel/core": "7.29.0",
    "@vue/babel-plugin-jsx": "2.0.1",
    "@playwright/test": "1.62.1",
    "happy-dom": "21.0.4",
    vue: "3.6.0-rc.10",
    "vue-jsx-vapor": "3.2.26",
  });
  const original = read<{ expectedText: string[]; source: string }>(
    path.join(root, "tests/_fixtures/differential/compiler/jsx-child-values-6890/original.json"),
  );
  assert.equal(original.source, "const App = () => <main>{state.value}</main>;");
  assert.deepEqual(original.expectedText, [
    "0",
    "",
    "NaN",
    "",
    "",
    "",
    "",
    "0NaN",
    "innerleaf",
    "0",
  ]);
  assert.throws(() =>
    checkSources(root, {
      ...custody,
      pins: { ...custody.pins, "tests/tooling/jsx-vapor-oracle/child-values.ts": "0".repeat(64) },
    }),
  );
});
test("npm source authority refuses wrong versions, tarballs and non-sha512 integrities", () => {
  const name = "@vizejs/native",
    version = "0.999.0";
  const row = {
    version,
    resolved: "https://registry.npmjs.org/@vizejs/native/-/native-0.999.0.tgz",
    integrity: "sha512-" + Buffer.alloc(64).toString("base64"),
  };
  assert.deepEqual(lockEntry(row, name, version), row);
  for (const changed of [
    { version: "0.998.0" },
    { integrity: "sha1-test" },
    { resolved: row.resolved.replace("registry.npmjs.org", "example.com") },
    { resolved: row.resolved.replace("native-0.999.0", "other-0.999.0") },
    { resolved: row.resolved + "?source=other" },
  ])
    assert.throws(() => lockEntry({ ...row, ...changed }, name, version));
});
test("the npm cache content must actually match its complete sha512 integrity", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "jsx-sri-control-"));
  try {
    const bytes = Buffer.from("inert archive custody control");
    const digest = createHash("sha512").update(bytes).digest();
    const hex = digest.toString("hex");
    const file = path.join(
      temporary,
      "_cacache/content-v2/sha512",
      hex.slice(0, 2),
      hex.slice(2, 4),
      hex.slice(4),
    );
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, bytes);
    const integrity = "sha512-" + digest.toString("base64");
    assert.deepEqual(cachedArchive(temporary, integrity), bytes);
    fs.appendFileSync(file, "altered");
    assert.throws(() => cachedArchive(temporary, integrity));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
test("whole package archive payloads reject installed byte changes, traversal and links", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "jsx-archive-control-"));
  try {
    const manifest = Buffer.from('{"name":"inert-control","version":"1.0.0"}');
    const code = Buffer.from("export const control = 1;\n");
    fs.writeFileSync(path.join(temporary, "package.json"), manifest);
    fs.writeFileSync(path.join(temporary, "index.js"), code);
    const entries = [
      { name: "package/package.json", data: manifest },
      { name: "package/index.js", data: code },
    ];
    const archive = writeTarGz([{ name: "package", typeflag: "5" }, ...entries]);
    assert.equal(validateArchive(archive, temporary).length, 2);
    for (const extra of [
      { name: "package/../escaped", data: code },
      { name: "package/link", typeflag: "2", linkname: "/tmp/external" },
      entries[1]!,
    ])
      assert.throws(() => validateArchive(writeTarGz([...entries, extra]), temporary));
    fs.appendFileSync(path.join(temporary, "index.js"), "changed");
    assert.throws(() => validateArchive(archive, temporary));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
test("PR controls cannot install or execute the main-only public runtime job", () => {
  const workflow = parseYaml(
    fs.readFileSync(path.join(root, ".github/workflows/public-jsx-child-values.yml"), "utf8"),
  );
  assert.equal(workflow.jobs.controls.if, "github.event_name == 'pull_request'");
  const installed = workflow.jobs["installed-public"];
  assert.equal(
    installed.if,
    "github.event_name == 'workflow_dispatch' && github.repository == 'ubugeeei-prod/vize' && github.ref == 'refs/heads/main'",
  );
  assert.equal(installed.env.PUBLIC_JSX_TARGET, "${{ inputs.target }}");
  assert.deepEqual(workflow.on.workflow_dispatch.inputs.target, {
    description:
      "Explicit source-bound qualification target; older evidence keeps its original scope",
    required: true,
    type: "choice",
    default: vaporNine,
    options: [vaporNine, nativeVdomTen],
  });
  const upload = installed.steps.find((step: { uses?: string }) =>
    step.uses?.startsWith("actions/upload-artifact@"),
  );
  assert.equal(upload.if, "always()");
  assert.equal(upload.with["if-no-files-found"], "error");
});

test("the VDOM target is explicit, contains the actual repair merge and retains every original pin", () => {
  assert.equal(qualificationTarget(undefined), vaporNine);
  assert.equal(qualificationTarget(vaporNine), vaporNine);
  assert.equal(qualificationTarget(nativeVdomTen), nativeVdomTen);
  for (const value of ["", "latest", "0.625.0", "native-vdom-ten-v2"]) {
    assert.throws(() => qualificationTarget(value));
  }
  const original = targetCustody(vaporNine),
    vdom = targetCustody(nativeVdomTen);
  assert.equal(original.merge, candidate);
  assert.equal(vdom.merge, nativeVdomMerge);
  assert.equal(nativeVdomMerge, "f233823358c1242013b51e53680ce257253c84fc");
  assert.notEqual(original.merge, vdom.merge);
  assert.deepEqual(vdom.referenceDependencies, original.referenceDependencies);
  assert.equal(Object.keys(original.pins).length, 7);
  assert.equal(Object.keys(vdom.pins).length, 8);
  for (const [name, digest] of Object.entries(original.pins)) assert.equal(vdom.pins[name], digest);
  checkSources(root, vdom);
});

// Inert admission fixtures only: these controls execute no compiler, Vue or browser.
const original = read<{ expectedText: string[]; expectedElementIdentities: number[][] }>(
  path.join(root, "tests/_fixtures/differential/compiler/jsx-child-values-6890/original.json"),
);
const inertTrace = () => ({
  phases: original.expectedText.map((text, index) => ({
    dom: [{ tag: "inert-test-only", attributes: {}, children: [text] }],
    text,
    elementIdentities: original.expectedElementIdentities[index],
    warnings: [],
    reads: index + 1,
  })),
  readsAfterUnmount: 10,
  completeUnmount: true,
});
const inertNode = () => ({
  routes: { "public-babel": inertTrace(), "source-vdom": inertTrace() },
  receipt: {
    vdomMatchesBabel: Array(10).fill(true),
    qualifiedVaporPhases: 9,
    referenceConflictPhases: 1,
    wholeCorpusQualified: false,
    issue6890Complete: false,
  },
});
const inertBrowser = () => ({
  browserVersion: "inert-test-only",
  routes: Object.fromEntries(
    ["native", "native-shadow", "reference", "reference-shadow"].map((name) => [
      name,
      inertTrace(),
    ]),
  ),
  wholeCorpusQualified: false,
  issue6890Complete: false,
});
test("the distinct VDOM admission refuses the old four-failure vector and any whole-trace divergence", () => {
  qualifyVdomNode(inertNode(), original);
  const historicalVector = [true, true, true, true, true, false, false, false, false, true];
  const old = inertNode();
  old.receipt.vdomMatchesBabel = historicalVector;
  assert.throws(() => qualifyVdomNode(old, original));
  for (const change of [
    (trace: ReturnType<typeof inertTrace>) => {
      trace.phases[7]!.text = "wrong array";
    },
    (trace: ReturnType<typeof inertTrace>) => {
      trace.phases[8]!.dom[0]!.tag = "different complete DOM";
    },
    (trace: ReturnType<typeof inertTrace>) => {
      trace.phases[8]!.elementIdentities = [999];
    },
    (trace: ReturnType<typeof inertTrace>) => {
      trace.phases[9]!.reads++;
    },
    (trace: ReturnType<typeof inertTrace>) => {
      trace.completeUnmount = false;
    },
    (trace: ReturnType<typeof inertTrace>) => {
      trace.readsAfterUnmount++;
    },
  ]) {
    const value = inertNode();
    change(value.routes["source-vdom"]);
    assert.throws(() => qualifyVdomNode(value, original));
  }
  const missing = inertNode();
  delete (missing.routes as Record<string, unknown>)["source-vdom"];
  assert.throws(() => qualifyVdomNode(missing, original));
  const whole = inertNode();
  whole.receipt.issue6890Complete = true;
  assert.throws(() => qualifyVdomNode(whole, original));
});
test("Chromium admission requires all original and shadowed whole pairs and disposal", () => {
  qualifyVdomBrowser(inertBrowser(), original);
  for (const name of ["native", "native-shadow", "reference", "reference-shadow"]) {
    const missing = inertBrowser();
    delete missing.routes[name];
    assert.throws(() => qualifyVdomBrowser(missing, original));
    const changed = inertBrowser();
    changed.routes[name]!.phases[8]!.dom[0]!.attributes = { changed: "yes" };
    assert.throws(() => qualifyVdomBrowser(changed, original));
    const disposal = inertBrowser();
    disposal.routes[name]!.completeUnmount = false;
    assert.throws(() => qualifyVdomBrowser(disposal, original));
  }
  const empty = inertBrowser();
  empty.browserVersion = "";
  assert.throws(() => qualifyVdomBrowser(empty, original));
  const whole = inertBrowser();
  whole.wholeCorpusQualified = true;
  assert.throws(() => qualifyVdomBrowser(whole, original));
});
