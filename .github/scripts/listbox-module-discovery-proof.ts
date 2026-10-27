import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const base = "b6127a994f7e2a75eb2c89de44d54cf8a0f17fcd";
const head = process.env.GITHUB_SHA;
assert.ok(head);
const sourceHead = process.env.EXPECTED_SOURCE_HEAD;
assert.ok(sourceHead);
const output = path.join(process.env.RUNNER_TEMP ?? "/tmp", "listbox-module-layout.json");
const category = "crates/vize_patina/src/rules/a11y.rs";
const rule = "crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs";
const tests = "crates/vize_patina/src/rules/a11y/interactive_supports_focus_tests.rs";
const gate = "tools/commands/ci/source-file-lengths.rs";
type ProcessPacket = {
  argv: string[];
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
};
const processes: ProcessPacket[] = [];
const evidence: Record<string, unknown> = { base, head, sourceHead, processes };
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
function run(argv: string[], retain = true): ProcessPacket {
  const executable = argv[0];
  assert.ok(executable);
  const child = spawnSync(executable, argv.slice(1), {
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, SOURCE_LENGTH_BASE_REF: base },
  });
  const packet = {
    argv,
    status: child.status,
    signal: child.signal,
    error: child.error?.message ?? null,
    stdout: child.stdout,
    stderr: child.stderr,
  };
  if (retain) processes.push(packet);
  assert.equal(packet.error, null, argv.join(" "));
  assert.equal(packet.signal, null, argv.join(" "));
  return packet;
}
function git(...args: string[]): string {
  const packet = run(["git", ...args], false);
  assert.equal(packet.status, 0, packet.stderr);
  return packet.stdout;
}
function source(ref: string, file: string): string {
  return git("show", ref + ":" + file);
}
try {
  assert.equal(git("rev-parse", "HEAD").trim(), head);
  assert.equal(git("rev-parse", "HEAD^").trim(), sourceHead);
  for (const file of [category, rule, tests])
    assert.equal(source(sourceHead, file), source(head, file));
  assert.deepEqual(
    git("diff", "--name-only", base, sourceHead).trim().split("\n").sort(),
    [category, rule, tests].sort(),
  );
  const allowed = [
    category,
    rule,
    tests,
    ".github/scripts/listbox-module-discovery-proof.ts",
    ".github/workflows/listbox-module-discovery.yml",
  ].sort();
  assert.deepEqual(git("diff", "--name-only", base, head).trim().split("\n").sort(), allowed);
  const beforeRule = source(base, rule);
  const afterRule = source(head, rule);
  const marker = "\n#[cfg(test)]\n";
  const production = beforeRule.split(marker)[0];
  assert.ok(production);
  assert.equal(afterRule.split(marker)[0], production);
  assert.equal(
    hash(production),
    "7e367876a980eaf299d76847463648951a164e9ec7452f52b837a6a12f6869b0",
  );
  const oldTail =
    '#[cfg(test)]\n#[path = "interactive_supports_focus_tests.rs"]\nmod listbox_tests;\n';
  const newTail =
    "#[cfg(test)]\npub(super) fn create_legacy_linter_for_test() -> crate::linter::Linter {\n    tests::create_legacy_linter()\n}\n";
  assert.equal(afterRule, beforeRule.replace(oldTail, newTail));
  assert.equal(
    source(head, category),
    source(base, category) + "\n#[cfg(test)]\nmod interactive_supports_focus_tests;\n",
  );
  assert.equal(
    source(head, tests),
    source(base, tests).replace(
      "super::tests::create_legacy_linter()",
      "super::interactive_supports_focus::create_legacy_linter_for_test()",
    ),
  );
  evidence.runtimeRulePrefixSha256 = hash(production);
  evidence.existingInlineTestsUnchanged = true;
  evidence.wholePacketControllerUnchangedExceptHelperLookup = true;
  const protectedPaths = [
    gate,
    "tests/tooling/source-file-lengths.test.ts",
    "tests/tooling/davinci-module-layout.test.ts",
    "tests/tooling/rule-parity-plain-types.test.ts",
    "crates/vize_patina/src/markup/differential/battery.rs",
    "tests/tooling/lint-listbox-focus-10925.test.ts",
    "tests/_fixtures/differential/linter/a11y-listbox-focus-10925",
    "docs/content",
  ];
  assert.equal(run(["git", "diff", "--exit-code", base, head, "--", ...protectedPaths]).status, 0);
  evidence.gateSha256 = hash(source(base, gate));
  assert.equal(source(base, gate), source(head, gate));
  assert.equal(run(["git", "checkout", "--detach", base]).status, 0);
  const before = run(["rust-script", gate, "--check", "--base-ref", base, "--limit", "5"]);
  assert.equal(before.status, 1);
  assert.match(before.stdout + before.stderr, /Rust modules must use ordinary module discovery/);
  assert.match(before.stdout + before.stderr, /interactive_supports_focus\.rs:478: path attribute/);
  evidence.beforeGate = before;
  assert.equal(run(["git", "checkout", "--detach", head]).status, 0);
  const after = run(["rust-script", gate, "--check", "--base-ref", base, "--limit", "5"]);
  assert.equal(after.status, 0, after.stdout + after.stderr);
  evidence.afterGate = after;
  const contracts = run([
    process.execPath,
    "--test",
    "tests/tooling/source-file-lengths.test.ts",
    "tests/tooling/davinci-module-layout.test.ts",
    "tests/tooling/rule-parity-plain-types.test.ts",
  ]);
  assert.equal(contracts.status, 0, contracts.stdout + contracts.stderr);
  evidence.metadataContracts = contracts;
  evidence.status = "passed";
} catch (error) {
  evidence.status = "failed";
  evidence.error =
    error instanceof Error ? { message: error.message, stack: error.stack } : String(error);
  process.exitCode = 1;
} finally {
  const restored = run(["git", "checkout", "--detach", head]);
  assert.equal(restored.status, 0);
  fs.writeFileSync(output, JSON.stringify(evidence, null, 2) + "\n");
  console.log(
    JSON.stringify({
      status: evidence.status,
      base,
      head,
      output,
      beforeGate: (evidence.beforeGate as ProcessPacket | undefined)?.status,
      afterGate: (evidence.afterGate as ProcessPacket | undefined)?.status,
      runtimeRulePrefixSha256: evidence.runtimeRulePrefixSha256,
      gateSha256: evidence.gateSha256,
    }),
  );
}
