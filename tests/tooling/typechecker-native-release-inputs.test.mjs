import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import {
  prepareNativeConsumerFixture,
  verifyConsumerRefusalControls,
  verifyNativeConsumerClosure,
  verifyWorkflowConsumers,
} from "./support/typechecker-native-release-consumers.mjs";
import {
  localImportInputs,
  planToolingTests,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
const workflowPath = ".github/workflows/typechecker-native-phases.yml";
const workflow = parse(readFileSync(join(root, workflowPath), "utf8"));
const pinPath = "tests/tooling/fixtures/typechecker-native-release-inputs.json";
const pins = JSON.parse(readFileSync(join(root, pinPath), "utf8"));
const excluded = [
  "tests/tooling/release-input-scopes.test.mjs",
  "tests/tooling/release/release-public-acceptance.test.ts",
  "tests/tooling/support/release-retirement-transport-law.ts",
];
const hash = (file) => createHash("sha256").update(readFileSync(file)).digest("hex");
const sorted = (values) =>
  [...values].sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));

function physicalSource(cwd, file) {
  for (let current = join(cwd, file); ; current = dirname(current)) {
    const stat = lstatSync(current);
    assert.ok(
      current === join(cwd, file) ? stat.isFile() : stat.isDirectory(),
      `${file}: physical reviewed source required`,
    );
    if (current === resolve(cwd)) return;
  }
}

// The actual workflow uses only literals, * and **. Implement that documented
// subset in order; reject new syntax instead of guessing GitHub's semantics.
function matches(path, pattern) {
  assert.match(pattern, /^[A-Za-z0-9_./*!-]+$/);
  const source = pattern.replace(/\*\*|\*|[.+^${}()|[\]\\]/g, (part) =>
    part === "**" ? ".*" : part === "*" ? "[^/]*" : `\\${part}`,
  );
  return new RegExp(`^${source}$`).test(path);
}
function sourceTriggered(files, patterns = workflow.on.pull_request.paths) {
  return files.some((file) => {
    let included = false;
    for (const pattern of patterns) {
      const negative = pattern.startsWith("!");
      if (matches(file, negative ? pattern.slice(1) : pattern)) included = !negative;
    }
    return included;
  });
}

/** An excluded source edit must fail Check until its whole cohort is reviewed. */
export function verifyReviewedReleaseInputs(cwd = root) {
  assert.equal(pins.schema, "vize-native-release-input-review-v1");
  assert.deepEqual(pins.excludedRoots, excluded);
  const present = excluded.filter((file) => {
    try {
      assert.ok(lstatSync(join(cwd, file)).isFile(), `${file}: physical reviewed root required`);
      physicalSource(cwd, file);
      return true;
    } catch (error) {
      if (error.code === "ENOENT") return false;
      throw error;
    }
  });
  assert.ok(present.length, "excluded roots cannot all disappear");
  const files = new Set();
  for (const file of present) {
    const closure = localImportInputs(file, cwd);
    assert.equal(closure.complete, true, `${file}: complete reviewed import closure required`);
    for (const member of closure.files) files.add(member);
  }
  const observed = {};
  for (const file of sorted(files)) {
    assert.doesNotMatch(file, /^(?:crates|davinci|npm|tools\/benchmarks)\//);
    physicalSource(cwd, file);
    observed[file] = hash(join(cwd, file));
  }
  const review = pins.reviews.find(
    (entry) =>
      JSON.stringify(sorted(entry.presentRoots)) === JSON.stringify(sorted(present)) &&
      JSON.stringify(entry.files) === JSON.stringify(observed),
  );
  assert.ok(
    review,
    "reviewed non-native body/import cohort changed: remove the exclusions or explicitly review and repin the complete source",
  );
  assert.match(review.sourceSha, /^[0-9a-f]{40}$/);
  return review.name;
}

function sourceFixture(t) {
  const directory = mkdtempSync(join(os.tmpdir(), "vize-native-release-inputs-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const present = excluded.filter((file) => existsSync(join(root, file)));
  const files = new Set(present.flatMap((file) => localImportInputs(file).files));
  for (const file of files) {
    mkdirSync(dirname(join(directory, file)), { recursive: true });
    copyFileSync(join(root, file), join(directory, file));
  }
  return directory;
}

void test("actual ordered YAML excludes exactly three reviewed release tests", () => {
  const paths = workflow.on.pull_request.paths;
  assert.deepEqual(
    paths.filter((path) => path.startsWith("!")),
    excluded.map((file) => `!${file}`),
  );
  assert.deepEqual(
    paths.filter((path) => !path.startsWith("!")),
    pins.positivePaths,
  );
  const tooling = paths.indexOf("tests/tooling/**");
  assert.deepEqual(
    paths.slice(tooling + 1, tooling + 4),
    excluded.map((file) => `!${file}`),
  );
  assert.equal(sourceTriggered(excluded), false);
  assert.equal(
    sourceTriggered([
      ...excluded,
      "docs/davinci/decisions/2026-09-27-level-restructure.md",
      "docs/release/2026-10-11-retirement-registry-transport.md",
      "tools/support/release/retirement_absence.ts",
      "tools/support/release/retirement_transport.ts",
    ]),
    false,
  );
  assert.equal(sourceTriggered(excluded, pins.positivePaths), true, "original broad trigger");
  assert.equal(
    sourceTriggered([excluded[0]], ["tests/tooling/**", `!${excluded[0]}`, excluded[0]]),
    true,
  );
});

void test("unknown siblings and every original product or native driver family remain positive", () => {
  for (const file of [
    "tests/tooling/release-input-scopes-new.test.mjs",
    "tests/tooling/release/release-public-acceptance-native.test.ts",
    "tests/tooling/support/release-retirement-transport-law.mjs",
    "tests/tooling/support/unknown-native-authority.ts",
    "tests/tooling/support/typechecker-native-release-consumers.mjs",
    "tests/tooling/fixtures/typechecker/native.ts",
    "tests/tooling/typechecker-native-release-inputs.test.mjs",
    pinPath,
    workflowPath,
    "crates/vize_canon/src/batch/type_checker/mod.rs",
    "crates/vize_maestro/src/ide/diagnostics/editor_typecheck_emit_tests.rs",
    "crates/vize_maestro/src/ide/diagnostics/editor_typecheck_defaults_tests.rs",
    "crates/vize/tests/check_original_cli.rs",
    "tests/_fixtures/differential/typechecker/original.vue",
    "tests/_fixtures/differential/lsp/original.vue",
    ".github/actions/check-reference-path/run.sh",
    "tools/benchmarks/scripts/typechecker-native-phases.mjs",
    "tools/benchmarks/scripts/type-snapshot-cli-corpus.mjs",
    "Cargo.toml",
    "Cargo.lock",
    "pnpm-lock.yaml",
  ]) {
    assert.equal(sourceTriggered([file]), true, file);
    assert.equal(sourceTriggered([...excluded, file]), true, `mixed ${file}`);
  }
});

void test("the actual full local imports and physical source bytes match one atomic review", () => {
  assert.ok(
    [
      "current-main",
      "reviewed-retirement-transport",
      "reviewed-docs-planner-composition",
      "reviewed-installed-vapor-rc10",
      "reviewed-init-js-manager-composition",
      "reviewed-native-ssr-direct-child-custody",
      "reviewed-init-native-ssr-composition",
    ].includes(verifyReviewedReleaseInputs()),
  );
  assert.equal(pins.reviews.length, 7);
  const rootCounts = pins.reviews.map((review) => review.presentRoots.length);
  assert.deepEqual(rootCounts, [2, 3, 3, 3, 3, 3, 3]);
});
void test("a native command with unchanged imports fails for both roots and transitive helpers", (t) => {
  for (const member of [excluded[0], "tools/support/release/retirement_evidence.ts"]) {
    const directory = sourceFixture(t);
    const before = localImportInputs(excluded[0], directory);
    const file = join(directory, member);
    writeFileSync(
      file,
      readFileSync(file, "utf8") +
        '\nprocess.getBuiltinModule("node:child_process").spawnSync("vize", ["check"]);\n',
    );
    assert.deepEqual(localImportInputs(excluded[0], directory), before, "imports remain unchanged");
    assert.throws(
      () => verifyReviewedReleaseInputs(directory),
      /reviewed non-native body\/import cohort changed/,
    );
  }
});
void test("unknown, unresolved, nonliteral and native source imports cannot inherit a review", (t) => {
  for (const declaration of [
    'import "./new-native-authority.ts";',
    'import "./missing-authority.ts";',
    "await import(process.env.NATIVE_AUTHORITY);",
    'import "../../crates/vize/src/unknown.ts";',
  ]) {
    const directory = sourceFixture(t);
    const file = join(directory, excluded[0]);
    writeFileSync(file, readFileSync(file, "utf8") + `\n${declaration}\n`);
    const candidate = declaration.includes("new-native")
      ? "tests/tooling/new-native-authority.ts"
      : declaration.includes("crates/")
        ? "crates/vize/src/unknown.ts"
        : undefined;
    if (candidate) {
      mkdirSync(dirname(join(directory, candidate)), { recursive: true });
      writeFileSync(join(directory, candidate), "export const native = true;\n");
    }
    assert.throws(() => verifyReviewedReleaseInputs(directory));
  }
});

void test("partial presence and linked source cannot mix reviewed source cohorts", (t) => {
  const removed = sourceFixture(t);
  rmSync(join(removed, excluded[0]));
  assert.throws(() => verifyReviewedReleaseInputs(removed), /cohort changed/);
  const presence = sourceFixture(t);
  if (existsSync(join(presence, excluded[2]))) rmSync(join(presence, excluded[2]));
  else writeFileSync(join(presence, excluded[2]), "export const unknown = true;\n");
  assert.throws(
    () => verifyReviewedReleaseInputs(presence),
    /cohort changed|complete reviewed import/,
  );
  for (const member of [excluded[0], "tools/support/release/retirement_evidence.ts"]) {
    const linked = sourceFixture(t);
    rmSync(join(linked, member));
    symlinkSync(join(root, member), join(linked, member));
    assert.throws(
      () => verifyReviewedReleaseInputs(linked),
      /physical reviewed (?:source|root) required/,
    );
  }
  const dangling = sourceFixture(t);
  rmSync(join(dangling, excluded[2]), { force: true });
  symlinkSync(join(dangling, "absent-root.ts"), join(dangling, excluded[2]));
  assert.throws(() => verifyReviewedReleaseInputs(dangling), /physical reviewed root required/);
  const ancestor = sourceFixture(t);
  const directory = join(ancestor, "tests/tooling/support");
  rmSync(directory, { recursive: true, force: true });
  symlinkSync(join(root, "tests/tooling/support"), directory);
  assert.throws(() => verifyReviewedReleaseInputs(ancestor), /physical reviewed source required/);
});

void test("actual native drivers, actions and Rust/npm sources do not consume excluded tests", (t) => {
  const review = pins.reviews.find(({ name }) => name === verifyReviewedReleaseInputs());
  verifyNativeConsumerClosure(root, review.files);
  verifyConsumerRefusalControls(t, root, workflowPath, excluded, review.files);
});

void test("delegated tools, root scripts and changed metadata cannot introduce native consumers", (t) => {
  const directory = sourceFixture(t);
  const producer = "tools/benchmarks/scripts/typechecker-native-phases.mjs";
  prepareNativeConsumerFixture(directory, root, producer);
  const review = pins.reviews.find(({ name }) => name === verifyReviewedReleaseInputs(directory));
  verifyNativeConsumerClosure(directory, review.files);
  const source = join(directory, producer);
  writeFileSync(
    source,
    readFileSync(source, "utf8") + '\nimport "../../support/native-release-delegate.mjs";\n',
  );
  const delegate = join(directory, "tools/support/native-release-delegate.mjs");
  writeFileSync(delegate, `import "../../${excluded[0]}";\n`);
  assert.equal(verifyReviewedReleaseInputs(directory), review.name, "excluded cohort unchanged");
  assert.throws(() => verifyNativeConsumerClosure(directory, review.files));
  rmSync(delegate);
  const manifest = join(directory, "package.json");
  const packageSource = JSON.parse(readFileSync(manifest, "utf8"));
  packageSource.scripts["native:release-input"] = `node --test ${excluded[1]}`;
  writeFileSync(manifest, JSON.stringify(packageSource));
  assert.equal(verifyReviewedReleaseInputs(directory), review.name, "unchanged script cohort");
  assert.throws(() => verifyNativeConsumerClosure(directory, review.files));
  copyFileSync(join(root, "package.json"), manifest);
  const metadata = join(directory, "tools/config/vite-plus/tooling-test-scopes.ts");
  writeFileSync(
    metadata,
    readFileSync(metadata, "utf8") + "\nexport const newNativeConsumer = true;\n",
  );
  assert.throws(
    () => verifyNativeConsumerClosure(directory, review.files),
    /reviewed consumer body/,
  );
});

void test("a future workflow command or input cannot introduce an excluded native consumer", () => {
  for (const authority of [
    { steps: [{ run: `node --test ${excluded[0]}` }] },
    { env: { NATIVE_CASE: excluded[1] }, steps: [{ run: 'node --test "$NATIVE_CASE"' }] },
    { uses: "./.github/workflows/native.yml", with: { native_source: excluded[2] } },
  ])
    assert.throws(() =>
      verifyWorkflowConsumers({ jobs: { newNativeConsumer: authority } }, "future"),
    );
  for (const kind of ["env", "workflow_dispatch", "workflow_call"]) {
    const inputs = { native_case: { default: excluded[0], options: [excluded[1]] } };
    const document =
      kind === "env" ? { env: { NATIVE_CASE: excluded[0] } } : { on: { [kind]: { inputs } } };
    assert.throws(() => verifyWorkflowConsumers(document, "future"));
  }
});

void test("Check still selects the release contract, this guard and all full merge contracts", () => {
  const guard = "tests/tooling/typechecker-native-release-inputs.test.mjs";
  for (const file of [
    ...excluded,
    "tools/support/release/retirement_absence.ts",
    "tools/support/release/retirement_transport.ts",
    "tests/tooling/support/typechecker-native-release-consumers.mjs",
  ]) {
    const plan = planToolingTests([file]);
    assert.ok(plan.tests.includes(excluded[1]), `${file}: public acceptance contract`);
    assert.ok(plan.tests.includes(guard), `${file}: source authority guard`);
  }
  const merge = planToolingTests(excluded, { tier: "merge" });
  assert.equal(merge.tests.length, merge.totalTests);
  assert.equal(merge.deferredTests, 0);
  assert.equal(workflow.on.workflow_dispatch.inputs.source_sha.required, true);
  assert.deepEqual(workflow.on.pull_request.types, ["opened", "synchronize", "reopened", "edited"]);
  assert.equal(workflow.jobs["native-phases"]["timeout-minutes"], 40);
});
