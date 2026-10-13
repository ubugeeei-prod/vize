import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  expectedEntries,
  verdict,
} from "../../tools/commands/ci/github/allow-armature-tokenizer-reexports.ts";

const tool = fileURLToPath(
  new URL("../../tools/commands/ci/github/allow-armature-tokenizer-reexports.ts", import.meta.url),
);
const summary =
  "     Summary semver requires new major version: 6 major and 0 minor checks failed\n";
const accepted =
  "Only the documented vize_armature tokenizer cross-crate re-export false positives remain.";

/** One diagnostic line in the shape cargo-semver-checks 0.47.0 prints. */
function diagnostic(lint: string, entry: string): string {
  if (lint === "pub_module_level_const_missing") {
    return `  ${entry} in file /work/crates/vize_armature/src/tokenizer/char_codes.rs:12`;
  }
  const file = lint === "module_missing" ? "tokenizer/char_codes.rs" : "tokenizer/types.rs";
  return `  ${entry}, previously in file /work/crates/vize_armature/src/${file}:7`;
}

type Sections = ReadonlyMap<string, readonly string[]>;

function reviewedSections(): Map<string, string[]> {
  const sections = new Map<string, string[]>();
  for (const [lint, counter] of expectedEntries()) {
    const entries: string[] = [];
    for (const [entry, total] of counter) {
      for (let index = 0; index < total; index += 1) entries.push(diagnostic(lint, entry));
    }
    sections.set(lint, entries);
  }
  return sections;
}

function render(sections: Sections, tail = summary): string {
  let log = "    Checking vize_armature v0.449.0 -> v0.449.0 (no change)\n";
  for (const [lint, entries] of sections) {
    log += `\n--- failure ${lint}: an item is missing ---\n\n`;
    log += `Description:\nA publicly-visible item cannot be imported by its prior path.\n`;
    log += `        ref: https://doc.rust-lang.org/cargo/reference/semver.html\n\n`;
    log += `Failed in:\n${entries.join("\n")}\n`;
  }
  return `${log}\n${tail}`;
}

void test("the reviewed tokenizer re-export false positives are the only accepted report", () => {
  const log = render(reviewedSections());
  assert.deepEqual(verdict(log), { status: 0, stdout: [accepted], stderr: [] });
  const colored = log.replaceAll("--- failure", "\u001b[1m\u001b[31m--- failure\u001b[0m");
  assert.equal(verdict(colored).status, 0, "terminal colors are stripped before matching");
  const tokenizerFile = log.replaceAll("tokenizer/types.rs:7", "tokenizer.rs:301");
  assert.equal(verdict(tokenizerFile).status, 0, "items may come from tokenizer.rs itself");
  const windows = log.replaceAll("\n", "\r\n");
  assert.equal(verdict(windows).status, 0, "line endings are read as text");
});

void test("a missing, added or repeated diagnostic refuses the allowance", () => {
  const missing = reviewedSections();
  missing.get("enum_missing")?.pop();
  assert.deepEqual(verdict(render(missing)).stderr, [
    'enum_missing: unexpected={}, missing={"enum vize_armature::QuoteType":1}',
  ]);

  const added = reviewedSections();
  added.get("struct_missing")?.push(diagnostic("struct_missing", "struct vize_armature::Other"));
  assert.deepEqual(verdict(render(added)).stderr, [
    'struct_missing: unexpected={"struct vize_armature::Other":1}, missing={}',
  ]);

  const repeated = reviewedSections();
  repeated
    .get("pub_module_level_const_missing")
    ?.push(diagnostic("pub_module_level_const_missing", "TAB"));
  assert.deepEqual(verdict(render(repeated)).stderr, [
    'pub_module_level_const_missing: unexpected={"TAB":1}, missing={}',
  ]);

  const otherLint = reviewedSections();
  otherLint.set("function_parameter_count_changed", []);
  const refused = verdict(render(otherLint));
  assert.equal(refused.status, 1, "an unreviewed lint is refused even without entries");
  assert.deepEqual(refused.stdout, []);
});

void test("an unexpected summary, tool error or diagnostic shape refuses the allowance", () => {
  const sections = reviewedSections();
  const sevenMajor = summary.replace("6 major", "7 major");
  assert.deepEqual(verdict(render(sections, sevenMajor)).stderr, ["unexpected SemVer summary"]);
  assert.deepEqual(verdict(render(sections, `${summary}\nerror: could not compile\n`)).stderr, [
    "SemVer command also reported a tool/build error",
  ]);
  assert.deepEqual(verdict(render(sections, `${summary}\npanicked at src/main.rs\n`)).stderr, [
    "SemVer command also reported a tool/build error",
  ]);

  const elsewhere = reviewedSections();
  elsewhere.set("trait_missing", [
    "  trait vize_armature::Callbacks, previously in file /work/src/parser.rs:9",
  ]);
  assert.deepEqual(verdict(render(elsewhere)).stderr, [
    "unexpected item diagnostic: trait vize_armature::Callbacks, previously in file /work/src/parser.rs:9",
  ]);

  const unindented = reviewedSections();
  unindented.get("module_missing")?.push("mod vize_armature::other");
  assert.deepEqual(verdict(render(unindented)).stderr, [
    "unexpected diagnostic line: mod vize_armature::other",
  ]);

  const duplicate = render(reviewedSections()).replace(
    summary,
    `--- failure enum_missing: again ---\n\nFailed in:\n  enum x, previously in file /a/src/tokenizer.rs:1\n\n${summary}`,
  );
  assert.deepEqual(verdict(duplicate).stderr, ["missing evidence or duplicate lint: enum_missing"]);

  const noEvidence = render(reviewedSections()).replace("Failed in:\n", "Failed at:\n");
  assert.deepEqual(verdict(noEvidence).stderr, [
    "missing evidence or duplicate lint: enum_missing",
  ]);
});

void test("the command prints its verdict and exits with it", () => {
  const directory = mkdtempSync(join(tmpdir(), "armature-semver-allowance-"));
  try {
    const run = (name: string, log: string) => {
      const file = join(directory, name);
      writeFileSync(file, log);
      return spawnSync(process.execPath, [tool, file], { encoding: "utf8" });
    };
    const passed = run("accepted.log", render(reviewedSections()));
    assert.equal(passed.status, 0, passed.stderr);
    assert.equal(passed.stdout, `${accepted}\n`);
    const refused = run("refused.log", "unrelated output\n");
    assert.equal(refused.status, 1);
    assert.equal(refused.stdout, "");
    assert.equal(refused.stderr, "unexpected SemVer summary\n");
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
