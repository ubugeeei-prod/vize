import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  compilerCssApiReport,
  cssMapProducerPaths,
  loadCompilerCssApiCases,
  newCompilerCssApiCases,
  onlyCompilerCssChanges,
} from "../differential/compiler-css-api.ts";
import { loadCompilerManifest } from "../differential/compiler.mjs";
import { readCargoMetadata } from "../../tools/support/compat/github/plan-affected-rust.ts";
import { readJsWorkspace } from "../../tools/support/compat/github/plan-affected-js.ts";
import { planPrChecks } from "../../tools/support/compat/github/plan-pr-checks.ts";
import { canonicalSourceReport } from "../../tools/support/compat/github/canonical-corpus-selection.mjs";
import {
  analyzeLegacyFix,
  failingProducts,
} from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const manifestPath = path.join(root, "tests/_fixtures/differential/compiler/manifest.json");
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const read = (file) =>
  fs.existsSync(path.join(root, file)) ? fs.readFileSync(path.join(root, file)) : null;

await test("closed actual CSS API inputs remain separate from all SSR execution and native rates", () => {
  const loaded = loadCompilerManifest(manifestPath);
  assert.equal(loaded.cases.length, manifest.cases.length);
  assert.equal(loaded.cssApiCases.length, 1);
  assert.ok(loaded.cases.every((row) => row.targets.length === 1 && row.targets[0] === "ssr"));
  const report = compilerCssApiReport(loaded.cssApiCases);
  assert.deepEqual(report, {
    plannedCases: 1,
    genericCliUnsupported: 1,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
    rows: [
      {
        id: "compiler/css/source-map-10693",
        state: "source-inspected",
        genericCli: "unsupported-css-api",
        wholeProductNative: "unsupported",
        inputSha256: "95eb31d1885582970845eecdd729bd7794feab04b648ad9240859fe5375628ba",
        publicBeforeSha256: "a77d5512fc0ea4d50229b55649cd505d61e481d544118bbeeb746d39d1a1b9d6",
        sourceAfterSha256: "38dae0e480b12f25f532450c61f08f2721aa0bff940c2121e9c10b0f1c839ea1",
        witness: { runner: "tests/native/css-compile-source-maps.mjs", requestCount: 84 },
      },
    ],
  });
});

await test("the actual required fixture gate admits only the exact CSS producer and refuses mixed changes", () => {
  const historical = (revision, file) => {
    try {
      return execFileSync("git", ["show", `${revision}:${file}`], {
        cwd: root,
        stdio: ["ignore", "pipe", "ignore"],
      });
    } catch {
      return null;
    }
  };
  const manifestRelative = "tests/_fixtures/differential/compiler/manifest.json";
  const before = "b0033883518f364cfbd3dd0ab9464715e9f51758";
  const after = "510b0bb1033a22265e3408693a4aa00b78351cef";
  const original = JSON.parse(historical(before, manifestRelative));
  const cssOnly = {
    ...manifest,
    // This historical CSS-only change predates later ordinary compiler cases.
    // Keep those genuine current additions outside this unit test's diff.
    cases: original.cases,
    cssApiCases: manifest.cssApiCases.filter((row) => row.id === "compiler/css/source-map-10693"),
  };
  const readSide = (side, file) => {
    if (side === "base") return historical(before, file);
    if (file === manifestRelative) return Buffer.from(JSON.stringify(cssOnly));
    if (cssMapProducerPaths.includes(file)) return historical(after, file);
    return read(file);
  };
  const request = {
    title: "fix(css): emit source maps",
    paths: cssMapProducerPaths,
    readFile: readSide,
  };
  const valid = analyzeLegacyFix(request);
  assert.deepEqual(failingProducts(valid), []);
  assert.deepEqual(valid.products[0].cases, ["compiler/css/source-map-10693"]);
  for (const extra of [
    "crates/vize_atelier_sfc/src/style.rs",
    "crates/vize_atelier_ssr/src/compile.rs",
  ])
    assert.deepEqual(
      failingProducts(analyzeLegacyFix({ ...request, paths: [...cssMapProducerPaths, extra] })),
      ["compiler"],
    );
  assert.deepEqual(
    failingProducts(
      analyzeLegacyFix({
        ...request,
        readFile: (side, file) =>
          side === "head" && file === cssMapProducerPaths[0]
            ? Buffer.from("foreign source")
            : readSide(side, file),
      }),
    ),
    ["compiler"],
  );
});

await test("CSS input, source-after map and forged completion cannot acquire generic execution credit", () => {
  const original = manifest.cssApiCases;
  for (const mutate of [
    (row) => {
      row.nativeCompatibility = "completed";
    },
    (row) => {
      row.genericCli = "completed";
    },
    (row) => {
      row.targets = ["ssr"];
    },
    (row) => {
      row.id = "compiler/css/foreign";
    },
    (row) => {
      row.witness.requestCount = 83;
    },
    (row) => {
      row.inputs.files[0].path = "Layout.vue.txt";
    },
    (row) => {
      row.inputs.files[3].sha256 = "0".repeat(64);
    },
    (row) => {
      row.nativeHandled = 1;
    },
  ]) {
    const changed = structuredClone(original);
    mutate(changed[0]);
    assert.throws(() => loadCompilerCssApiCases(changed, read));
  }
  assert.throws(() =>
    loadCompilerCssApiCases(original, (file) =>
      file.endsWith("source-after.json") ? Buffer.from("{}") : read(file),
    ),
  );
  assert.throws(() => loadCompilerCssApiCases([...original, ...original], read));
});

await test("CSS registration binds exact actual before/head producer bytes and refuses missing or unrelated files", () => {
  const readSide = (side, file) => {
    if (!cssMapProducerPaths.includes(file)) return read(file);
    const revision =
      side === "base"
        ? "b0033883518f364cfbd3dd0ab9464715e9f51758"
        : "510b0bb1033a22265e3408693a4aa00b78351cef";
    try {
      return execFileSync("git", ["show", `${revision}:${file}`], {
        cwd: root,
        stdio: ["ignore", "pipe", "ignore"],
      });
    } catch {
      assert.equal(side, "base");
      assert.equal(file, cssMapProducerPaths[3]);
      return null;
    }
  };
  assert.deepEqual(newCompilerCssApiCases(undefined, manifest.cssApiCases, readSide), [
    "compiler/css/source-map-10693",
  ]);
  assert.deepEqual(
    newCompilerCssApiCases(manifest.cssApiCases, manifest.cssApiCases, readSide),
    [],
  );
  for (const side of ["base", "head"])
    for (const source of cssMapProducerPaths)
      assert.throws(() =>
        newCompilerCssApiCases(undefined, manifest.cssApiCases, (actual, file) =>
          actual === side && file === source ? Buffer.from("foreign") : readSide(actual, file),
        ),
      );
  assert.equal(onlyCompilerCssChanges(cssMapProducerPaths), true);
  assert.equal(onlyCompilerCssChanges(["crates/vize_atelier_sfc/src/style.rs"]), false);
  assert.equal(
    onlyCompilerCssChanges([...cssMapProducerPaths, "crates/vize_atelier_sfc/src/compile.rs"]),
    false,
  );
});

await test("actual Cargo and tracked JS dependency graphs select required native CSS API proof for every producer and deleted path", () => {
  const cargoMetadata = readCargoMetadata(root),
    workspace = readJsWorkspace(root);
  for (const file of [
    ...cssMapProducerPaths,
    "crates/vize_atelier_sfc/src/css/parser/deleted.rs",
  ]) {
    const selected = planPrChecks({
      paths: [file],
      eventName: "pull_request",
      labels: [],
      cargoMetadata,
      workspace,
    });
    assert.equal(selected.js.conformance.ui, true, file);
  }
  const unrelated = planPrChecks({
    paths: ["README.md"],
    eventName: "pull_request",
    labels: [],
    cargoMetadata,
    workspace,
  });
  assert.equal(unrelated.js.conformance.ui, false);
});

await test("the required UI-conformance producer cannot be skipped, swallowed or detached from source-report", () => {
  const { parse } = createRequire(import.meta.url)("yaml");
  const workflow = parse(
    fs.readFileSync(path.join(root, ".github/workflows/pr-js-checks.yml"), "utf8"),
  );
  const steps = workflow.jobs["pr-js-package-conformance"].steps;
  const proof = steps.find(
    (step) => step.name === "Verify whole source CSS API maps with the built native image",
  );
  assert.deepEqual(Object.keys(proof).sort(), ["env", "if", "name", "run"]);
  assert.equal(workflow.jobs["pr-js-package-conformance"]["continue-on-error"], undefined);
  assert.equal(proof.run, "node --test tests/native/css-compile-source-maps.mjs");
  assert.equal(proof.env.VIZE_CSS_MAP_REQUIRE_SOURCE, "1");
  assert.equal(proof.env.VIZE_CSS_MAP_SOURCE_SHA, "${{ github.sha }}");
  assert.equal(proof.if, steps.find((step) => step.name === "Check UI package conformance").if);
  const outer = parse(
    fs.readFileSync(path.join(root, ".github/workflows/pr-source-checks.yml"), "utf8"),
  );
  assert.ok(outer.jobs["source-report"].needs.includes("pr-js-packages"));
  assert.equal(outer.jobs["pr-js-packages"].uses, "./.github/workflows/pr-js-checks.yml");
  assert.equal(outer.jobs["pr-js-packages"]["continue-on-error"], undefined);
  const needs = {
    "pr-source-plan": { result: "success", outputs: { "canonical-corpus": "true" } },
    "pr-js-packages": { result: "success" },
    "canonical-corpus": { result: "success" },
    "nuxt-module-qualification": { result: "skipped" },
  };
  assert.equal(canonicalSourceReport(needs, "pull_request").exitCode, 0);
  for (const result of ["failure", "cancelled", "skipped"])
    assert.equal(
      canonicalSourceReport({ ...needs, "pr-js-packages": { result } }, "pull_request").exitCode,
      1,
    );
});
