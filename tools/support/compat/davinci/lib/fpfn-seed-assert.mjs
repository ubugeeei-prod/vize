// Identity-based recall assertion for the seeded-defect pilot (P0-13).
//
// The oracle compares the EXACT diagnostic set of the seeded tree against
// the manifest, per the assurance doctrine's ban on count-only matching:
//
//   expected(seeded) = shift(baseline diagnostics through the edit list)
//                    ∪ {class-(a) expected diagnostics from the manifest}
//
// and the verdict is pass only when actual == expected as a multiset. Every
// deviation is listed exactly: class-(a) misses (a seeded defect the
// toolchain failed to flag), baseline-shift misses (a pristine diagnostic
// that vanished), unexpected diagnostics (drift the seeding should not have
// caused), and unmappable baselines (a pristine diagnostic overlapping an
// edit span — requires human triage).
//
// Identity is (path, ruleId, severity, line, column, endLine, endColumn).
// Message text is deliberately excluded: locale stability of diagnostic
// text is owned by the canon suites, not this recall oracle.

import fs from "node:fs";
import path from "node:path";

import {
  CLASS_A,
  CLASS_B,
  CLASS_B_RULE,
  applySeed,
  planClassB,
  describeMappedSpan,
  spanOverlapsEdits,
} from "./fpfn-seed-apply.mjs";
import {
  diagnosticKey,
  flattenLintJson,
  lineColToIndex,
  lineStartsOf,
  runVizeLintJson,
  sortDiagnostics,
} from "./fpfn-shared.mjs";

/** Load lint JSON either from a self-test hook file or by running vize. */
function loadLintRows(hookPath, cli, cwd, files) {
  if (hookPath != null) {
    return flattenLintJson(JSON.parse(fs.readFileSync(hookPath, "utf8")));
  }
  return flattenLintJson(runVizeLintJson(cli, cwd, files));
}

function countByKey(rows) {
  const counts = new Map();
  for (const row of rows) {
    const key = diagnosticKey(row);
    const entry = counts.get(key);
    if (entry) entry.count += 1;
    else counts.set(key, { row, count: 1 });
  }
  return counts;
}

/** Multiset difference a − b as sorted rows. */
function multisetDifference(a, b) {
  const out = [];
  for (const [key, { row, count }] of a) {
    const other = b.get(key)?.count ?? 0;
    for (let i = 0; i < count - other; i += 1) out.push(row);
  }
  return sortDiagnostics(out);
}

/** Map every baseline diagnostic into seeded coordinates via the manifest. */
function shiftBaseline(rows, manifest, outDir) {
  const shifted = [];
  const unmappable = [];
  const textCache = new Map();
  const readText = (tree, relPath) => {
    const key = `${tree}:${relPath}`;
    if (!textCache.has(key)) {
      textCache.set(key, fs.readFileSync(path.join(outDir, tree, relPath), "utf8"));
    }
    return textCache.get(key);
  };
  for (const row of rows) {
    const edits = manifest.edits[row.path] ?? [];
    if (edits.length === 0) {
      shifted.push(row);
      continue;
    }
    const originalText = readText("original", row.path);
    const originalStarts = lineStartsOf(originalText);
    const start = lineColToIndex(originalText, originalStarts, row.line, row.column);
    const end = lineColToIndex(originalText, originalStarts, row.endLine, row.endColumn);
    if (start == null || end == null || spanOverlapsEdits(start, end, edits)) {
      unmappable.push(row);
      continue;
    }
    const seededText = readText("seeded", row.path);
    const described = describeMappedSpan(seededText, edits, start, end);
    if (described == null) {
      unmappable.push(row);
      continue;
    }
    shifted.push({
      path: row.path,
      ruleId: row.ruleId,
      severity: row.severity,
      line: described.line,
      column: described.column,
      endLine: described.endLine,
      endColumn: described.endColumn,
    });
  }
  return { shifted: sortDiagnostics(shifted), unmappable: sortDiagnostics(unmappable) };
}

function expectedClassARows(manifest) {
  return manifest.injections
    .filter((injection) => injection.class === CLASS_A)
    .map((injection) => ({
      path: injection.path,
      ruleId: injection.expectedRule,
      // vue/no-undefined-refs default severity is Warning (=1 in the JSON
      // format) per its RuleMeta in
      // crates/vize_patina/src/rules/vue/no_undefined_refs.rs.
      severity: 1,
      line: injection.expected.line,
      column: injection.expected.column,
      endLine: injection.expected.endLine,
      endColumn: injection.expected.endColumn,
      identifier: injection.identifier.original,
    }));
}

/** Build the full identity-assertion report. Pure given its inputs. */
export function assertSeededTree({ manifest, outDir, cli, hooks }) {
  const files = manifest.files.map((file) => file.path);
  const baselineRows = loadLintRows(
    hooks?.baselineLintJson,
    cli,
    path.join(outDir, "original"),
    files,
  ).filter((row) => row.ruleId !== CLASS_B_RULE);
  const seededRows = loadLintRows(
    hooks?.seededLintJson,
    cli,
    path.join(outDir, "seeded"),
    files,
  ).filter((row) => row.ruleId !== CLASS_B_RULE);

  const { shifted, unmappable } = shiftBaseline(baselineRows, manifest, outDir);
  const classARows = expectedClassARows(manifest);
  const expectedRows = sortDiagnostics([
    ...shifted,
    ...classARows.map(({ identifier: _identifier, ...row }) => row),
  ]);

  const expectedCounts = countByKey(expectedRows);
  const actualCounts = countByKey(seededRows);
  const missingRows = multisetDifference(expectedCounts, actualCounts);
  const unexpected = multisetDifference(actualCounts, expectedCounts);

  const classAKeys = new Map(
    classARows.map((row) => {
      const { identifier, ...bare } = row;
      return [diagnosticKey(bare), identifier];
    }),
  );
  const classAMisses = [];
  const baselineMisses = [];
  for (const row of missingRows) {
    const identifier = classAKeys.get(diagnosticKey(row));
    if (identifier != null) classAMisses.push({ ...row, identifier });
    else baselineMisses.push(row);
  }

  const unused = assertUnused({ manifest, outDir, cli, hooks });
  shifted.push(...unused.shifted);
  unmappable.push(...unused.unmappable);
  baselineMisses.push(...unused.baselineMisses);
  unexpected.push(...unused.unexpected);

  const pass =
    classAMisses.length === 0 &&
    unused.report.misses.length === 0 &&
    baselineMisses.length === 0 &&
    unmappable.length === 0 &&
    unexpected.length === 0;

  return {
    schemaVersion: 1,
    tool: "tools/davinci/seed-defects.mjs --assert",
    source: manifest.source,
    scope: manifest.scope,
    lint: {
      baselineDiagnostics: baselineRows.length + unused.baselineCount,
      seededDiagnostics: seededRows.length + unused.seededCount,
    },
    classA: {
      expected: classARows.length,
      detected: classARows.length - classAMisses.length,
      misses: classAMisses,
    },
    classB: unused.report,
    baselineShift: {
      mapped: shifted.length,
      misses: baselineMisses,
      unmappable,
    },
    unexpected,
    verdict: pass ? "pass" : "fail",
  };
}

function assertUnused({ manifest, outDir, cli, hooks }) {
  const plane = path.join(outDir, "class-b");
  const bManifest = { ...manifest, edits: {} };
  const files = manifest.files.map((file) => file.path);
  for (const file of files) {
    const original = fs.readFileSync(path.join(outDir, "original", file), "utf8");
    const { seeded, edits } = applySeed(original, null, planClassB(original).plan);
    bManifest.edits[file] = edits;
    for (const [tree, text] of [
      ["original", original],
      ["seeded", seeded],
    ]) {
      const target = path.join(plane, tree, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, text);
    }
  }
  const load = (tree, hook) => {
    const cwd = path.join(plane, tree);
    const config = path.join(cwd, "vite.config.mjs");
    fs.writeFileSync(
      config,
      `export default ${JSON.stringify({ vue: { linter: { rules: { [CLASS_B_RULE]: "warn" } } } })};\n`,
    );
    const rows = hook
      ? loadLintRows(hook, cli, cwd, files)
      : flattenLintJson(runVizeLintJson(cli, cwd, files, config));
    return hook ? rows.filter((row) => row.ruleId !== "vue/no-undefined-refs") : rows;
  };
  const baseline = load("original", hooks?.baselineLintJson);
  const seeded = load("seeded", hooks?.seededLintJson);
  const { shifted, unmappable } = shiftBaseline(baseline, bManifest, plane);
  const expected = manifest.injections
    .filter((row) => row.class === CLASS_B)
    .map((row) => ({
      path: row.path,
      ruleId: CLASS_B_RULE,
      severity: 1,
      line: row.expected.line,
      column: row.expected.column,
      endLine: row.expected.endLine,
      endColumn: row.expected.endColumn,
    }));
  const expectedCounts = countByKey([...shifted, ...expected]);
  const actualCounts = countByKey(seeded);
  const seedKeys = new Set(expected.map(diagnosticKey));
  const missing = multisetDifference(expectedCounts, actualCounts);
  const misses = missing.filter((row) => seedKeys.has(diagnosticKey(row)));
  return {
    report: {
      expected: expected.length,
      detected: expected.length - misses.length,
      misses,
      note: "gated by exact rule, severity and span on the independent class-b mutation plane (opt-in consumer)",
    },
    baselineCount: baseline.length,
    seededCount: seeded.length,
    shifted,
    unmappable,
    baselineMisses: missing.filter((row) => !seedKeys.has(diagnosticKey(row))),
    unexpected: multisetDifference(actualCounts, expectedCounts),
  };
}
