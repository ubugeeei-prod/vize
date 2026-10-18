import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  integrationIdentity,
  integrationReference,
  sha256,
  type Json,
} from "../../tools/support/release/consumer_recovery/policy.ts";
import { verifyStoredIntegration } from "../../tools/support/release/consumer_recovery/custody.ts";
import {
  assertCheckoutAuthority,
  policyFiles,
  sourceAuthority,
} from "../../tools/support/release/consumer_recovery/oracles.ts";

type Raw = { integration: Json; merge: Json; gates: { workflow_runs: Json[] } };
type Fixture = {
  schema: string;
  evidence: {
    run: number;
    attempt: number;
    artifact: { id: number; bytes: number; sha256: string };
    before: { kind: string; bytes: number; sha256: string };
    after: { kind: string; bytes: number; sha256: string };
    publication: { sha256: string };
  };
  identity: {
    cut: string;
    head: string;
    tag: string;
    sourcePr: string;
    run: string;
    attempt: number;
  };
  mutablePaths: string[];
  beforeRaw: string;
  afterRaw: string;
  publicationRaw: string;
};
const retained: Fixture = JSON.parse(
  readFileSync(
    new URL("../_fixtures/release/consumer-recovery-integration-drift.json", import.meta.url),
    "utf8",
  ),
);
const before = (): Raw => JSON.parse(retained.beforeRaw);
const after = (): Raw => JSON.parse(retained.afterRaw);
function location(raw: unknown, pointer: string) {
  const pieces = pointer.slice(1).split("/");
  const key = pieces.pop()!;
  let parent = raw;
  for (const name of pieces) {
    assert.ok(parent && typeof parent === "object");
    parent = (parent as Json)[name];
  }
  assert.ok(parent && typeof parent === "object");
  return { parent: parent as Json, key };
}
function set(raw: unknown, pointer: string, value: unknown) {
  const { parent, key } = location(raw, pointer);
  parent[key] = value;
}
function differences(left: unknown, right: unknown, pointer = ""): string[] {
  if (JSON.stringify(left) === JSON.stringify(right)) return [];
  if (left && right && typeof left === "object" && typeof right === "object") {
    return [...new Set([...Object.keys(left), ...Object.keys(right)])].flatMap((key) =>
      differences((left as Json)[key], (right as Json)[key], `${pointer}/${key}`),
    );
  }
  return [pointer];
}
const checkIndex = before().gates.workflow_runs.findIndex((run) => run.event === "push");
assert.ok(checkIndex >= 0);
const check = `/gates/workflow_runs/${checkIndex}`;

test("actual failed finalizer artifact and separate live follow-up preserve original raw bytes", () => {
  assert.equal(retained.schema, "vize-consumer-recovery-integration-drift-v1");
  assert.equal(retained.evidence.run, 38990889311);
  assert.equal(retained.evidence.attempt, 2);
  assert.equal(retained.evidence.artifact.id, 12084743637);
  assert.equal(retained.evidence.artifact.bytes, 902801);
  assert.equal(
    retained.evidence.artifact.sha256,
    "b684d5bfaa3d8f30505df446a5a6cb9fc2ac865820fb668eb5ff409794435bd0",
  );
  for (const side of ["before", "after"] as const) {
    const bytes = Buffer.from(side === "before" ? retained.beforeRaw : retained.afterRaw);
    assert.equal(bytes.length, retained.evidence[side].bytes);
    assert.equal(sha256(bytes), retained.evidence[side].sha256);
  }
  assert.equal(sha256(retained.publicationRaw), retained.evidence.publication.sha256);
  assert.match(retained.evidence.before.kind, /retained prepare snapshot/u);
  assert.match(retained.evidence.after.kind, /independent live API follow-up/u);
  assert.match(retained.evidence.after.kind, /not the un-emitted failed-job comparison snapshot/u);
  const publication = JSON.parse(retained.publicationRaw);
  assert.equal(publication.github.target, retained.identity.head);
  assert.equal(publication.github.integrationPr, String(before().integration.number));
});

test("only four observed repository timestamp paths drift while release authority remains identical", () => {
  const earlier = before(),
    later = after();
  assert.notEqual(retained.beforeRaw, retained.afterRaw);
  assert.notDeepEqual(earlier, later);
  const paths = differences(earlier, later).sort();
  assert.deepEqual(paths, [
    "/integration/base/repo/pushed_at",
    "/integration/base/repo/updated_at",
    "/integration/head/repo/pushed_at",
    "/integration/head/repo/updated_at",
  ]);
  assert.deepEqual(paths, retained.mutablePaths);
  assert.deepEqual(integrationIdentity(earlier), integrationIdentity(later));
  assert.equal(
    sha256(retained.beforeRaw),
    retained.evidence.before.sha256,
    "projection must not rewrite raw custody evidence",
  );
});

// A valid replacement value must change identity; malformed values may fail admission first.
const immutable: Record<string, unknown> = {
  "/integration/id": 4879670905,
  "/integration/number": 9198,
  "/integration/merged": false,
  "/integration/merge_commit_sha": "a".repeat(40),
  "/integration/head/sha": "a".repeat(40),
  "/integration/head/ref": "release-integration/v0.458.0",
  "/integration/head/repo/id": 1123610217,
  "/integration/head/repo/full_name": "foreign/vize",
  "/integration/base/sha": "a".repeat(40),
  "/integration/base/ref": "other",
  "/integration/base/repo/id": 1123610217,
  "/integration/base/repo/full_name": "foreign/vize",
  "/merge/sha": "a".repeat(40),
  "/merge/commit/tree/sha": "a".repeat(40),
  "/merge/parents/0/sha": "a".repeat(40),
  "/merge/parents": [{ sha: "a".repeat(40) }, { sha: "b".repeat(40) }],
  "/merge/commit/verification/verified": false,
  "/merge/commit/verification/reason": "unsigned",
  "/merge/commit/verification/signature": "different signature",
  "/merge/commit/verification/payload": "different signed payload",
  [`${check}/id`]: 38808071558,
  [`${check}/workflow_id`]: 237041398,
  [`${check}/path`]: ".github/workflows/other.yml",
  [`${check}/head_sha`]: "a".repeat(40),
  [`${check}/head_branch`]: "other",
  [`${check}/event`]: "workflow_dispatch",
  [`${check}/run_attempt`]: 2,
  [`${check}/status`]: "in_progress",
  [`${check}/conclusion`]: "failure",
  [`${check}/repository/id`]: 1123610217,
  [`${check}/repository/full_name`]: "foreign/vize",
  [`${check}/head_repository/id`]: 1123610217,
  [`${check}/head_repository/full_name`]: "foreign/vize",
};
for (const [pointer, value] of Object.entries(immutable)) {
  test(`stable integration authority detects immutable change ${pointer}`, () => {
    const changed = before();
    set(changed, pointer, value);
    let identity: unknown;
    try {
      identity = integrationIdentity(changed);
    } catch (error) {
      assert.ok(error instanceof Error);
      return;
    }
    assert.notDeepEqual(identity, integrationIdentity(before()), pointer);
  });
}

for (const key of ["source", "head", "cut", "tag"]) {
  test(`stable integration authority retains the exact canonical ${key} pin marker`, () => {
    const changed = before();
    const name = `vize-release-pin-${key}`;
    const body = String(changed.integration.body);
    changed.integration.body = body.replace(
      new RegExp(`(<!-- ${name}: )([^ ]+)( -->)`),
      "$1different$3",
    );
    assert.notEqual(changed.integration.body, body);
    let identity: unknown;
    try {
      identity = integrationIdentity(changed);
    } catch (error) {
      assert.ok(error instanceof Error);
      return;
    }
    assert.notDeepEqual(identity, integrationIdentity(before()));
  });
}

const required = [
  "/integration/id",
  "/integration/number",
  "/integration/merged",
  "/integration/merge_commit_sha",
  "/integration/head/sha",
  "/integration/head/ref",
  "/integration/head/repo/id",
  "/integration/head/repo/full_name",
  "/integration/base/sha",
  "/integration/base/ref",
  "/integration/base/repo/id",
  "/integration/base/repo/full_name",
  "/merge/sha",
  "/merge/commit/tree/sha",
  "/merge/parents",
  "/merge/commit/verification/verified",
  "/merge/commit/verification/reason",
  "/merge/commit/verification/signature",
  "/merge/commit/verification/payload",
  `${check}/id`,
  `${check}/workflow_id`,
  `${check}/path`,
  `${check}/head_sha`,
  `${check}/head_branch`,
  `${check}/event`,
  `${check}/run_attempt`,
  `${check}/status`,
  `${check}/conclusion`,
  `${check}/repository/id`,
  `${check}/repository/full_name`,
  `${check}/head_repository/id`,
  `${check}/head_repository/full_name`,
];
for (const pointer of required) {
  test(`stable integration authority refuses missing or null authority ${pointer}`, () => {
    for (const value of [undefined, null]) {
      const changed = before();
      const { parent, key } = location(changed, pointer);
      if (value === undefined) delete parent[key];
      else parent[key] = value;
      assert.throws(() => integrationIdentity(changed), pointer);
    }
  });
}

test("malformed numeric identities, hashes, parent sets and pin markers cannot form custody authority", () => {
  for (const pointer of [
    "/integration/id",
    "/integration/number",
    `${check}/id`,
    `${check}/workflow_id`,
    `${check}/run_attempt`,
  ]) {
    for (const value of [0, -1, 1.5, 9007199254740992, "1", "01", {}, []]) {
      const changed = before();
      set(changed, pointer, value);
      assert.throws(() => integrationIdentity(changed), `${pointer}: ${JSON.stringify(value)}`);
    }
  }
  for (const pointer of [
    "/integration/head/sha",
    "/merge/sha",
    "/merge/commit/tree/sha",
    `${check}/head_sha`,
  ]) {
    const changed = before();
    set(changed, pointer, "a".repeat(39));
    assert.throws(() => integrationIdentity(changed), pointer);
  }
  for (const value of [[], [{ sha: "invalid" }]]) {
    const changed = before();
    set(changed, "/merge/parents", value);
    assert.throws(() => integrationIdentity(changed));
  }
  for (const key of ["source", "head", "cut", "tag"]) {
    const changed = before();
    const marker = String(changed.integration.body).match(
      new RegExp(`<!-- vize-release-pin-${key}: [^>]+ -->`),
    )![0];
    changed.integration.body = String(changed.integration.body).replace(marker, "");
    assert.throws(() => integrationIdentity(changed));
    changed.integration.body = String(before().integration.body) + "\n" + marker;
    assert.throws(() => integrationIdentity(changed));
  }
});

test("every qualifying exact Check is retained and ordering cannot create a false change", () => {
  const changed = before();
  const original = integrationIdentity(changed);
  changed.gates.workflow_runs.reverse();
  assert.deepEqual(integrationIdentity(changed), original);
  const qualifying = changed.gates.workflow_runs.find((run) => run.event === "push")!;
  changed.gates.workflow_runs.push({ ...structuredClone(qualifying), id: 38808071559 });
  assert.notDeepEqual(integrationIdentity(changed), original);
  changed.gates.workflow_runs.push(structuredClone(qualifying));
  assert.throws(() => integrationIdentity(changed), "duplicate authority runs must be refused");
});

test("an initially latest red exact Check refuses despite an older green run", () => {
  const changed = before();
  const run = structuredClone(changed.gates.workflow_runs.find((value) => value.event === "push")!);
  Object.assign(run, { id: 38808071559, run_attempt: 2, conclusion: "failure" });
  changed.gates.workflow_runs.push(run);
  assert.throws(() => integrationIdentity(changed), /successful latest/);
  const newerGreen = structuredClone(run);
  Object.assign(newerGreen, { id: 38808071561, run_attempt: 1, conclusion: "success" });
  changed.gates.workflow_runs.push(newerGreen);
  const withFailure = integrationIdentity(changed);
  assert.equal(withFailure.checks.find((check) => check.id === run.id)?.conclusion, "failure");
  run.run_attempt = 3;
  assert.notDeepEqual(integrationIdentity(changed), withFailure);
  changed.gates.workflow_runs = changed.gates.workflow_runs.filter((check) => check !== run);
  assert.notDeepEqual(
    integrationIdentity(changed),
    withFailure,
    "a green run must not hide a removed red observation",
  );
});

test("the original Check's red latest attempt refuses when another older run is green", () => {
  const changed = before();
  const original = changed.gates.workflow_runs.find((run) => run.event === "push")!;
  const olderGreen = structuredClone(original);
  olderGreen.id = Number(olderGreen.id) - 1;
  changed.gates.workflow_runs.push(olderGreen);
  Object.assign(original, { run_attempt: 2, conclusion: "failure" });
  assert.throws(() => integrationIdentity(changed), /successful latest/);
  original.conclusion = "success";
  assert.equal(integrationIdentity(changed).checks.at(-1)?.run_attempt, 2);
});

test("retained before and live-after snapshots independently satisfy the original tag's delivery guards", () => {
  const github = JSON.parse(retained.publicationRaw).github;
  for (const raw of [before(), after()]) {
    assert.deepEqual(verifyStoredIntegration(github, retained.identity, raw), raw);
  }
  const mutations = [
    (raw: Raw) => {
      raw.integration.number = 9198;
    },
    (raw: Raw) => {
      set(raw, "/integration/head/sha", "a".repeat(40));
    },
    (raw: Raw) => {
      set(raw, "/merge/parents/0/sha", "a".repeat(40));
    },
    (raw: Raw) => {
      raw.integration.merge_commit_sha = "a".repeat(40);
      raw.merge.sha = "a".repeat(40);
      raw.gates.workflow_runs.find((run) => run.event === "push")!.head_sha = "a".repeat(40);
    },
    (raw: Raw) => {
      raw.integration.body = String(raw.integration.body).replace(
        "vize-release-pin-source: 9196",
        "vize-release-pin-source: 9198",
      );
    },
  ];
  for (const mutate of mutations) {
    const changed = before();
    mutate(changed);
    assert.doesNotThrow(
      () => integrationIdentity(changed),
      "self-consistent projections still need authentic tag guards",
    );
    assert.throws(() => verifyStoredIntegration(github, retained.identity, changed));
  }
});

test("an older red original Check remains bound when a newer Check succeeds", () => {
  const raw = before();
  const original = integrationIdentity(raw);
  const run = raw.gates.workflow_runs.find((value) => value.event === "push")!;
  raw.gates.workflow_runs.push({ ...structuredClone(run), id: 38808071559 });
  Object.assign(run, { run_attempt: 2, conclusion: "failure" });
  const changed = integrationIdentity(raw);
  assert.notDeepEqual(changed, original);
  const retainedOriginal = changed.checks.find((check) => check.id === run.id)!;
  assert.equal(retainedOriginal.run_attempt, 2);
  assert.equal(retainedOriginal.conclusion, "failure");
  raw.gates.workflow_runs = raw.gates.workflow_runs.filter((value) => value !== run);
  assert.notDeepEqual(integrationIdentity(raw), changed);
});

test("tag and source PR references reject malformed, hybrid and unsafe identities without normalization", () => {
  for (const invalid of [
    "",
    "0",
    "09197",
    "9197 ",
    " 9197",
    "9197\n",
    "9197\r\n",
    "+9197",
    "9197.0",
    "9.197e3",
    "9197/9198",
    "9197abc",
    "９１９７",
    "9007199254740992",
  ]) {
    assert.throws(() => integrationReference(`#${invalid}`, invalid), invalid);
    assert.throws(() => integrationReference(`#${invalid}`, "9197"), invalid);
    assert.throws(() => integrationReference("#9197", invalid), invalid);
  }
  for (const source of [9197, null, {}, ["9197"], "#9197"])
    assert.throws(() => integrationReference("#9197", source));
  for (const tag of [9197, null, {}, ["#9197"], "9197", "##9197", "#9198"])
    assert.throws(() => integrationReference(tag, "9197"));
});

function policyRepository(check: (directory: string, git: (...args: string[]) => string) => void) {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-recovery-policy-"));
  const git = (...args: string[]) =>
    execFileSync("git", ["-C", directory, ...args], { encoding: "utf8", stdio: "pipe" });
  try {
    git("init", "--quiet");
    git("config", "core.hooksPath", path.join(directory, "no-hooks"));
    git("config", "commit.gpgSign", "false");
    git("config", "user.name", "Recovery fixture");
    git("config", "user.email", "recovery@example.invalid");
    for (const file of policyFiles) {
      const target = path.join(directory, file);
      mkdirSync(path.dirname(target), { recursive: true });
      writeFileSync(
        target,
        file.endsWith("/oracles.ts")
          ? readFileSync(new URL(`../../${file}`, import.meta.url))
          : `import { writeFileSync } from "node:fs";\nwriteFileSync(${JSON.stringify(path.join(directory, "executed"))}, "executed");\nthrow new Error("Historical policy must never execute: ${file}");\n`,
      );
    }
    check(directory, git);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("real historical Git blobs remain authoritative after actor drift; producing checkout stays strict", () => {
  policyRepository((directory, git) => {
    const expected = policyFiles.map((file) => ({
      file,
      sha256: sha256(readFileSync(path.join(directory, file))),
    }));
    git("add", ".");
    git("commit", "--quiet", "-m", "Original policy");
    const original = git("rev-parse", "HEAD").trim();
    const read = (sha: string, file: string) =>
      git("--no-replace-objects", "show", `${sha}:${file}`);
    const checkout = (file: string) => readFileSync(path.join(directory, file));
    assert.deepEqual(sourceAuthority(original, read), expected);
    assertCheckoutAuthority(expected, checkout);
    const actor = "tools/support/release/public_acceptance/lsp_source.ts";
    writeFileSync(path.join(directory, actor), "throw new Error('Different current actor');\n");
    assert.throws(() => assertCheckoutAuthority(expected, checkout), /checkout differs/);
    git("commit", "--all", "--quiet", "-m", "Reviewed actor successor");
    const current = sourceAuthority(git("rev-parse", "HEAD").trim(), read);
    assert.deepEqual(
      current,
      expected.map((entry) =>
        entry.file === actor ? { file: actor, sha256: sha256(checkout(actor)) } : entry,
      ),
    );
    assertCheckoutAuthority(current, checkout);
    assert.deepEqual(sourceAuthority(original, read), expected);
    assert.equal(existsSync(path.join(directory, "executed")), false);
  });
});

test("real Git policies with incomplete, substituted or unsupported inventories cannot authorize evidence", () => {
  policyRepository((directory, git) => {
    const file = "tools/support/release/consumer_recovery/oracles.ts";
    const original = readFileSync(path.join(directory, file), "utf8");
    const first = `  ${JSON.stringify(policyFiles[0])},\n`;
    const read = (sha: string, name: string) =>
      git("--no-replace-objects", "show", `${sha}:${name}`);
    for (const changed of [
      original.replace(first, ""),
      original.replace(first, first + first),
      original.replace(first, '  "foreign/authority.ts",\n'),
      original.replace("export const policyFiles = [", "export const policyFiles = compute(["),
      original.replace(first, `  ${JSON.stringify(policyFiles[1])},\n`),
    ]) {
      writeFileSync(path.join(directory, file), changed);
      git("add", ".");
      git("commit", "--quiet", "-m", "Unsupported policy");
      assert.throws(
        () => sourceAuthority(git("rev-parse", "HEAD").trim(), read),
        /unsupported historical/,
      );
    }
    writeFileSync(path.join(directory, file), original);
    rmSync(path.join(directory, policyFiles[0]));
    git("commit", "--all", "--quiet", "-m", "Missing required blob");
    assert.throws(() => sourceAuthority(git("rev-parse", "HEAD").trim(), read));
    assert.throws(() => sourceAuthority("HEAD", read));
  });
});
