import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { asArray, asRecord, asString } from "../../tools/support/json-value.ts";
import {
  ancestry,
  applyReturnedEdits,
  checkout,
  completedProducer,
  corpus,
  custody,
  expected,
  expectedCli,
  expectedEslint,
  referenceFor,
  request,
  requiredMerges,
  sourceFor,
  validateSourceCopies,
} from "../../tools/support/release/patina_attribute_ignore_public/contract.ts";
import { configSource } from "../../tools/support/release/patina_attribute_ignore_public/cli.ts";
import {
  assertObserverOptions,
  observerOption,
  strictHostJson,
} from "../../tools/support/release/patina_attribute_ignore_public/plugin.ts";

const candidate = "b".repeat(40),
  run = "123";
const environment: NodeJS.ProcessEnv = {
  GITHUB_ACTIONS: "true",
  GITHUB_REPOSITORY: "ubugeeei-prod/vize",
  GITHUB_REF: "refs/heads/main",
  GITHUB_EVENT_NAME: "workflow_dispatch",
  GITHUB_WORKFLOW_REF:
    "ubugeeei-prod/vize/.github/workflows/patina-attribute-ignore-public.yml@refs/heads/main",
};

await test("the finite source owner keeps all original input/oracle bytes and separate provider identities", () => {
  const source = custody();
  assert.equal(source.source, requiredMerges[1]);
  assert.deepEqual(source.referenceProviders, {
    "eslint-plugin-vue": "10.9.2",
    eslint: "10.4.1",
    "vue-eslint-parser": "10.4.1",
    "@typescript-eslint/parser": "8.65.0",
  });
  assert.deepEqual(source.targetProviders, {
    eslint: "10.13.0",
    "vue-eslint-parser": "10.4.1",
    oxlint: "1.78.0",
    "@oxlint/plugins": "1.78.0",
  });
  assert.equal(corpus.valid.length, 9);
  assert.equal(corpus.invalid.length, 6);
  for (const row of corpus.valid) {
    const input = sourceFor(row),
      reference = referenceFor(row),
      dto = expected(row);
    assert.equal(applyReturnedEdits(input, dto), reference.fixedSource, row.id);
    assert.equal(expectedEslint(row).length, row.expectedCount);
    assert.equal(asArray(expectedCli(row)).length, 1);
  }
});

await test("source custody refuses missing changed duplicated or escaped original inputs", () => {
  const source = custody(),
    reader = (path: string) => readFileSync(join(checkout, path));
  assert.deepEqual(validateSourceCopies(source, reader), source);
  for (const mutation of [
    (row: Record<string, unknown>) => {
      row.role = "unrelated-role";
    },
    (row: Record<string, unknown>) => {
      row.source = "0".repeat(40);
    },
    (row: Record<string, unknown>) => {
      asRecord(row.referenceProviders).eslint = "10.13.0";
    },
    (row: Record<string, unknown>) => {
      asArray(row.files).pop();
    },
    (row: Record<string, unknown>) => {
      const rows = asArray(row.files);
      rows[1] = rows[0];
    },
    (row: Record<string, unknown>) => {
      asRecord(asArray(row.files)[0]).path = "../outside";
    },
  ]) {
    const changed = structuredClone(source);
    mutation(changed);
    assert.throws(() => validateSourceCopies(changed, reader));
  }
  const first = asString(asRecord(asArray(source.files)[0]).path);
  assert.throws(() =>
    validateSourceCopies(source, (path) =>
      path === first ? Buffer.concat([reader(path), Buffer.from("x")]) : reader(path),
    ),
  );
  assert.throws(() =>
    validateSourceCopies(source, () => {
      throw new Error("missing original");
    }),
  );
});

await test("only the main workflow can request one exact published candidate without source overrides", () => {
  request("0.999.0", candidate, run, environment);
  for (const patch of [
    { GITHUB_ACTIONS: "false" },
    { GITHUB_REF: "refs/heads/other" },
    { GITHUB_EVENT_NAME: "pull_request" },
    { GITHUB_REPOSITORY: "other/repository" },
    { GITHUB_WORKFLOW_REF: "other.yml@refs/heads/main" },
    { NODE_OPTIONS: "--require=source.cjs" },
    { NODE_PATH: "/outside" },
    { NAPI_RS_NATIVE_LIBRARY_PATH: "/source.node" },
    { VIZE_PREFER_WORKSPACE_BINDING: "1" },
    { NAPI_RS_FORCE_WASI: "1" },
    { VIZE_PUBLIC_NATIVE_CUSTODY: "{}" },
    { VIZE_ATTR_PUBLIC_OBSERVER: "/forged.json" },
  ])
    assert.throws(() => request("0.999.0", candidate, run, { ...environment, ...patch }));
  for (const args of [
    ["latest", candidate, run],
    ["0.999.0", "main", run],
    ["0.999.0", candidate, "0"],
  ])
    assert.throws(() => request(args[0]!, args[1]!, args[2]!, environment));
});

await test("producer admission refuses running failed cancelled neutral and wrong-owner records", () => {
  const facts = {
    id: 123,
    repository: { full_name: "ubugeeei-prod/vize" },
    head_branch: "main",
    path: ".github/workflows/release.yml",
    event: "workflow_dispatch",
    status: "completed",
    conclusion: "success",
    head_sha: "a".repeat(40),
    run_attempt: 1,
  };
  assert.deepEqual(completedProducer(facts, run), facts);
  for (const patch of [
    { status: "in_progress" },
    { status: "queued" },
    { conclusion: "failure" },
    { conclusion: "cancelled" },
    { conclusion: "neutral" },
    { repository: { full_name: "other/repo" } },
    { head_branch: "other" },
    { path: "other.yml" },
    { event: "push" },
    { head_sha: "main" },
    { run_attempt: 0 },
    { id: 124 },
  ])
    assert.throws(() => completedProducer({ ...facts, ...patch }, run));
  const merge = requiredMerges[1];
  const comparison = {
    base_commit: { sha: merge },
    merge_base_commit: { sha: merge },
    behind_by: 0,
    status: "ahead",
  };
  ancestry(comparison, merge);
  for (const patch of [
    { base_commit: { sha: candidate } },
    { merge_base_commit: { sha: candidate } },
    { behind_by: 1 },
    { status: "diverged" },
    { status: "behind" },
  ])
    assert.throws(() => ancestry({ ...comparison, ...patch }, merge));
});

await test("actual returned edits and complete host JSON reject malformed boundaries without output filtering", () => {
  const row = corpus.valid[0]!,
    input = sourceFor(row),
    dto = expected(row);
  assert.equal(applyReturnedEdits(input, dto), referenceFor(row).fixedSource);
  for (const edits of [
    [{ start: -1, end: 2, newText: "x" }],
    [{ start: 2, end: 1, newText: "x" }],
    [{ start: 0, end: input.length + 1, newText: "x" }],
    [{ start: "0", end: 1, newText: "x" }],
    [
      { start: 1, end: 5, newText: "x" },
      { start: 3, end: 7, newText: "y" },
    ],
  ])
    assert.throws(() => applyReturnedEdits(input, { diagnostics: [{ fix: { edits } }] }));
  assert.deepEqual(strictHostJson('{"diagnostics":[]}\n'), { diagnostics: [] });
  for (const raw of [
    "",
    'WARNING: JS plugins are experimental\n{"diagnostics":[]}',
    '{"diagnostics":[]}trailing',
    '{"diagnostics":[]}\n{}',
    "[]",
  ])
    assert.throws(() => strictHostJson(raw));
  assertObserverOptions(observerOption);
  for (const flag of [
    undefined,
    "",
    observerOption + " --require=/other.cjs",
    "--import=file:///other.ts",
  ])
    assert.throws(() => assertObserverOptions(flag));
});

await test("both original CLI carrier forms preserve the complete source policy and disabled-rule control", () => {
  for (const row of corpus.valid) {
    const body = JSON.stringify({
      linter: {
        preset: "incremental",
        rules: { "vue/attribute-hyphenation": "error" },
        ruleOptions: { "vue/attribute-hyphenation": row.policy },
      },
    });
    assert.equal(configSource(row.policy, false), `export default { vue: ${body} };\n`);
    assert.equal(configSource(row.policy, true), `export default () => ({ vue: ${body} });\n`);
  }
  assert(configSource({ ignore: [] }, false, true).includes('"vue/attribute-hyphenation":"off"'));
});
