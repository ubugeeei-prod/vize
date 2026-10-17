import { vueConfigFixtureSource } from "./vue-config-fixture.ts";
// Named corrected-engine outputs; the entire original corpus remains unchanged.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const authority = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/differential/css-engine-vendor-exit/finite-successors.json"),
    "utf8",
  ),
);
assert.equal(authority.schema, "vize.css-engine-finite-successors");
assert.equal(authority.rustCases.length, 11);
assert.equal(authority.cliCells.length, 26);
assert.equal(new Set(authority.cliCells.map((cell: { id: string }) => cell.id)).size, 26);

interface InputCase {
  id: string;
  source: string;
  options: Record<string, unknown>;
}
interface WholeCell {
  id: string;
  args: string[];
  config: string;
  status: number | null;
  signal: NodeJS.Signals | null;
  error: unknown;
  readError: string | null;
  before: string;
  after: string | null;
  stdout: string;
  stderr: string;
  stdoutBase64: string;
  stderrBase64: string;
  beforeBase64: string;
  afterBase64: string | null;
}

export function assertOriginalCLIPlan(
  ids: readonly string[],
  formatterCases: readonly { id: string }[],
  linterCases: readonly { id: string }[],
) {
  assert.equal(formatterCases.length, 12);
  assert.equal(linterCases.length, 15);
  const expected = [
    ...formatterCases.flatMap((entry) =>
      Array.from({ length: 5 }, (_, pass) => `formatter/${entry.id}/${pass + 1}`),
    ),
    ...linterCases.flatMap((entry) =>
      Array.from({ length: 2 }, (_, pass) => `linter/${entry.id}/${pass + 1}`),
    ),
  ];
  assert.equal(expected.length, 90);
  assert.deepEqual(ids, expected, "the entire original ordered90-ID Cartesian plan");
}

export function finiteSuccessor(surface: string, entry: InputCase) {
  const matches = authority.rustCases.filter(
    (row: { surface: string; case: string }) => row.surface === surface && row.case === entry.id,
  );
  assert(matches.length <= 1, "one named successor");
  if (!matches.length) return undefined;
  const row = matches[0];
  assert.equal(row.source, entry.source, "whole unmodified original input");
  if (row.options) assert.deepEqual(row.options, entry.options);
  return row;
}

export function assertWholeFormatterCell(
  row: WholeCell,
  entry: InputCase,
  pass: number,
  expected: string,
) {
  assert(Number.isInteger(pass) && pass >= 1 && pass <= 5, "original formatter pass1..5");
  assert.equal(row.id, `formatter/${entry.id}/${pass}`);
  const check = pass === 1 || pass === 5;
  const original = pass <= 2;
  const before = original ? entry.source : expected;
  const changed = before !== expected;
  const options = Object.keys(entry.options).length
    ? ["--config", "vite.config.mjs"]
    : ["--no-config"];
  assert.deepEqual(row.args, ["fmt", ...options, check ? "--check" : "--write", "App.vue"]);
  assert.equal(row.config, vueConfigFixtureSource({ formatter: entry.options }));
  assert.equal(row.status, check && changed ? 1 : 0);
  assert.equal(row.before, before);
  assert.equal(row.after, check ? before : expected);
  assert.equal(row.stdout, "");
  const action = check ? "Checked" : "Formatted";
  const detail = changed ? (check ? "Would reformat: App.vue\n" : "Reformatted: App.vue\n") : "";
  const summary = check
    ? changed
      ? "would be reformatted"
      : "already formatted"
    : changed
      ? "reformatted"
      : "unchanged";
  assert.equal(
    row.stderr,
    `Found 1 file(s)\n${detail}\n${action} 1 file(s)\n  1 file(s) ${summary}\n`,
  );
  const changedCell = authority.cliCells.find((cell: { id: string }) => cell.id === row.id);
  if (changedCell) {
    assert.deepEqual(row.args, canonicalArguments(changedCell.args));
    assert.equal(row.config, vueConfigFixtureSource(changedCell.sourceOptions));
    assert.equal(row.after, changedCell.wholeFileAfter);
    assert.equal(row.stdout, changedCell.wholeStdout);
    assert.equal(row.stderr, changedCell.wholeStderr);
    assert.equal(row.status, changedCell.status);
  }
  assertPhysicalBytes(row);
}

export function assertWholeLinterCell(
  row: WholeCell,
  entry: InputCase,
  pass: number,
  originalStdout: string,
) {
  assert(Number.isInteger(pass) && pass >= 1 && pass <= 2, "original linter pass1..2");
  assert.equal(row.id, `linter/${entry.id}/${pass}`);
  const changed = authority.cliCells.find((cell: { id: string }) => cell.id === row.id);
  const filename = String(entry.options.filename);
  assert.deepEqual(row.args, [
    "lint",
    "-f",
    "json",
    "--help-level",
    "full",
    "--locale",
    "en",
    "--config",
    "vite.config.mjs",
    filename,
  ]);
  assert.equal(
    row.config,
    vueConfigFixtureSource({
      linter: {
        preset: "incremental",
        rules: Object.fromEntries(
          (entry.options.enabledRules as string[]).map((rule) => [rule, "warn"]),
        ),
      },
    }),
  );
  assert.equal(row.status, 0);
  assert.equal(row.before, entry.source);
  assert.equal(row.after, entry.source);
  assert.equal(row.stdout, changed ? changed.wholeStdout : originalStdout);
  assert.equal(row.stderr, "");
  if (changed) {
    assert.deepEqual(row.args, canonicalArguments(changed.args));
    assert.equal(row.config, vueConfigFixtureSource(changed.sourceOptions));
  }
  assertPhysicalBytes(row);
}

function assertPhysicalBytes(row: WholeCell) {
  assert.equal(row.signal, null);
  assert.equal(row.error, null);
  assert.equal(row.readError, null);
  for (const [value, base64] of [
    [row.stdout, row.stdoutBase64],
    [row.stderr, row.stderrBase64],
    [row.before, row.beforeBase64],
    [row.after, row.afterBase64],
  ]) {
    assert.equal(Buffer.from(value ?? "").toString("base64"), base64);
  }
}

// Transport names change with the configuration API. The complete frozen
// argument order, native options, source, output, and status stay authoritative.
function canonicalArguments(args: string[]): string[] {
  return args.map((argument) => (argument === "vize.config.json" ? "vite.config.mjs" : argument));
}
