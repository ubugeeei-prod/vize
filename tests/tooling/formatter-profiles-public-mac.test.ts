import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  macRole,
  macOriginal,
  reportSha,
  assertMacHost,
  validateDispatch,
} from "../../tools/support/release/formatter_profiles_public/mac_contract.ts";
import {
  assertMacInputs,
  macInstallArgs,
  macRootSeal,
  resetMacInput,
} from "../../tools/support/release/formatter_profiles_public/mac_install.ts";
import {
  assertMacFiles,
  macJournalExit,
  macObservation,
  macVerdicts,
} from "../../tools/support/release/formatter_profiles_public/mac_run.ts";
import {
  macReference,
  assertFullFalse,
} from "../../tools/support/release/formatter_profiles_public/mac_reference.ts";
import { inventory } from "../../tools/support/release/editor_vite_stub/filesystem.ts";
import { Recorder, sha256 } from "../../tools/support/config/windows_consumer/common.ts";

const valid = {
  version: "0.628.0",
  producer: "123",
  authority: "1".repeat(40),
  beforeProducer: "456",
  beforeAuthority: "2".repeat(40),
};
test("the Mac role physically consumes the complete initial11264 report and exact original joint argv", () => {
  const original = macOriginal();
  assert.equal(reportSha, "e40736f2951cab4ee3617aaa00930429adaeb287c826d7d9487a72015e02c8cd");
  assert.equal(Buffer.byteLength(original.source), 219);
  assert.equal(
    sha256(original.source),
    "cbe837ee3a56dd7df80ed2c301295be38205faaf5f728c43f18b5c6670ead730",
  );
  assert.equal(
    original.source,
    readFileSync(
      new URL(
        "../_fixtures/differential/formatter-regressions/space-separated-booleans-11264/Original.vue.txt",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  assert.deepEqual(original.equals, [
    "fmt",
    "a.vue",
    "--no-config",
    "--write",
    "--semi=false",
    "--bracket-spacing=false",
    "--sort-attributes=false",
  ]);
  assert.deepEqual(original.spaced, [
    "fmt",
    "a.vue",
    "--no-config",
    "--write",
    "--semi",
    "false",
    "--bracket-spacing",
    "false",
    "--sort-attributes",
    "false",
  ]);
});

test("closed dispatch validation rejects unknown role and missing Mac tuple before authority access", () => {
  validateDispatch("workflow_dispatch", "refs/heads/main", macRole, valid);
  validateDispatch("workflow_dispatch", "refs/heads/main", "linux-node24", {
    version: "0.628.0",
    producer: "123",
    authority: "1".repeat(40),
  });
  validateDispatch("pull_request", "refs/pull/1/merge", "linux-node24", {});
  for (const role of ["", "mac", "linux", "unknown"])
    assert.throws(() => validateDispatch("workflow_dispatch", "refs/heads/main", role, {}));
  for (const key of Object.keys(valid))
    assert.throws(
      () =>
        validateDispatch("workflow_dispatch", "refs/heads/main", macRole, { ...valid, [key]: "" }),
      key,
    );
  assert.throws(() =>
    validateDispatch("workflow_dispatch", "refs/heads/main", macRole, {
      ...valid,
      version: "0.627.0",
    }),
  );
  assert.throws(() => validateDispatch("workflow_dispatch", "refs/heads/private", macRole, valid));
  assert.throws(() => validateDispatch("push", "refs/heads/main", macRole, valid));
});

test("original npm default lifecycle installation and exact host cannot inherit the Linux role", () => {
  assert.deepEqual(macInstallArgs("0.627.0"), ["install", "vize@0.627.0"]);
  assert.deepEqual(macInstallArgs("0.628.0"), ["install", "vize@0.628.0"]);
  assertMacHost("darwin", "arm64", "v25.8.1");
  for (const host of [
    ["linux", "arm64", "v25.8.1"],
    ["darwin", "x64", "v25.8.1"],
    ["darwin", "arm64", "v24.14.0"],
  ])
    assert.throws(() => assertMacHost(host[0]!, host[1]!, host[2]!));
});

test("original help and version preserve Rust clap exits while successful fmt returns to Node", () => {
  assert.equal(macJournalExit("original-help"), null);
  assert.equal(macJournalExit("original-version"), null);
  for (const label of ["equals-original", "spaced-rewrite", "true-control-check"])
    assert.equal(macJournalExit(label), 0);
});

test("per-command and whole reports never serialize the private child environment or unknown secrets", () => {
  const temporary = mkdtempSync(join(tmpdir(), "vize-mac-custody-canary-"));
  const token = "synthetic-private-github-canary",
    unknown = "synthetic-unknown-custody-canary";
  const privateCustody = {
    journal: "/controlled/original.native.ndjson",
    env: {
      GITHUB_TOKEN: token,
      UNKNOWN_SECRET: unknown,
      ENCODED_SECRET: Buffer.from(unknown).toString("base64"),
    },
    extraPrivateCarrier: { token, nested: [unknown] },
  };
  const input = {
    label: "original",
    args: ["fmt", "a.vue", "--no-config"],
    packet: {
      label: "original",
      argv: ["node", "cli", "fmt", "a.vue", "--no-config"],
      cwd: "/controlled",
      status: 0,
      signal: null,
      error: null,
      stdout: "complete original stdout\n",
      stderr: "complete original stderr\n",
      durationMs: 3,
    },
    before: [],
    after: [],
    outputRawBase64: Buffer.from("complete product output").toString("base64"),
    custody: privateCustody,
    writable: ["a.vue"],
    extraUnknownSecret: unknown,
  };
  try {
    const row = macObservation(input);
    assert.deepEqual(row, {
      label: input.label,
      args: input.args,
      packet: input.packet,
      before: input.before,
      after: input.after,
      outputRawBase64: input.outputRawBase64,
      custody: { journal: privateCustody.journal },
      writable: input.writable,
    });
    const record = new Recorder(temporary);
    record.json("original.observation.json", row);
    record.json("whole-observations.json", {
      rows: [row],
      outcomes: [{ name: "equals", rawBase64: row.outputRawBase64 }],
    });
    const forbidden = [
      token,
      unknown,
      Buffer.from(token).toString("base64"),
      Buffer.from(unknown).toString("base64"),
    ];
    function inspect(value: unknown): void {
      if (typeof value === "string")
        for (const canary of forbidden) assert(!value.includes(canary));
      else if (Array.isArray(value)) for (const item of value) inspect(item);
      else if (value !== null && typeof value === "object")
        for (const [key, item] of Object.entries(value)) {
          assert(
            ![
              "env",
              "GITHUB_TOKEN",
              "UNKNOWN_SECRET",
              "ENCODED_SECRET",
              "extraPrivateCarrier",
              "extraUnknownSecret",
            ].includes(key),
          );
          inspect(item);
        }
    }
    for (const name of ["original.observation.json", "whole-observations.json"]) {
      const raw = readFileSync(join(temporary, name), "utf8");
      for (const canary of forbidden) assert(!raw.includes(canary));
      inspect(JSON.parse(raw));
    }
    assert.throws(
      () => inspect(JSON.parse(JSON.stringify({ ...row, custody: privateCustody }))),
      "the former full-carrier serialization must fail the recursive privacy guard",
    );
    assert.equal(
      privateCustody.env.GITHUB_TOKEN,
      token,
      "the original child execution environment remains intact",
    );
    assert.equal(privateCustody.env.UNKNOWN_SECRET, unknown);
  } finally {
    rmSync(temporary, { recursive: true });
  }
});

test("whole source reference rejects false agreement assembled from every genuinely observed wrong alternative", () => {
  const reference = macReference();
  assert.equal(reference.packets.length, 30);
  assert.equal(
    sha256(reference.output),
    "baa8e7b1a02bfd36727815527ebc96435f31d281db7dfcdfc0fc3bfa054a4662",
  );
  assertFullFalse(reference.output, reference.output);
  // These are validator falsifications built from genuine full source outputs,
  // not claims that a public version produced these synthetic paired results.
  for (const wrong of reference.alternatives) {
    const bothFalseForms = [wrong, Buffer.from(wrong)];
    assert.deepEqual(bothFalseForms[0], bothFalseForms[1]);
    assert.throws(() => assertFullFalse(bothFalseForms[0]!, reference.output));
    assert.throws(() => assertFullFalse(bothFalseForms[1]!, reference.output));
  }
});

test("a failed old formatting verdict cannot suppress containing-release evaluation", () => {
  const visited: string[] = [];
  const verdicts = macVerdicts([
    {
      version: "0.627.0",
      evaluate: () => {
        visited.push("before");
        throw new Error("observed old mismatch");
      },
    },
    {
      version: "0.628.0",
      evaluate: () => {
        visited.push("after");
        return "whole reference matched";
      },
    },
  ]);
  assert.deepEqual(visited, ["before", "after"]);
  assert.deepEqual(
    verdicts.map((row) => row.state),
    ["failed", "passed"],
  );
  assert.throws(() => macVerdicts([]));
});

test("owned fixture resets refuse foreign links and changed roots before any write or command reuse", () => {
  const temporary = realpathSync(mkdtempSync(join(tmpdir(), "vize-formatter-mac-reset-")));
  const root = join(temporary, "consumer"),
    foreign = join(temporary, "foreign");
  try {
    mkdirSync(root);
    mkdirSync(foreign);
    for (const name of ["a.vue", "Unselected.vue", "vite.config.mjs"])
      writeFileSync(join(root, name), "long original");
    const seal = macRootSeal(root);
    resetMacInput(seal, "short");
    assert.equal(readFileSync(join(root, "a.vue"), "utf8"), "short");
    const outside = join(foreign, "outside.vue");
    writeFileSync(outside, "foreign immutable");
    rmSync(join(root, "a.vue"));
    symlinkSync(outside, join(root, "a.vue"));
    assert.throws(() => assertMacInputs(seal));
    assert.throws(() => resetMacInput(seal, "must not write"));
    assert.equal(readFileSync(outside, "utf8"), "foreign immutable");
    rmSync(join(root, "a.vue"));
    writeFileSync(join(root, "a.vue"), "owned");
    renameSync(root, root + "-saved");
    symlinkSync(foreign, root);
    assert.throws(() => assertMacInputs(seal));
    assert.throws(() => resetMacInput(seal, "must not write"));
    assert.equal(readFileSync(outside, "utf8"), "foreign immutable");
  } finally {
    rmSync(temporary, { recursive: true });
  }
});

test("whole Mac filesystem guard rejects unselected, unrelated module, extra file and link mutation", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-formatter-mac-pure-"));
  try {
    mkdirSync(join(root, "node_modules"));
    for (const name of ["a.vue", "Unselected.vue", "node_modules/foreign.js", "package-lock.json"])
      writeFileSync(join(root, name), "original");
    const baseline = inventory(root);
    assertMacFiles(baseline, structuredClone(baseline), []);
    writeFileSync(join(root, "a.vue"), "formatted");
    assertMacFiles(baseline, inventory(root), ["a.vue"]);
    assert.throws(() => assertMacFiles(baseline, inventory(root), []));
    for (const name of ["Unselected.vue", "node_modules/foreign.js", "package-lock.json"]) {
      writeFileSync(join(root, name), "changed");
      assert.throws(() => assertMacFiles(baseline, inventory(root), ["a.vue"]));
      writeFileSync(join(root, name), "original");
    }
    writeFileSync(join(root, "extra"), "new");
    assert.throws(() => assertMacFiles(baseline, inventory(root), ["a.vue"]));
    const alias = structuredClone(baseline);
    const row = alias.find((row) => row.path === "a.vue")!;
    row.kind = "link";
    row.target = "/foreign";
    assert.throws(() => assertMacFiles(baseline, alias, ["a.vue"]));
  } finally {
    rmSync(root, { recursive: true });
  }
});

test("workflow controls validate before routing and never expand the public job into a matrix", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/formatter-profiles-public.yml", import.meta.url),
    "utf8",
  );
  assert(workflow.includes("mac_contract.ts --validate-dispatch"));
  assert(workflow.includes("node --test tests/tooling/formatter-profiles-public-mac.test.ts"));
  assert(workflow.includes("runs-on: macos-15"));
  assert(workflow.includes('node-version: "25.8.1"'));
  assert(!workflow.includes("matrix:"));
  assert.equal(workflow.split("mac_run.ts").length, 2);
  assert(workflow.includes("inputs.role == 'linux-node24' || inputs.role == ''"));
  assert(workflow.includes("inputs.role == 'mac-arm64-node25-11264'"));
});
