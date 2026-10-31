import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";
import {
  localImportInputs,
  planToolingTests,
  proseReferences,
  readsProse,
  SHARED_HELPER_IMPORTERS,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import {
  auditedCliCone,
  toolingProductSelection,
  toolingSourceConeIdentity,
} from "../../tools/support/compat/github/tooling-product-inputs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const campaign = "tests/tooling/content-mapper-instance-globals-10617.test.ts";
const nativeDeclaration = "npm/native/index.d.ts";
const originalSeven = [
  "crates/vize_vitrine/src/napi/format.rs",
  nativeDeclaration,
  "tests/tooling/glyph-line-endings-native.test.ts",
  "tests/tooling/glyph-line-endings-types.test.ts",
  "docs/content/guide/formatter.md",
  "docs/content/ja/guide/formatter.md",
  "docs/davinci/decisions/2026-10-14-6098-glyph-format-presets.md",
];
function fixture(t: TestContext) {
  const cwd = mkdtempSync(path.join(tmpdir(), "vize-cli-product-cone-"));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  const closure: unknown = localImportInputs(campaign, root);
  assert.ok(closure && typeof closure === "object" && "files" in closure);
  assert.ok(
    Array.isArray(closure.files) &&
      closure.files.every((name: unknown) => typeof name === "string"),
  );
  for (const name of [...closure.files, originalSeven[2]!, originalSeven[3]!]) {
    const target = path.join(cwd, name);
    mkdirSync(path.dirname(target), { recursive: true });
    copyFileSync(path.join(root, name), target);
  }
  for (const directory of ["tests/tooling/davinci", "tests/tooling/release", "npm/native"])
    mkdirSync(path.join(cwd, directory), { recursive: true });
  writeFileSync(path.join(cwd, "npm/native/index.js"), "export const formatSfc = () => {};\n");
  // Preserve the real selector's shared-helper context without importing the
  // campaign itself: its executable literals must still be tested as narrow.
  for (let index = 0; index <= SHARED_HELPER_IMPORTERS; index += 1)
    writeFileSync(
      path.join(cwd, `tests/tooling/shared-launch-${index}.test.ts`),
      'import "./support/lsp/launch.ts"; import "../_helpers/realworld-patch.ts";',
    );
  const selected = (paths: string[]) => planToolingTests(paths, { cwd }).tests as string[];
  return {
    cwd,
    selected,
    identity: toolingSourceConeIdentity(cwd, closure.files),
    members: closure.files as string[],
    file: (name: string) => path.join(cwd, name),
  };
}

test("the original seven paths honor the exact audit and retain the Glyph checks", (t) => {
  const f = fixture(t);
  const source = readFileSync(path.join(root, campaign), "utf8");
  const legacyInference = readsProse(nativeDeclaration, [proseReferences(source)]);
  const auditMatched = f.identity === auditedCliCone;
  if (auditMatched) assert.equal(legacyInference, true);
  const selected = f.selected(originalSeven);
  assert.equal(
    selected.includes(campaign),
    !auditMatched,
    JSON.stringify(
      originalSeven.map((input) => ({ input, selected: f.selected([input]).includes(campaign) })),
    ),
  );
  assert.ok(selected.includes(originalSeven[2]!));
  assert.ok(selected.includes(originalSeven[3]!));
  t.diagnostic(
    JSON.stringify({
      schema: "vize-cli-product-cone-selection-v1",
      paths: originalSeven,
      legacyBareExecutableInference: legacyInference,
      auditMatched,
      cliCampaignSelected: selected.includes(campaign),
      glyphChecks: selected.filter((file) =>
        /glyph-line-endings-(?:native|types)\.test\.ts$/u.test(file),
      ),
    }),
  );
});

test("all CLI producers, copied declarations, fixtures and selector changes retain the campaign", (t) => {
  const f = fixture(t);
  for (const input of [
    "crates/vize/src/content_mapper.rs",
    "crates/vize_canon/src/lib.rs",
    "crates/vize_corsa/src/lib.rs",
    "crates/new-cli-dependency/src/lib.rs",
    "davinci/vize_l0/src/lib.rs",
    "vendor/typechecker-runtime/src/lib.rs",
    ".cargo/config.toml",
    ".github/workflows/full-qualification.yml",
    "npm/cli/content-mapper-helpers.d.ts",
    "npm/cli/generated/content-mapper-types.d.ts",
    "tests/fixtures/typechecker/undeclared-instance-globals-10617/authority.json",
    "tests/tooling/support/content-mapper-instance-globals-hosts-10617.ts",
    "tools/support/compat/github/source-cli-identity.ts",
    "tools/support/compat/github/plan-tooling-tests.mjs",
    "tools/support/compat/github/tooling-product-inputs.ts",
  ]) {
    assert.ok(f.selected([input]).includes(campaign), input);
    assert.ok(f.selected([...originalSeven, input]).includes(campaign), input);
  }
});

test("unknown and shared inputs never inherit an independent-product decision", (t) => {
  const f = fixture(t);
  for (const input of [
    "new-source-root/future-input.ts",
    "npm/native/new-public-types.d.ts",
    "package.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "Cargo.toml",
    "Cargo.lock",
  ])
    assert.ok(f.selected([nativeDeclaration, input]).includes(campaign), input);
});

test("an unresolved or computed import restores the campaign for Node declarations", (t) => {
  for (const mutation of ["import(variable);", 'import "./missing-consumer.ts";']) {
    const f = fixture(t);
    writeFileSync(f.file(campaign), readFileSync(f.file(campaign), "utf8") + "\n" + mutation);
    assert.ok(f.selected([nativeDeclaration]).includes(campaign), mutation);
  }
});

test("direct, transitive and joined native reads invalidate the audited source cone", (t) => {
  for (const mutation of [
    'import "../../npm/native/index.js";',
    'const input = "npm/native/index.d.ts";',
    'import "@vizejs/native";',
    'const input = path.join(root, "npm", "native", "index.d.ts");',
  ]) {
    const f = fixture(t);
    const owner = mutation.startsWith("import")
      ? campaign
      : "tests/tooling/support/content-mapper-instance-globals-hosts-10617.ts";
    writeFileSync(f.file(owner), readFileSync(f.file(owner), "utf8") + "\n" + mutation);
    assert.ok(f.selected([nativeDeclaration]).includes(campaign), mutation);
  }
});

test("future opaque filesystem reads invalidate even a complete local-import cone", (t) => {
  for (const owner of [
    campaign,
    "tests/tooling/support/content-mapper-instance-globals-hosts-10617.ts",
  ]) {
    const f = fixture(t);
    const mutation =
      "if (process.env.VIZE_EXTRA_DECL) fs.readFileSync(process.env.VIZE_EXTRA_DECL);";
    writeFileSync(f.file(owner), readFileSync(f.file(owner), "utf8") + "\n" + mutation);
    assert.equal(localImportInputs(campaign, f.cwd).complete, true);
    assert.equal(readsProse(nativeDeclaration, [proseReferences(mutation)]), false);
    assert.ok(f.selected([nativeDeclaration]).includes(campaign), owner);
  }
});

test("a new closed helper or workspace alias cannot reuse the prior cone audit", (t) => {
  for (const mutation of [
    'import "./support/new-cone-input.ts";',
    'import "#native-declarations";',
  ]) {
    const f = fixture(t);
    writeFileSync(f.file("tests/tooling/support/new-cone-input.ts"), "export const input = 1;");
    writeFileSync(
      f.file("package.json"),
      JSON.stringify({ imports: { "#native-declarations": "./npm/native/index.js" } }),
    );
    writeFileSync(f.file(campaign), readFileSync(f.file(campaign), "utf8") + "\n" + mutation);
    assert.equal(localImportInputs(campaign, f.cwd).complete, true);
    assert.ok(f.selected([nativeDeclaration]).includes(campaign), mutation);
  }
});

test("unregistered consumers and lost CLI-adapter identity remain conservative", () => {
  const current: unknown = localImportInputs(campaign, root);
  assert.ok(current && typeof current === "object" && "files" in current);
  assert.ok(
    Array.isArray(current.files) &&
      current.files.every((name: unknown) => typeof name === "string"),
  );
  const paths = [nativeDeclaration];
  const valid = {
    complete: true,
    files: [campaign],
    references: ["npm", "npm/cli", "content-mapper"],
    identity: auditedCliCone,
  };
  // Exercise the eligible fast path even after a future live consumer drift
  // makes the repository-backed counterfactual correctly run the campaign.
  assert.deepEqual(
    toolingProductSelection(campaign, paths, () => valid),
    {
      required: false,
      paths: [],
    },
  );
  assert.deepEqual(
    toolingProductSelection(campaign, originalSeven, () => valid),
    {
      required: false,
      paths: originalSeven.filter((file) => file !== nativeDeclaration),
    },
  );
  for (const identity of [undefined, "unreviewed-consumer-body"])
    assert.equal(
      toolingProductSelection(campaign, paths, () => ({ ...valid, identity })).required,
      true,
    );
  assert.deepEqual(
    toolingProductSelection("other.test.ts", paths, () => {
      throw new Error("unrelated consumers must not scan this cone");
    }),
    {
      required: false,
      paths,
    },
  );
  assert.equal(
    toolingProductSelection(campaign, paths, () => ({ ...valid, complete: false })).required,
    true,
  );
  assert.equal(
    toolingProductSelection(campaign, paths, () => ({ ...valid, references: ["npm"] })).required,
    true,
  );
  for (const reference of [
    "../npm/native",
    "npm\\native\\index.d.ts",
    "/checkout/npm/native/index.d.ts",
    "@vizejs/native",
    "@vizejs/native/format",
  ])
    assert.equal(
      toolingProductSelection(campaign, paths, () => ({
        ...valid,
        references: [...valid.references, reference],
      })).required,
      true,
      reference,
    );
});

test("every consumer/helper body is part of the exact dependency audit", (t) => {
  const original = fixture(t);
  for (const member of original.members) {
    const f = fixture(t);
    writeFileSync(
      f.file(member),
      readFileSync(f.file(member), "utf8") + "\n// new consumer body\n",
    );
    assert.equal(localImportInputs(campaign, f.cwd).complete, true);
    assert.ok(f.selected([nativeDeclaration]).includes(campaign), member);
  }
});

test("a shadowed command runner cannot inherit the prior source-cone audit", (t) => {
  const f = fixture(t);
  // Exact marker replacements are meaningful only for the exact audited body.
  // A future rename/reformat must restore execution without creating a new gate.
  if (f.identity !== auditedCliCone) {
    assert.ok(f.selected([nativeDeclaration]).includes(campaign));
    return;
  }
  const source = readFileSync(f.file(campaign), "utf8");
  const marker = "    try {\n      for (const root of [published, reference])";
  const mutated = source
    .replace("const run = (label:", "const sourceRun = (label:")
    .replace(
      marker,
      "    const run: typeof sourceRun = (label, command, args, cwd, input) => {\n" +
        "      if (process.env.VIZE_EXTRA_DECL) fs.readFileSync(process.env.VIZE_EXTRA_DECL);\n" +
        "      return sourceRun(label, command, args, cwd, input);\n    };\n" +
        marker,
    );
  assert.notEqual(mutated, source);
  assert.ok(mutated.includes("const sourceRun = (label:"));
  assert.ok(mutated.includes("const run: typeof sourceRun"));
  writeFileSync(f.file(campaign), mutated);
  assert.equal(localImportInputs(campaign, f.cwd).complete, true);
  assert.ok(f.selected([nativeDeclaration]).includes(campaign));
});

test("audit identities preserve paths and every source byte and reject missing inputs", (t) => {
  const cwd = mkdtempSync(path.join(tmpdir(), "vize-cone-identity-"));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  writeFileSync(path.join(cwd, "a.ts"), "x");
  const expected = "1056313a922e2acddacdd46f4f78038f0d7db26da5fa22146aeca5d566a857b7";
  assert.equal(toolingSourceConeIdentity(cwd, ["a.ts"]), expected);
  assert.equal(toolingSourceConeIdentity(cwd, ["a.ts", "a.ts"]), expected);
  writeFileSync(path.join(cwd, "a.ts"), "x\n");
  assert.notEqual(toolingSourceConeIdentity(cwd, ["a.ts"]), expected);
  writeFileSync(path.join(cwd, "b.ts"), "x");
  assert.notEqual(toolingSourceConeIdentity(cwd, ["b.ts"]), expected);
  assert.equal(toolingSourceConeIdentity(cwd, ["missing.ts"]), undefined);
  assert.equal(toolingSourceConeIdentity(cwd, ["../unknown.ts"]), undefined);
});

test("full and release-style merge inventories keep every original runtime scenario", (t) => {
  const f = fixture(t);
  const plan = planToolingTests(originalSeven, { cwd: f.cwd, tier: "merge" });
  assert.deepEqual(plan.tests, toolingTestFiles(f.cwd));
  assert.ok(plan.tests.includes(campaign));
});
