import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  CONFIG_BYTES,
  CONFIG_NAMES,
  CONFIG_PINS,
  PARSER_HASH,
  PROFILES,
  SOURCE_PINS,
  SUITE_HASH,
  extractCliDiagnostics,
  loadUpstream,
  machineDiagnostics,
  relativeFiles,
  sha256,
} from "../../tools/benchmarks/scripts/vue-benchmarks-current-lint-contract.mjs";
import { probeCurrentLint } from "../../tools/benchmarks/scripts/vue-benchmarks-current-lint.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));

test("pinned original eleven-plant judge retains all attribution and clean-twin refusals", async () => {
  const upstream = await loadUpstream(root);
  assert.equal(upstream.LINT_VALIDITY_SUITE_HASH, SUITE_HASH);
  const plants = upstream.LINT_VALIDITY_PLANTS;
  const files = relativeFiles(plants);
  for (const [index, plant] of plants.entries()) {
    const good = {
      file: files[index],
      line: plant.dirtyLine,
      rule: plant.rules[0],
      message: "authored positive authority",
    };
    assert.equal(upstream.judgeLintPair(plant, [good], [], files[index]).ok, true);
    for (const bad of [
      { ...good, file: "AnotherPlant.vue" },
      { ...good, file: `[${files[index]}` },
      { ...good, line: plant.dirtyLine + 1 },
      { ...good, rule: "unrelated/rule", message: "unrelated diagnostic" },
    ]) {
      assert.equal(upstream.judgeLintPair(plant, [bad], [], files[index]).ok, false);
    }
    assert.equal(
      upstream.judgeLintPair(plant, [good], [{ ...good, line: plant.dirtyLine + 1 }], files[index])
        .ok,
      false,
    );
    assert.ok(plant.dirty.split("\n")[plant.dirtyLine - 1].length > 0);
    assert.notEqual(sha256(plant.dirty), sha256(plant.clean));
  }
  const source = upstream.custody.sources["scripts/lib/lint-validity-child.mjs"];
  assert.equal(sha256(source), SOURCE_PINS["scripts/lib/lint-validity-child.mjs"]);
  assert.equal(sha256(extractCliDiagnostics(source)), PARSER_HASH);
  assert.throws(() =>
    extractCliDiagnostics(
      source.replace("function cliDiagnostics(raw) {", "function cliDiagnostics(output) {"),
    ),
  );
  assert.throws(() =>
    extractCliDiagnostics(source.replace("const diagnostics = [];", "const diagnostics = [null];")),
  );
  // Authored grammar example; this is not represented as observed CLI output.
  const graphical = ` × [vize:vue/no-v-html] 'v-html' can lead to XSS attack.\n ╭─[${files[0]}:6:8]\n`;
  const parsed = upstream.cliDiagnostics(graphical);
  assert.equal(parsed.length, 1);
  assert.equal(parsed[0].file, `[${files[0]}`);
  assert.equal(upstream.judgeLintPair(plants[0], parsed, [], files[0]).ok, false);
  assert.deepEqual(CONFIG_NAMES.slice().sort(), Object.keys(CONFIG_BYTES).sort());
  for (const file of CONFIG_NAMES) assert.equal(sha256(CONFIG_BYTES[file]), CONFIG_PINS[file]);
});

test("supplemental machine decoding preserves strict counts, complete ranges and identities", () => {
  const row = {
    file: "nested/00/v-html/Plant.vue",
    errorCount: 0,
    warningCount: 1,
    messages: [
      {
        ruleId: "vue/no-v-html",
        severity: 1,
        message: "authored diagnostic",
        line: 6,
        column: 8,
        endLine: 6,
        endColumn: 14,
      },
    ],
  };
  assert.deepEqual(machineDiagnostics([row]), [
    {
      file: row.file,
      line: 6,
      column: 8,
      rule: "vue/no-v-html",
      message: "authored diagnostic",
      raw: JSON.stringify(row.messages[0]),
    },
  ]);
  assert.throws(() => machineDiagnostics([row, row]));
  assert.throws(() => machineDiagnostics([{ ...row, warningCount: 0 }]));
  assert.throws(() =>
    machineDiagnostics([{ ...row, messages: [{ ...row.messages[0], endColumn: 7 }] }]),
  );
  assert.throws(() =>
    machineDiagnostics([{ ...row, messages: [{ ...row.messages[0], severity: 0 }] }]),
  );
});

test("current receipted CLI records both untouched eleven-pair judges for both thread profiles", async (t) => {
  const observed = await probeCurrentLint(root);
  assert.equal(observed.status, "OBSERVED");
  assert.equal(observed.ranked, false);
  assert.equal(observed.nativeMigrationCredit, 0);
  assert.equal(observed.plants.length, 11);
  assert.deepEqual(
    observed.profiles.map((profile) => profile.profile),
    PROFILES,
  );
  assert.equal(observed.attempts.length, 9);
  assert.equal(observed.configs.length, 4);
  const destination = path.join(root, "target/differential/vue-benchmarks-current-lint");
  for (const attempt of observed.attempts) {
    assert.equal(attempt.error, null);
    assert.equal(attempt.signal, null);
    for (const stream of Object.values(attempt.streams)) {
      const bytes = fs.readFileSync(path.join(destination, stream.file));
      assert.equal(bytes.length, stream.bytes);
      assert.equal(sha256(bytes), stream.sha256);
    }
  }
  for (const profile of observed.profiles) {
    assert.equal(profile.runs.length, 4);
    assert.equal(profile.human.length, 11);
    assert.equal(profile.machine.length, 11);
    assert.deepEqual(
      profile.human.map((row) => row.id),
      observed.plants.map((row) => row.id),
    );
    assert.deepEqual(
      profile.machine.map((row) => row.id),
      observed.plants.map((row) => row.id),
    );
    for (const polarity of ["dirty", "clean"]) {
      const human = profile.runs.find(
        (run) => run.polarity === polarity && run.reporter === "human",
      );
      const machine = profile.runs.find(
        (run) => run.polarity === polarity && run.reporter === "machine",
      );
      assert.deepEqual(machine.inventory, human.inventory);
      assert.equal(machine.cwd, human.cwd);
      assert.deepEqual(human.args, ["lint", "."]);
      assert.deepEqual(machine.args, ["lint", ".", "--format", "json"]);
      assert.ok(Array.isArray(machine.completeRows));
    }
    t.diagnostic(
      `${profile.profile}: original human judge ${profile.human.filter((row) => row.ok).length}/11; supplemental machine judge ${profile.machine.filter((row) => row.ok).length}/11; UNRANKED`,
    );
  }
});
