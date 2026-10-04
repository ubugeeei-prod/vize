#!/usr/bin/env node
/**
 * Split the pinned native runtime's terminal extendedDiagnostics footer.
 * Diagnostic and footer text retain their original bytes, including CRLF.
 * Native phase timings are reported fields, never a model of process startup.
 */
import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";

const COUNTS = [
  "Files",
  "Lines",
  "Identifiers",
  "Symbols",
  "Types",
  "Instantiations",
  "Memory allocs",
];
const TIMES = ["Config time", "Parse time", "Bind time", "Check time", "Emit time", "Total time"];
const REQUIRED = [...COUNTS, "Memory used", ...TIMES];
const TIME_MS = new Map([
  ["ns", 0.000001],
  ["us", 0.001],
  ["µs", 0.001],
  ["μs", 0.001],
  ["ms", 1],
  ["s", 1_000],
]);
const MEMORY_UNITS = new Set(["B", "K", "KB", "KiB", "M", "MB", "MiB", "G", "GB", "GiB"]);
const FIELD =
  /^([A-Za-z][A-Za-z0-9 /()_-]*):[ \t]+([0-9]+(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?)([ \t]*)([A-Za-zµμ%]*)([ \t]*)$/u;

function parseFooter(rows) {
  assert(
    rows.length > 0 && rows[0].text.startsWith("Files:"),
    "native phase footer must start with Files",
  );
  const fields = new Map();
  for (const { text } of rows) {
    const matched = FIELD.exec(text);
    assert(matched, `malformed native phase footer line: ${JSON.stringify(text)}`);
    const [, key, numericText, spacing, unit] = matched;
    assert(!fields.has(key), `duplicate native phase footer key: ${key}`);
    const value = Number(numericText);
    assert(Number.isFinite(value) && value >= 0, `invalid native phase value: ${key}`);
    if (COUNTS.includes(key))
      assert(
        /^\d+$/u.test(numericText) && Number.isSafeInteger(value) && unit === "",
        `invalid native count: ${key}`,
      );
    if (key === "Memory used") assert(MEMORY_UNITS.has(unit), "invalid native memory unit");
    if (TIMES.includes(key)) assert(TIME_MS.has(unit), `invalid native time unit: ${key}`);
    const field = { value, rawValue: `${numericText}${spacing}${unit}`, unit };
    if (TIME_MS.has(unit)) {
      field.timeMs = value * TIME_MS.get(unit);
      assert(Number.isFinite(field.timeMs), `invalid native time conversion: ${key}`);
    }
    fields.set(key, field);
  }
  for (const key of REQUIRED) assert(fields.has(key), `missing native phase footer key: ${key}`);
  assert(
    [...fields.keys()].at(-1) === "Total time",
    "native phase footer must end with Total time",
  );
  return Object.fromEntries(fields);
}

/**
 * Strip exactly one complete, unindented numeric footer at the end of stdout.
 * Statistic-looking diagnostic lines preceding that footer remain untouched.
 */
export function splitNativePhaseReport(stdout) {
  assert.equal(typeof stdout, "string", "native phase stdout must be a string");
  const rows = [...stdout.matchAll(/[^\n]*(?:\n|$)/gu)]
    .filter((match) => match[0].length > 0)
    .map((match) => ({ offset: match.index, text: match[0].replace(/\r?\n$/u, "") }));
  const starts = rows.flatMap((row, index) => (row.text.startsWith("Files:") ? [index] : []));
  assert(starts.length > 0, "native extendedDiagnostics footer is absent");
  const start = starts.at(-1);
  const fields = parseFooter(rows.slice(start));
  // A preceding complete footer is ambiguous evidence, not diagnostic text.
  for (const previous of starts.slice(0, -1)) {
    const end = rows.findIndex(
      (row, index) => index >= previous && index < start && row.text.startsWith("Total time:"),
    );
    if (end < 0) continue;
    let complete = false;
    try {
      parseFooter(rows.slice(previous, end + 1));
      complete = true;
    } catch {}
    assert(!complete, "duplicate native extendedDiagnostics footer");
  }
  const unsupportedFields = {};
  for (const [name, key] of [
    ["startupTimeMs", "Startup time"],
    ["programTimeMs", "Program time"],
  ])
    if (!Object.hasOwn(fields, key))
      unsupportedFields[name] = {
        value: null,
        status: "unknown",
        reason: "not reported by native extendedDiagnostics",
      };
  const offset = rows[start].offset;
  return {
    diagnosticText: stdout.slice(0, offset),
    rawFooter: stdout.slice(offset),
    fields,
    unsupportedFields,
  };
}

const TEST_FOOTER = `Files:              83
Lines:           58446
Identifiers:     49720
Symbols:         32791
Types:             344
Instantiations:      0
Memory used:    25586K
Memory allocs:   42233
Config time:    0.003s
Parse time:     0.028s
Bind time:      0.006s
Check time:     0.005s
Emit time:      0.000s
Total time:     0.050s
`;

export function selfTestNativePhaseReport() {
  const diagnostic =
    "雪.ts(1,7): error TS2322: Type 'Files: 83' is not assignable to type 'number'.\n  Parse time: 99s\nFiles: 1\n  TS2345: a multi-line diagnostic continues here.\n\n";
  for (const ending of ["\n", "\r\n"]) {
    const prefix = diagnostic.replaceAll("\n", ending);
    const footer = TEST_FOOTER.replaceAll("\n", ending);
    const parsed = splitNativePhaseReport(prefix + footer);
    assert(
      Buffer.from(parsed.diagnosticText).equals(Buffer.from(prefix)),
      "diagnostic prefix bytes changed",
    );
    assert(Buffer.from(parsed.rawFooter).equals(Buffer.from(footer)), "footer bytes changed");
    assert.equal(parsed.fields.Files.value, 83);
    assert.equal(parsed.fields["Memory used"].rawValue, "25586K");
    assert.equal(parsed.fields["Memory used"].unit, "K");
    assert.equal(parsed.fields["Parse time"].value, 0.028);
    assert.equal(parsed.fields["Parse time"].rawValue, "0.028s");
    assert.equal(parsed.fields["Parse time"].timeMs, 28);
    assert.equal(parsed.fields["Emit time"].timeMs, 0);
    assert.equal(parsed.fields["Total time"].timeMs, 50);
    assert.equal(Object.keys(parsed.fields).length, REQUIRED.length);
    assert.deepEqual(parsed.unsupportedFields.startupTimeMs.value, null);
    assert.equal(parsed.unsupportedFields.programTimeMs.status, "unknown");
  }
  assert.equal(splitNativePhaseReport(TEST_FOOTER).diagnosticText, "");
  assert.equal(splitNativePhaseReport(TEST_FOOTER.trimEnd()).rawFooter, TEST_FOOTER.trimEnd());
  const extra = splitNativePhaseReport(
    TEST_FOOTER.replace(
      "Total time:",
      "GC time: 0.125ms\nAllocated bytes: 12MiB\nCustom fraction: 1.25\nconstructor: 4\nTotal time:",
    ),
  );
  assert.deepEqual(extra.fields["GC time"], {
    value: 0.125,
    rawValue: "0.125ms",
    unit: "ms",
    timeMs: 0.125,
  });
  assert.deepEqual(extra.fields["Allocated bytes"], { value: 12, rawValue: "12MiB", unit: "MiB" });
  assert.equal(extra.fields["Custom fraction"].value, 1.25);
  assert.equal(extra.fields.constructor.value, 4);
  for (const [rawValue, timeMs] of [
    ["1000000ns", 1],
    ["1000us", 1],
    ["1000µs", 1],
    ["1000μs", 1],
    ["1ms", 1],
    ["0.001 s", 1],
  ]) {
    const parsed = splitNativePhaseReport(
      TEST_FOOTER.replace("Total time:", `I/O Read time: ${rawValue}\nTotal time:`),
    );
    assert.equal(parsed.fields["I/O Read time"].rawValue, rawValue);
    assert.equal(parsed.fields["I/O Read time"].timeMs, timeMs);
  }
  const reportedProgram = splitNativePhaseReport(
    TEST_FOOTER.replace("Total time:", "Program time: 0.008s\nTotal time:"),
  );
  assert.equal(reportedProgram.fields["Program time"].timeMs, 8);
  assert.equal(Object.hasOwn(reportedProgram.unsupportedFields, "programTimeMs"), false);
  const invalid = [
    "",
    diagnostic,
    TEST_FOOTER.replace(/^Files:.*\n/u, ""),
    TEST_FOOTER.replace(/^Bind time:.*\n/mu, ""),
    TEST_FOOTER.replace(/^Total time:.*\n/mu, ""),
    TEST_FOOTER.replace("Types:             344", "Types: 344\nTypes: 344"),
    TEST_FOOTER + TEST_FOOTER,
    TEST_FOOTER + diagnostic,
    TEST_FOOTER.replace("Check time:     0.005s", "Check time: NaN"),
    TEST_FOOTER.replace("Check time:     0.005s", "Check time: Infinitys"),
    TEST_FOOTER.replace("Check time:     0.005s", "Check time: -0.005s"),
    TEST_FOOTER.replace("Check time:     0.005s", "Check time: 1e309s"),
    TEST_FOOTER.replace("Check time:     0.005s", "Check time: 0.005unknown"),
    TEST_FOOTER.replace("Files:              83", "Files: 83.5"),
    TEST_FOOTER.replace("Files:              83", "Files: 9007199254740993"),
    TEST_FOOTER.replace("Files:              83", "Files: 83K"),
    TEST_FOOTER.replace("Memory used:    25586K", "Memory used: 25586"),
    TEST_FOOTER.replace("Total time:", "Unknown field: not numeric\nTotal time:"),
    TEST_FOOTER.replace("Types:             344", "\nTypes:             344"),
  ];
  for (const output of invalid) assert.throws(() => splitNativePhaseReport(output));
  assert.throws(() => splitNativePhaseReport(null), /must be a string/u);
  console.log("native phase footer parser checks passed");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    assert.deepEqual(
      process.argv.slice(2),
      ["--self-test"],
      "usage: typechecker-native-phase-report.mjs --self-test",
    );
    selfTestNativePhaseReport();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
