/** Complete diagnostic/program normalization and the paired timing protocol. */
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import { join, relative, resolve, sep } from "node:path";
import { fileSha256 } from "./benchmark-binary.mjs";
import { diagnosticFingerprint, normalizeTypecheckResult } from "./typecheck-command.mjs";

export const SIDES = ["base", "head"];
export const FILE_COUNT = 500;
export const WARMUPS = 2;
export const PAIRS = 9;
// Keep the existing check-gate protocol. 1T means one server plus Rayon=1;
// the native runtime retains its default scheduler width in both modes.
export const MODES = [
  { id: "max", args: [], rayon: null },
  { id: "1t", args: ["--servers", "1"], rayon: 1 },
];
export const BUILD = {
  profile: "ci-opt",
  inherits: "release",
  lto: "thin",
  codegenUnits: 16,
  strip: "symbols",
  incremental: false,
};
export const PROTOCOL = {
  corpora: [
    { id: "generated500", vueFiles: FILE_COUNT, pattern: "." },
    { id: "shared-leaf501-default", vueFiles: 501, pattern: "default", config: "disabled" },
  ],
  warmups: WARMUPS,
  pairs: PAIRS,
  cold: "first vize check process per side/mode/corpus; filesystem caches are not evicted",
  steady:
    "fresh CLI and native TypeScript processes; filesystem caches may be warm; no persistent checker session",
  rotation: "base/head then head/base, alternating within each row",
  gates:
    "all minimal plants and full timed corpus plus plant, on every side and thread mode before warmups",
  profile: "separate --profile-json runs after all unprofiled timing pairs, excluded from timings",
  projection: "complete generated code, mappings and semantic links must match before CLI runs",
};

export function writeJson(path, data) {
  writeFileSync(path, `${JSON.stringify(data, null, 2)}\n`);
}

export function compareStrings(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function pairOrder(index) {
  return index % 2 === 0 ? SIDES : [...SIDES].reverse();
}

export function checkEnvironment(mode, source = process.env) {
  const env = { ...source };
  for (const key of Object.keys(env))
    if (
      key.startsWith("VIZE_") ||
      ["CORSA_PATH", "TSGO_PATH", "RAYON_NUM_THREADS", "GOMAXPROCS"].includes(key)
    )
      delete env[key];
  env.NO_COLOR = "1";
  env.VIZE_BENCH = "1";
  if (mode.rayon != null) env.RAYON_NUM_THREADS = String(mode.rayon);
  return env;
}

function portablePath(path, cwd) {
  return relative(cwd, resolve(cwd, path)).split(sep).join("/") || ".";
}

/** Include clean files and effective program membership as well as diagnostics. */
export function normalizeCliReport(result, cwd, expectedVueFiles = []) {
  const checked = normalizeTypecheckResult(result, cwd, "json");
  const report = JSON.parse(result.stdout);
  assert(Array.isArray(report.programs), "check report has no effective program evidence");
  assert(Number.isSafeInteger(report.fileCount) && report.fileCount >= 0, "invalid fileCount");
  const files = report.files
    .map((entry) => {
      assert.equal(typeof entry.file, "string");
      assert(Array.isArray(entry.diagnostics));
      return {
        file: portablePath(entry.file, cwd),
        diagnostics: [...entry.diagnostics].sort(compareStrings),
      };
    })
    .sort((a, b) => a.file.localeCompare(b.file, "en"));
  assert.equal(
    new Set(files.map((entry) => entry.file)).size,
    files.length,
    "duplicate report file",
  );
  const vueFiles = files
    .filter((entry) => entry.file.endsWith(".vue"))
    .map((entry) => entry.file)
    .sort(compareStrings);
  assert.deepEqual(
    vueFiles,
    [...expectedVueFiles].sort(compareStrings),
    "check did not report the entire Vue corpus",
  );
  assert.equal(report.fileCount, files.length, "fileCount disagrees with report coverage");
  const programs = report.programs
    .map((program) => {
      assert(Array.isArray(program.files), "program has no source membership");
      return {
        root: portablePath(program.root, cwd),
        tsconfig: program.tsconfig == null ? null : portablePath(program.tsconfig, cwd),
        compilerOptions: program.compilerOptions ?? null,
        files: program.files.map((file) => portablePath(file, cwd)).sort(compareStrings),
      };
    })
    .sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b), "en"));
  const members = new Set(programs.flatMap((program) => program.files));
  for (const file of expectedVueFiles) assert(members.has(file), `program did not include ${file}`);
  return {
    ...checked,
    errorCount: report.errorCount,
    warningCount: report.warningCount,
    fileCount: report.fileCount,
    files,
    programs,
  };
}

function median(values) {
  assert(values.length > 0 && values.every((value) => Number.isFinite(value) && value > 0));
  return [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
}

export function summarizePairs(samples) {
  assert.equal(samples.base.length, PAIRS);
  assert.equal(samples.head.length, PAIRS);
  const base = samples.base.map((sample) => sample.ms);
  const head = samples.head.map((sample) => sample.ms);
  const ratios = head.map((ms, index) => ms / base[index]);
  return {
    baseMedianMs: median(base),
    headMedianMs: median(head),
    headToBaseMedianRatio: median(head) / median(base),
    medianPairedHeadToBaseRatio: median(ratios),
    baseSamplesMs: base,
    headSamplesMs: head,
    pairedHeadToBaseRatios: ratios,
    sampleIds: Object.fromEntries(
      SIDES.map((side) => [side, samples[side].map((sample) => sample.id)]),
    ),
  };
}

export function profileReads(profile, side, expectedRetainedReads) {
  assert(Array.isArray(profile.counters), "profile counters are missing");
  const names = {
    retained: "atelier.template_reads.retained",
    parses: "atelier.template_reads.parses",
  };
  if (side === "base") return { instrumented: false, retained: null, parses: null };
  assert.equal(profile.truncation?.dropped_counters, 0, "profile counter evidence is truncated");
  const totals = Object.fromEntries(
    Object.entries(names).map(([name, key]) => {
      const entries = profile.counters.filter((counter) => counter.key === key);
      assert(entries.length <= 1, `duplicate profile counter ${key}`);
      const total = entries[0]?.total ?? 0;
      assert(Number.isSafeInteger(total) && total >= 0, `invalid profile counter ${key}`);
      return [name, total];
    }),
  );
  if (expectedRetainedReads != null) {
    assert.equal(totals.retained, expectedRetainedReads, "retained AST reads were not exercised");
    assert.equal(totals.parses, 0, "shared-leaf template reads still parsed expressions");
  }
  return { instrumented: true, ...totals };
}

/** Static protocol checks run without Rust, Vue, or the native TS runtime. */
export function selfTest() {
  const cwd = mkdtempSync(join(os.tmpdir(), "retained-template-reads-protocol-"));
  try {
    for (const file of ["A.vue", "B.vue"]) writeFileSync(join(cwd, file), "<template />\n");
    const report = {
      files: [
        { file: "B.vue", diagnostics: [] },
        {
          file: "A.vue",
          diagnostics: ["error:2:7 [TS2322] Type 'string' is not assignable to type 'number'."],
        },
      ],
      programs: [
        {
          root: ".",
          tsconfig: "tsconfig.json",
          compilerOptions: { strict: true },
          files: ["B.vue", "A.vue"],
        },
      ],
      errorCount: 1,
      warningCount: 0,
      fileCount: 2,
    };
    const result = (data, status = 1) => ({ stdout: JSON.stringify(data), stderr: "", status });
    const normalized = normalizeCliReport(result(report), cwd, ["A.vue", "B.vue"]);
    const shuffled = structuredClone(report);
    shuffled.files.reverse();
    shuffled.programs[0].files.reverse();
    shuffled.files[0].file = join(cwd, "A.vue");
    assert.equal(
      diagnosticFingerprint(normalized),
      diagnosticFingerprint(normalizeCliReport(result(shuffled), cwd, ["A.vue", "B.vue"])),
    );
    const changed = structuredClone(report);
    changed.files[1].diagnostics[0] = changed.files[1].diagnostics[0].replace("string", "boolean");
    assert.notEqual(
      diagnosticFingerprint(normalized),
      diagnosticFingerprint(normalizeCliReport(result(changed), cwd, ["A.vue", "B.vue"])),
    );
    assert.throws(
      () => normalizeCliReport(result(report, 0), cwd, ["A.vue", "B.vue"]),
      /exit status/u,
    );
    assert.throws(
      () => normalizeCliReport(result({ ...report, errorCount: 0 }), cwd, ["A.vue", "B.vue"]),
      /count/u,
    );
    assert.throws(
      () => normalizeCliReport(result(report), cwd, ["A.vue", "B.vue", "Missing.vue"]),
      /entire Vue corpus/u,
    );
    assert.throws(
      () => normalizeCliReport(result({ ...report, fileCount: 1 }), cwd, ["A.vue", "B.vue"]),
      /report coverage/u,
    );
    const missingMember = structuredClone(report);
    missingMember.programs[0].files.pop();
    assert.throws(
      () => normalizeCliReport(result(missingMember), cwd, ["A.vue", "B.vue"]),
      /program did not include/u,
    );
    assert.throws(
      () =>
        normalizeCliReport(result({ ...report, files: [...report.files, report.files[0]] }), cwd, [
          "A.vue",
          "B.vue",
        ]),
      /duplicate report file/u,
    );
    assert.throws(
      () =>
        normalizeCliReport({ ...result(report), stderr: "unexpected stderr" }, cwd, [
          "A.vue",
          "B.vue",
        ]),
      /unexpected type-check stderr/u,
    );
    assert.deepEqual(
      Array.from({ length: 4 }, (_, index) => pairOrder(index)),
      [
        ["base", "head"],
        ["head", "base"],
        ["base", "head"],
        ["head", "base"],
      ],
    );
    assert.deepEqual(
      MODES.map((mode) => mode.args),
      [[], ["--servers", "1"]],
    );
    const inherited = {
      PATH: "unchanged",
      RAYON_NUM_THREADS: "32",
      GOMAXPROCS: "32",
      VIZE_CHECKERS: "11",
      VIZE_PROFILE: "1",
    };
    const maximal = checkEnvironment(MODES[0], inherited);
    assert.equal(maximal.PATH, "unchanged");
    for (const key of ["RAYON_NUM_THREADS", "GOMAXPROCS", "VIZE_CHECKERS", "VIZE_PROFILE"])
      assert.equal(maximal[key], undefined);
    const single = checkEnvironment(MODES[1], inherited);
    assert.equal(single.RAYON_NUM_THREADS, "1");
    assert.equal(single.GOMAXPROCS, undefined);
    const samples = {
      base: Array.from({ length: PAIRS }, (_, index) => ({ id: `b${index}`, ms: index + 1 })),
      head: Array.from({ length: PAIRS }, (_, index) => ({ id: `h${index}`, ms: (index + 1) * 2 })),
    };
    assert.equal(summarizePairs(samples).medianPairedHeadToBaseRatio, 2);
    assert.equal(summarizePairs(samples).headToBaseMedianRatio, 2);
    assert.throws(() => summarizePairs({ ...samples, head: samples.head.slice(1) }));
    assert.throws(() =>
      summarizePairs({ ...samples, head: samples.head.map((sample) => ({ ...sample, ms: 0 })) }),
    );
    const profile = {
      counters: [{ key: "atelier.template_reads.retained", total: 40_000 }],
      truncation: { dropped_counters: 0 },
    };
    assert.deepEqual(profileReads(profile, "head", 40_000), {
      instrumented: true,
      retained: 40_000,
      parses: 0,
    });
    assert.deepEqual(profileReads({ counters: [] }, "base", 40_000), {
      instrumented: false,
      retained: null,
      parses: null,
    });
    assert.throws(() => profileReads(profile, "head", 39_999), /not exercised/u);
    assert.throws(
      () => profileReads({ ...profile, truncation: { dropped_counters: 1 } }, "head", 40_000),
      /truncated/u,
    );
    assert.throws(
      () =>
        profileReads(
          {
            ...profile,
            counters: [...profile.counters, { key: "atelier.template_reads.parses", total: 1 }],
          },
          "head",
          40_000,
        ),
      /still parsed/u,
    );
    const before = fileSha256(join(cwd, "A.vue"));
    writeFileSync(join(cwd, "A.vue"), "<template>changed</template>\n");
    assert.notEqual(fileSha256(join(cwd, "A.vue")), before);
    console.log("retained-template-reads CLI protocol checks passed");
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
}
