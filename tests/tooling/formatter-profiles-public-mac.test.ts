import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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
import { macInstallArgs } from "../../tools/support/release/formatter_profiles_public/mac_install.ts";
import {
  assertMacFiles,
  macJournalExit,
  macVerdicts,
} from "../../tools/support/release/formatter_profiles_public/mac_run.ts";
import {
  macReference,
  assertFullFalse,
} from "../../tools/support/release/formatter_profiles_public/mac_reference.ts";
import { inventory } from "../../tools/support/release/editor_vite_stub/filesystem.ts";
import { sha256 } from "../../tools/support/config/windows_consumer/common.ts";

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
