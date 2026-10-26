import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  compilerCssApiReport,
  compilerCssApiSources,
  loadCompilerCssApiCases,
  newCompilerCssApiCases,
  onlyCompilerCssChanges,
} from "../differential/compiler-css-api.ts";
import { cssBindingProducerPaths } from "../differential/compiler-css-api-registry.ts";
import {
  analyzeLegacyFix,
  failingProducts,
  legacyProducts,
} from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const manifestPath = "tests/_fixtures/differential/compiler/manifest.json";
const read = (file) =>
  fs.existsSync(path.join(root, file)) ? fs.readFileSync(path.join(root, file)) : null;
const from = (revision, file) => {
  try {
    return execFileSync("git", ["show", `${revision}:${file}`], {
      cwd: root,
      stdio: ["ignore", "pipe", "ignore"],
    });
  } catch {
    return null;
  }
};
const baseline = "3a3c8460f966fddbd8e1be4c083d460b24202f90",
  fixed = "91c25c1b4a2ea842705e4db321b6e92d2b70ba42";
const before = JSON.parse(from(baseline, manifestPath)),
  current = JSON.parse(read(manifestPath));
// Later ordinary compiler cases are not part of this CSS-only registration.
const cssOnly = Buffer.from(JSON.stringify({ ...current, cases: before.cases }));
const side = (side, file) =>
  side === "base"
    ? from(baseline, file)
    : file === manifestPath
      ? cssOnly
      : cssBindingProducerPaths.includes(file)
        ? from(fixed, file)
        : read(file);

await test("binding registration preserves every previous compiler row and pins its real source-before packet without changing source roles", () => {
  const oldCss = before.cssApiCases,
    newCss = current.cssApiCases;
  // Preserve actual existing rows while allowing independent compiler campaigns
  // to append their own genuine cases and runtime packs.
  for (const row of before.cases)
    assert.deepEqual(
      current.cases.find((candidate) => candidate.id === row.id),
      row,
    );
  for (const pack of before.runtimePacks)
    assert.deepEqual(
      current.runtimePacks.find((candidate) => candidate.path === pack.path),
      pack,
    );
  assert.deepEqual(newCss.slice(0, oldCss.length), oldCss);
  const loaded = loadCompilerCssApiCases(newCss, read);
  const entry = loaded.find((row) => row.id === "compiler/css/v-bind-source-map-10712");
  assert.ok(entry);
  assert.deepEqual(entry.before, {
    code: ".a {\n  color: var(--123abc-theme);\n}\n\n.b {\n  color: #00f;\n}\n",
    cssVars: ["theme"],
    errors: [],
    warnings: [],
  });
  assert.deepEqual(JSON.parse(entry.after.map), {
    version: 3,
    file: "style.css",
    names: [],
    sources: ["style.css"],
    sourcesContent: [entry.source],
    mappings: "AAAA;;;;AAAuB",
  });
  const report = compilerCssApiReport(loaded);
  assert.equal(report.plannedCases, 2);
  assert.equal(report.nativeHandled, 0);
  assert.equal(report.nativeEquivalent, 0);
  assert.equal(report.pairedComparisons, 0);
  assert.equal(report.genericCliUnsupported, 2);
  assert.deepEqual(report.rows[1], {
    id: entry.id,
    state: "source-inspected",
    genericCli: "unsupported-css-api",
    wholeProductNative: "unsupported",
    inputSha256: createHash("sha256").update(entry.source).digest("hex"),
    sourceBeforeSha256: entry.beforeSha256,
    sourceAfterSha256: entry.sourceAfterSha256,
    witness: { runner: "tests/native/css-v-bind-source-maps.mjs", requestCount: 40 },
  });
});

await test("binding source gate admits only actual newly registered exact source pairs and refuses unrelated compiler work", () => {
  const ids = newCompilerCssApiCases(before.cssApiCases, current.cssApiCases, side);
  assert.deepEqual(ids, ["compiler/css/v-bind-source-map-10712"]);
  const admitted = compilerCssApiSources(ids);
  assert.deepEqual([...admitted], cssBindingProducerPaths);
  assert.equal(onlyCompilerCssChanges(cssBindingProducerPaths, admitted), true);
  const request = {
    title: "fix(css): preserve binding source origins",
    paths: cssBindingProducerPaths,
    readFile: side,
  };
  assert.deepEqual(failingProducts(analyzeLegacyFix(request)), []);
  const actualChanged = execFileSync("git", ["diff", "--name-only", baseline, fixed], {
    cwd: root,
    encoding: "utf8",
  })
    .trim()
    .split("\n");
  assert.deepEqual(
    failingProducts(analyzeLegacyFix({ ...request, paths: actualChanged })),
    [],
    "all actual producer source changes must be certified, including cfg(test) modules",
  );
  for (const file of cssBindingProducerPaths)
    assert.deepEqual(legacyProducts([file]), ["compiler"]);
  for (const extra of [
    "crates/vize_atelier_sfc/src/style.rs",
    "crates/vize_atelier_ssr/src/compile.rs",
    "vendor/lightningcss/src/printer.rs",
  ]) {
    assert.equal(onlyCompilerCssChanges([...cssBindingProducerPaths, extra], admitted), false);
    assert.deepEqual(
      failingProducts(analyzeLegacyFix({ ...request, paths: [...cssBindingProducerPaths, extra] })),
      ["compiler"],
    );
  }
  for (const actual of ["base", "head"])
    for (const file of cssBindingProducerPaths)
      assert.throws(() =>
        newCompilerCssApiCases(before.cssApiCases, current.cssApiCases, (which, name) =>
          which === actual && name === file ? Buffer.from("foreign") : side(which, name),
        ),
      );
});

await test("binding before receipts and whole maps cannot be substituted, relabeled or awarded native or generic CLI credit", () => {
  for (const file of current.cssApiCases[1].inputs.files)
    assert.throws(() =>
      loadCompilerCssApiCases(current.cssApiCases, (name) =>
        name === `tests/_fixtures/differential/compiler/v-bind-source-map-10712/${file.path}`
          ? Buffer.from("{}")
          : read(name),
      ),
    );
  for (const mutate of [
    (row) => {
      row.witness.requestCount = 39;
    },
    (row) => {
      row.witness.runner = "tests/native/css-compile-source-maps.mjs";
    },
    (row) => {
      row.genericCli = "completed";
    },
    (row) => {
      row.nativeCompatibility = "completed";
    },
    (row) => {
      row.adapter = "compile-css-original-input-map-10693";
    },
  ]) {
    const rows = structuredClone(current.cssApiCases);
    mutate(rows[1]);
    assert.throws(() => loadCompilerCssApiCases(rows, read));
  }
  const primary = JSON.parse(
    read(
      "tests/_fixtures/differential/compiler/v-bind-source-map-10712/source-before-observation.json",
    ),
  );
  assert.equal(primary.identity.source, baseline);
  assert.equal(primary.identity.side, "head");
  assert.equal(primary.call.result.map, undefined);
});
