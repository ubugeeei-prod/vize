import assert from "node:assert/strict";
import { test } from "node:test";
import { normalizeParserRecovery } from "../../tools/benchmarks/scripts/type-snapshot-parser-recovery.ts";
import { normalizeTypecheckResult } from "../../tools/benchmarks/scripts/typecheck-command.mjs";

const warning =
  "\u001b[33mwarning:\u001b[0m corsa: Corsa CLI fast path unavailable; degraded to the slower project-session API (check failure). Type checking continues on a slower path.";
const recovered = {
  status: 1,
  stderr: `${warning}\nerror handling method 'exit': EOF\n`,
  stdout: JSON.stringify({
    files: [
      {
        file: "Comp0.vue",
        diagnostics: ["error:1:59 [TS1260] Keywords cannot contain escape characters."],
      },
      {
        file: "shared.ts",
        diagnostics: ["error:1:21 [TS2304] Cannot find name 'LeafGlobal'."],
      },
    ],
    errorCount: 2,
    warningCount: 0,
  }),
};

test("intentional malformed parity retains syntax and semantic diagnostics and raw stderr", () => {
  for (const phase of ["parity", "parity-profile"]) {
    const result = normalizeParserRecovery(recovered, true, phase);
    assert.equal(result.stdout, recovered.stdout);
    assert.equal(result.stderr, "");
    assert.match(recovered.stderr, /EOF/u);
    const normalized = normalizeTypecheckResult(result, process.cwd(), "json");
    assert.equal(normalized.diagnostics.length, 2);
    assert.equal(normalized.status, 1);
  }
});

test("successful session shutdown does not require the optional EOF notice", () => {
  assert.equal(
    normalizeParserRecovery({ ...recovered, stderr: `${warning}\n` }, true, "parity").stderr,
    "",
  );
});

test("a completed syntax-error report accepts the native context-cancelled shutdown variant", () => {
  const raw = { ...recovered, stderr: `${warning}\ncontext canceled\n` };
  const result = normalizeParserRecovery(raw, true, "parity");
  assert.equal(result.stderr, "");
  assert.equal(result.stdout, raw.stdout);
  assert.equal(normalizeTypecheckResult(result, process.cwd(), "json").diagnostics.length, 2);
  assert.equal(normalizeParserRecovery(raw, true, "timed-0"), raw);
});

test("two-server parser recovery retains the first sharded fallback notice", () => {
  const stderr = recovered.stderr.replace(
    "Corsa CLI fast path unavailable; degraded to the slower project-session API",
    "sharded Corsa CLI check failed; degraded to a single CLI program",
  );
  const raw = { ...recovered, stderr };
  const result = normalizeParserRecovery(raw, true, "parity");
  assert.equal(result.stderr, "");
  assert.equal(raw.stderr, stderr);
  assert.equal(normalizeTypecheckResult(result, process.cwd(), "json").diagnostics.length, 2);
  assert.equal(normalizeParserRecovery(raw, true, "timed-0"), raw);
});

test("timed, cold, gate and unmarked fixtures still reject recovery stderr", () => {
  for (const [expected, phase] of [
    [false, "parity"],
    [true, "cold"],
    [true, "timed-0"],
    [true, "warmup-0"],
    [true, "gate-corpus"],
    [true, "profile"],
  ] as const) {
    const result = normalizeParserRecovery(recovered, expected, phase);
    assert.equal(result, recovered);
    assert.throws(() => normalizeTypecheckResult(result, process.cwd(), "json"), /stderr/u);
  }
});

test("unknown, duplicated, missing or different-cause notices remain errors", () => {
  for (const stderr of [
    `${recovered.stderr}unexpected failure\n`,
    `${warning}\n${warning}\n`,
    "error handling method 'exit': EOF\n",
    "context canceled\n",
    `${recovered.stderr}context canceled\n`,
    warning.replace("check failure", "spawn failure"),
    warning.replace("check failure", "parse failure"),
  ]) {
    const result = { ...recovered, stderr };
    assert.equal(normalizeParserRecovery(result, true, "parity"), result);
  }
});

test("syntax recovery requires an error exit and a syntax diagnostic", () => {
  for (const result of [
    { ...recovered, status: 0 },
    { ...recovered, status: null },
    { ...recovered, stdout: recovered.stdout.replace("TS1260", "TS2322") },
  ]) {
    assert.equal(normalizeParserRecovery(result, true, "parity"), result);
  }
});

test("the explicit JS JSX fixture requires its loader.js TS7026 diagnostic", () => {
  const jsx = {
    ...recovered,
    stdout: recovered.stdout.replace("Comp0.vue", "loader.js").replace("TS1260", "TS7026"),
  };
  const normalized = normalizeParserRecovery(jsx, "js-jsx", "parity");
  assert.equal(normalized.stderr, "");
  assert.equal(normalized.stdout, jsx.stdout);
  assert.equal(normalizeTypecheckResult(normalized, process.cwd(), "json").diagnostics.length, 2);
  assert.equal(normalizeParserRecovery(jsx, true, "parity"), jsx);
  assert.equal(normalizeParserRecovery(jsx, false, "parity"), jsx);
  assert.equal(normalizeParserRecovery(jsx, "js-jsx", "timed-0"), jsx);
  for (const stdout of [
    jsx.stdout.replace("loader.js", "other.js"),
    jsx.stdout.replace("TS7026", "TS2322"),
  ]) {
    const other = { ...jsx, stdout };
    assert.equal(normalizeParserRecovery(other, "js-jsx", "parity"), other);
  }
});

test("recovery does not relax report consistency checks", () => {
  const result = normalizeParserRecovery(
    { ...recovered, stdout: recovered.stdout.replace('"errorCount":2', '"errorCount":1') },
    true,
    "parity",
  );
  assert.throws(
    () => normalizeTypecheckResult(result, process.cwd(), "json"),
    /count does not match/u,
  );
});
