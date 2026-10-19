import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { test } from "node:test";
import { runInNewContext } from "node:vm";

const root = new URL("../../", import.meta.url);
const read = (file) => fs.readFileSync(new URL(file, root), "utf8");
const corpus = JSON.parse(read("tests/tooling/fixtures/vuefes-failure-sidecar/corpus.json"));
const original = read("tests/tooling/fixtures/vuefes-failure-sidecar/original-vuefes.spec.ts.txt");
const current = read("tests/app/vrt/vuefes.spec.ts");
const { parse } = createRequire(new URL("../package.json", import.meta.url))("@babel/parser");
const ast = (source) => parse(source, { sourceType: "module", plugins: ["typescript"] }).program;
const originalAst = ast(original);
const currentAst = ast(current);
const callNodes = (node, predicate) => {
  if (!node || typeof node !== "object") return [];
  return [
    ...(predicate(node) ? [node] : []),
    ...Object.values(node).flatMap((value) =>
      Array.isArray(value)
        ? value.flatMap((item) => callNodes(item, predicate))
        : callNodes(value, predicate),
    ),
  ];
};
const semantic = (node) =>
  JSON.parse(
    JSON.stringify(node, (key, value) =>
      [
        "start",
        "end",
        "loc",
        "extra",
        "leadingComments",
        "trailingComments",
        "innerComments",
      ].includes(key)
        ? undefined
        : value,
    ),
  );
const calls = (tree, name) =>
  callNodes(
    tree,
    (node) =>
      node.type === "CallExpression" &&
      node.callee.type === "Identifier" &&
      node.callee.name === name,
  );

void test("original entire corpus witness and protected full38 bodies authenticate without a threshold/route exemption", () => {
  assert.equal(corpus.source, "605429c660b328bb03bec5bbfef15961e59fcaf0");
  // Only the recorded original spec body is authenticated. The other corpus
  // entries describe files that ordinary changes edit, so their bytes at HEAD
  // are not part of the witness.
  const recorded = corpus.files["tests/app/vrt/vuefes.spec.ts"];
  assert.equal(Buffer.byteLength(original), recorded.bytes);
  assert.equal(createHash("sha256").update(original).digest("hex"), recorded.sha256);
  const originalConstants = original.slice(
    original.indexOf("const __dirname"),
    original.indexOf("test.describe("),
  );
  const currentConstants = current.slice(
    current.indexOf("const __dirname"),
    current.indexOf("test.describe("),
  );
  assert.equal(currentConstants, originalConstants);
  assert.equal(
    current.slice(current.indexOf("async function setupPage")),
    original.slice(original.indexOf("async function setupPage")),
  );
  for (const name of ["setupPage", "openRoute", "prepareVuefesPage", "expectVisualParity"]) {
    assert.deepEqual(
      calls(currentAst, name).map(semantic),
      calls(originalAst, name).map(semantic),
      name,
    );
  }
  const contexts = (tree) =>
    callNodes(
      tree,
      (node) =>
        node.type === "CallExpression" &&
        node.callee.type === "MemberExpression" &&
        node.callee.property.name === "newContext",
    );
  assert.deepEqual(contexts(currentAst).map(semantic), contexts(originalAst).map(semantic));
  assert.equal(current.includes("await context.close();"), true);
});

void test("the only awaited observer follows the original screenshot matcher and identical original error rethrow", () => {
  const matcher = calls(currentAst, "expectVisualParity");
  assert.equal(matcher.length, 1);
  const catches = callNodes(currentAst, (node) => node.type === "CatchClause");
  const matcherCatch = catches.find((node) =>
    current.slice(node.start, node.end).includes("sidecar?.failed(error)"),
  );
  assert(matcherCatch);
  assert.equal(matcherCatch.param.name, "error");
  const rethrows = callNodes(matcherCatch, (node) => node.type === "ThrowStatement");
  assert.equal(rethrows.length, 1);
  assert.equal(rethrows[0].argument.name, "error");
  const installation = calls(currentAst, "createVuefesFailureSidecar");
  assert.equal(installation.length, 1);
  assert(installation[0].start < calls(currentAst, "setupPage")[0].start);
  const observers = ["sidecar", "transport", "cssom", "json"]
    .map((name) => read(`tests/_helpers/vuefes-failure-${name}.mjs`))
    .join("\n");
  assert.doesNotMatch(
    observers,
    /\.screenshot\(|\.goto\(|\.reload\(|\.route\(|\.addStyleTag\(|\.waitForTimeout\(|\bfetch\(/,
  );
  assert.doesNotMatch(
    observers,
    /\.setTimeout\(|testInfo\.duration|TYPECHECK_DIVERGENCE_MODE|VIZE_VUEFES_VRT_OUTPUT_DIR/,
  );
  assert.match(observers, /readCiRequest\(nativeDir, ciSourceIdentity\(nativeDir\)\)/);
  assert.match(observers, /browser-decoded response bytes, not compressed wire bytes/);
});

void test("both observed dev failures retain isolated packets across the original full38 route matrix", () => {
  const originalRoutes = originalAst.body
    .flatMap((node) => (node.type === "VariableDeclaration" ? node.declarations : []))
    .find((node) => node.id.name === "routes")
    .init.elements.map(
      (node) => node.properties.find((property) => property.key.name === "name").value.value,
    );
  const observerGuard = callNodes(
    currentAst,
    (node) =>
      node.type === "IfStatement" &&
      calls(node.consequent, "createVuefesFailureSidecar").length > 0,
  );
  assert.equal(observerGuard.length, 1);
  const selects = (mode, route) =>
    runInNewContext(current.slice(observerGuard[0].test.start, observerGuard[0].test.end), {
      mode,
      route,
    });
  const selected = ["dev", "preview"].flatMap((mode) =>
    originalRoutes.filter((name) => selects(mode, { name })).map((name) => `${mode}-${name}`),
  );
  assert.equal(originalRoutes.length * 2, 38);
  assert.deepEqual(selected, ["dev-home", "dev-speaker"]);

  const output = calls(currentAst, "createVuefesFailureSidecar")[0].arguments[1].properties.find(
    (property) => property.key?.name === "outputDir",
  ).value;
  const outputFor = (path, __dirname, route) =>
    runInNewContext(current.slice(output.start, output.end), { path, __dirname, route });
  const resolve = { resolve: (...segments) => segments.join("/") };
  assert.equal(
    outputFor(resolve, "original-directory", { name: "home" }),
    "original-directory/../../../target/app-e2e-logs/vuefes-cascade",
  );
  assert.equal(
    outputFor(resolve, "original-directory", { name: "speaker" }),
    "original-directory/../../../target/app-e2e-logs/vuefes-speaker",
  );
});

void test("browser controls reuse the existing post-failure qualifier step and retain both failure statuses", () => {
  const { parse: parseYaml } = createRequire(new URL("../../package.json", import.meta.url))(
    "yaml",
  );
  const workflow = parseYaml(read(".github/workflows/vuefes-ssr-style-dependencies.yml"));
  const steps = workflow.jobs["vuefes-ssr-style-dependencies"].steps;
  const row = steps.find((step) => step.name === "Original full VueFes VRT, with one native build");
  assert.equal(row.uses, "./.github/actions/app-e2e-row");
  assert.deepEqual(row.with, {
    profile: "full",
    suite: "vrt",
    shard: "vuefes",
    task: "test:vrt:vuefes",
    timeout: "15m",
    "needs-playwright": "true",
    "cache-key": "app-e2e-full-vrt-vuefes",
    "worktree-id": "ci-app-e2e-full-vrt-vuefes",
    "artifact-stem": "full-vrt-vuefes",
  });
  const contracts = steps.find(
    (step) => step.name === "Native metadata corpus and existing Vite contracts",
  );
  assert.equal(contracts.if, "${{ !cancelled() }}");
  assert.match(
    contracts.run,
    /node --test tests\/tooling\/vuefes-failure-\*\.test\.mjs tools\/support\/compat\/vuefes\/passive-failure-sidecar\.browser\.test\.mjs \|\| vuefes_contract_rc=\$\?/,
  );
  assert.match(
    contracts.run,
    /node npm\/native\/scripts\/test-preparation\.mjs vp run --filter '\.\/npm\/builder\/vite' test:prepared \|\| vuefes_contract_rc=\$\?/,
  );
  assert.match(contracts.run, /exit "\$vuefes_contract_rc"/);
  assert.match(
    contracts.run,
    /node --experimental-vm-modules tools\/support\/compat\/vuefes\/page-meta-dependencies\.ts \|\| vuefes_contract_rc=\$\?/,
  );
  assert.equal(steps.filter((step) => step.uses === "./.github/actions/app-e2e-row").length, 1);
  const upload = steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(upload.if, "${{ always() }}");
  assert.equal(upload.with["include-hidden-files"], true);
  for (const extension of ["json", "bin", "diff", "node"])
    assert(
      upload.with.path
        .split("\n")
        .includes(`npm/native/.artifacts/native/formatter-history/*.${extension}`),
    );
  for (const binary of [
    "npm/native/.artifacts/native/formatter-history/*.node",
    "npm/native/*.node",
    "target/**/ci/libvize_vitrine.so",
  ])
    assert(upload.with.path.split("\n").includes(binary), binary);
});
