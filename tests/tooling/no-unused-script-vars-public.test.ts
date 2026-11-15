import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  validateCandidateTag,
  validateProducerIdentity,
} from "../../tools/support/release/no_unused_script_vars_public/authority.ts";
import {
  configSource,
  expectedGlobalPacket,
  expectedOppositePacket,
  expectedQuietPacket,
  validateRequest,
} from "../../tools/support/release/no_unused_script_vars_public/run.ts";

const root = join(import.meta.dirname, "../..");
const corpus = JSON.parse(
  readFileSync(
    join(root, "tests/_fixtures/differential/linter/no-unused-script-vars-12999/corpus.json"),
    "utf8",
  ),
);
const candidate = "b".repeat(40);
const releaseSource = "a".repeat(40);
const environment: NodeJS.ProcessEnv = {
  GITHUB_ACTIONS: "true",
  GITHUB_REPOSITORY: "ubugeeei-prod/vize",
  GITHUB_REF: "refs/heads/main",
  GITHUB_EVENT_NAME: "workflow_dispatch",
  GITHUB_WORKFLOW_REF:
    "ubugeeei-prod/vize/.github/workflows/no-unused-script-vars-public.yml@refs/heads/main",
};

await test("all eleven directive packets keep exact authored character coordinates", () => {
  assert.equal(corpus.globalCases.length, 11);
  const actual = corpus.globalCases.map((row: Parameters<typeof expectedGlobalPacket>[0]) => ({
    id: row.id,
    positions: expectedGlobalPacket(row)[0]!.messages.map((message) => [
      message.line,
      message.column,
      message.endLine,
      message.endColumn,
    ]),
  }));
  assert.deepEqual(actual, [
    { id: "eslint-invalid-5", positions: [[2, 10, 2, 11]] },
    {
      id: "eslint-invalid-52",
      positions: [
        [2, 19, 2, 22],
        [2, 24, 2, 27],
      ],
    },
    {
      id: "eslint-invalid-53",
      positions: [
        [3, 4, 3, 7],
        [4, 4, 4, 7],
      ],
    },
    { id: "eslint-invalid-61", positions: [[2, 18, 2, 22]] },
    { id: "eslint-invalid-62", positions: [[2, 20, 2, 21]] },
    { id: "eslint-invalid-63", positions: [[2, 11, 2, 15]] },
    { id: "eslint-invalid-64", positions: [[2, 11, 2, 17]] },
    { id: "eslint-invalid-65", positions: [[2, 10, 2, 13]] },
    { id: "eslint-invalid-66", positions: [[2, 14, 2, 15]] },
    { id: "eslint-invalid-67", positions: [[2, 14, 2, 15]] },
    { id: "eslint-invalid-101", positions: [[3, 1, 3, 4]] },
  ]);
  for (const row of corpus.globalCases) {
    const report = expectedGlobalPacket(row);
    assert.equal(report[0]!.errorCount, 0);
    assert.equal(report[0]!.warningCount, row.diagnostics.length);
    assert.ok(report[0]!.messages.every((message) => message.severity === 1));
  }
});

await test("quiet and opposite controls remain separate complete packets", () => {
  assert.deepEqual(expectedQuietPacket(), [
    { file: "UnusedScriptVarsCase.vue", messages: [], errorCount: 0, warningCount: 0 },
  ]);
  const opposite = expectedOppositePacket(corpus.oppositeCase.code);
  assert.deepEqual(
    opposite[0]!.messages.map(({ line, column, endLine, endColumn, message }) => ({
      line,
      column,
      endLine,
      endColumn,
      message,
    })),
    [
      {
        line: 3,
        column: 7,
        endLine: 3,
        endColumn: 8,
        message:
          "[vize:vue/no-unused-script-vars] 'a' is defined but never used. Allowed unused args must match /^_/u",
      },
      {
        line: 8,
        column: 3,
        endLine: 8,
        endColumn: 5,
        message:
          "[vize:vue/no-unused-script-vars] 'cb' is assigned a value but never used. Allowed unused args must match /^_/u",
      },
    ],
  );
});

await test("the carrier enables only the reported rule with the pinned options", () => {
  const config = configSource(corpus.options);
  assert.deepEqual(JSON.parse(config.slice("export default ".length, -2)), {
    vue: {
      linter: {
        preset: "incremental",
        rules: { "vue/no-unused-script-vars": "warn" },
        ruleOptions: { "vue/no-unused-script-vars": corpus.options },
      },
    },
  });
});

await test("only the merged main workflow can request an exact release", () => {
  validateRequest("0.760.0", candidate, releaseSource, "43747366056", environment);
  for (const patch of [
    { GITHUB_ACTIONS: "false" },
    { GITHUB_REPOSITORY: "other/repository" },
    { GITHUB_REF: "refs/heads/other" },
    { GITHUB_EVENT_NAME: "pull_request" },
    { GITHUB_WORKFLOW_REF: "other.yml@refs/heads/main" },
    { NODE_PATH: "/source" },
    { NAPI_RS_NATIVE_LIBRARY_PATH: "/source.node" },
    { VIZE_NATIVE_PATH: "/source.node" },
  ])
    assert.throws(() =>
      validateRequest("0.760.0", candidate, releaseSource, "43747366056", {
        ...environment,
        ...patch,
      }),
    );
  for (const [version, prepared, source, run] of [
    ["latest", candidate, releaseSource, "43747366056"],
    ["0.760.0", "main", releaseSource, "43747366056"],
    ["0.760.0", candidate, "main", "43747366056"],
    ["0.760.0", candidate, releaseSource, "0"],
  ])
    assert.throws(() => validateRequest(version!, prepared!, source!, run!, environment));
});

await test("release authority accepts both tag forms and rejects a multi-parent candidate", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-unused-vars-authority-"));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: directory, encoding: "utf8" }).trim();
  try {
    git("init", "-q");
    git("config", "user.name", "Vize public control");
    git("config", "user.email", "vize-public@example.com");
    writeFileSync(join(directory, "source.txt"), "source\n");
    git("add", "source.txt");
    git("commit", "-qm", "source");
    const source = git("rev-parse", "HEAD");
    writeFileSync(join(directory, "candidate.txt"), "candidate\n");
    git("add", "candidate.txt");
    git("commit", "-qm", "candidate");
    const candidate = git("rev-parse", "HEAD");
    git("tag", "lightweight", candidate);
    git("tag", "-a", "annotated", "-m", "annotated", candidate);
    assert.deepEqual(validateCandidateTag("lightweight", candidate, source, directory), {
      candidate,
      source,
    });
    assert.deepEqual(validateCandidateTag("annotated", candidate, source, directory), {
      candidate,
      source,
    });
    const tree = git("rev-parse", `${candidate}^{tree}`);
    const merge = execFileSync("git", ["commit-tree", tree, "-p", candidate, "-p", source], {
      cwd: directory,
      encoding: "utf8",
      input: "merge\n",
    }).trim();
    git("tag", "merge", merge);
    assert.throws(() => validateCandidateTag("merge", merge, candidate, directory));
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

await test("producer authority binds the exact run, candidate, source, version and attempt", () => {
  const run = {
    id: 43_747_366_056,
    run_attempt: 1,
    name: "Release",
    path: ".github/workflows/release.yml",
    event: "workflow_dispatch",
    status: "completed",
    conclusion: "success",
    head_sha: "c".repeat(40),
  };
  const artifacts = {
    artifacts: [{ id: 14_000_000_001, name: "release-producer-source-1", expired: false }],
  };
  const record = {
    schema: "vize.releaseProducerCandidate.v1",
    repository: "ubugeeei-prod/vize",
    runId: "43747366056",
    runAttempt: 1,
    workflowHead: run.head_sha,
    version: "0.760.0",
    candidateSha: candidate,
    producerSource: releaseSource,
    nativeManifestGitBlob: "d".repeat(40),
    inventoryGitBlob: "e".repeat(40),
  };
  const request = {
    version: "0.760.0",
    candidate,
    source: releaseSource,
    run: "43747366056",
  };
  assert.deepEqual(validateProducerIdentity(request, run, artifacts, record), {
    artifactId: 14_000_000_001,
    workflowHead: run.head_sha,
    runAttempt: 1,
  });
  const rerun = { ...run, run_attempt: 2 };
  assert.deepEqual(validateProducerIdentity(request, rerun, artifacts, record), {
    artifactId: 14_000_000_001,
    workflowHead: run.head_sha,
    runAttempt: 1,
  });
  const rerunArtifacts = {
    artifacts: [
      ...artifacts.artifacts,
      { id: 14_000_000_002, name: "release-producer-source-2", expired: false },
    ],
  };
  const rerunRecord = { ...record, runAttempt: 2 };
  assert.deepEqual(validateProducerIdentity(request, rerun, rerunArtifacts, rerunRecord), {
    artifactId: 14_000_000_002,
    workflowHead: run.head_sha,
    runAttempt: 2,
  });
  assert.throws(() => validateProducerIdentity(request, rerun, rerunArtifacts, record));
  for (const patch of [
    { candidateSha: "f".repeat(40) },
    { producerSource: "f".repeat(40) },
    { version: "0.761.0" },
    { runId: "43747366057" },
    { workflowHead: "f".repeat(40) },
  ])
    assert.throws(() => validateProducerIdentity(request, run, artifacts, { ...record, ...patch }));
  assert.throws(() =>
    validateProducerIdentity(request, { ...run, name: "Other workflow" }, artifacts, record),
  );
});
