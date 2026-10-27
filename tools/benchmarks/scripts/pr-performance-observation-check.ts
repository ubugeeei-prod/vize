/** Independent full-count and negative/repair witnesses, outside timing samples. */
import assert from "node:assert/strict";
import { cpSync, existsSync, mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import {
  MINIMAL_PLANTS,
  prepareMinimalPlants,
  prepareCorpusPlant,
  vizeReportCatchesPlant,
  vizeCorpusPlantMatches,
} from "./check-gate-plants.mjs";
import {
  runObservation,
  invocationSucceeded,
  writeJson,
} from "./pr-performance-observation-evidence.ts";
import type { Invocation } from "./pr-performance-observation-evidence.ts";

type FileReport = { file: string; diagnostics: string[] };
export type CheckPacket = {
  files: FileReport[];
  errorCount: number;
  warningCount: number;
  fileCount: number;
};

export function checkPacket(stdout: string): CheckPacket {
  const report = JSON.parse(stdout) as CheckPacket;
  assert.ok(Array.isArray(report.files));
  assert.equal(report.fileCount, report.files.length, "fileCount must equal full array length");
  assert.equal(
    new Set(report.files.map((file) => file.file)).size,
    report.files.length,
    "duplicate file entry",
  );
  let errors = 0,
    warnings = 0;
  for (const file of report.files) {
    assert.equal(typeof file.file, "string");
    assert.ok(Array.isArray(file.diagnostics));
    for (const diagnostic of file.diagnostics) {
      assert.equal(typeof diagnostic, "string");
      if (diagnostic.startsWith("error:")) errors++;
      else if (diagnostic.startsWith("warning:")) warnings++;
      else throw new Error("unrecognized diagnostic severity");
    }
  }
  assert.equal(
    report.errorCount,
    errors,
    "error count must include identical duplicate diagnostics",
  );
  assert.equal(report.warningCount, warnings);
  return {
    files: [...report.files].sort((a, b) => a.file.localeCompare(b.file)),
    errorCount: errors,
    warningCount: warnings,
    fileCount: report.fileCount,
  };
}

export function assertFileCoverage(packet: CheckPacket, expected: string[]) {
  const actual = packet.files
    .filter((file) => file.file.endsWith(".vue"))
    .map((file) => file.file.replaceAll("\\", "/").replace(/^\.\//u, ""))
    .sort();
  assert.deepEqual(actual, [...expected].sort(), "every timed SFC must be reported exactly once");
}

const repair = (id: string, source: string) => {
  if (id === "script")
    return source.replace('const n: number = "not-a-number";', "const n: number = 1;");
  if (id === "templateProp")
    return source.replace(
      'const disabledFlag: string = "yes";',
      "const disabledFlag: boolean = true;",
    );
  if (id === "templateEvent")
    return source.replace(
      "const clickHandler = 123;",
      "const clickHandler = (_e: PointerEvent) => {};",
    );
  if (id === "componentProp") return source.replace('variant="danger"', 'variant="primary"');
  throw new Error("unknown witness");
};

export function qualifyCheck(options: {
  binary: string;
  backend: string | null;
  vueDir: string | null;
  inputDir: string;
  out: string;
  expectedFiles: string[];
  threads: number | "max";
  args: string[];
  env: NodeJS.ProcessEnv;
  timeoutMs: number;
}) {
  const result: {
    qualified: boolean;
    reason: string | null;
    baseline: CheckPacket | null;
    witnesses: Record<string, boolean>;
    scope: string;
  } = {
    qualified: false,
    reason: null,
    baseline: null,
    witnesses: {},
    scope:
      "all timed SFCs reported once; four invalid/repair classes and full-corpus two-diagnostic plant; not exhaustive type soundness",
  };
  mkdirSync(options.out, { recursive: true });
  try {
    assert.ok(options.backend && existsSync(options.backend), "explicit backend unavailable");
    assert.ok(options.vueDir && existsSync(options.vueDir), "explicit Vue types unavailable");
    const invoke = (cwd: string, id: string, corpus: boolean) => {
      // Keep the timed lane's program width and flags; replace only reporting/path arguments.
      const args = [...options.args];
      const configIndex = args.indexOf("--tsconfig");
      if (configIndex >= 0) args[configIndex + 1] = join(cwd, "tsconfig.json");
      args.push("--format", "json");
      const invocation: Invocation = {
        executable: options.binary,
        args,
        cwd,
        env: options.env,
        rawDir: join(options.out, "raw", id),
        timeoutMs: options.timeoutMs,
      };
      const run = runObservation(invocation);
      assert.ok(invocationSucceeded(run, true), `check command failed: ${id}`);
      const packet = checkPacket(run.stdout);
      assert.equal(
        run.receipt.status,
        packet.errorCount > 0 ? 1 : 0,
        `check exit/packet disagreement: ${id}`,
      );
      if (corpus) assertFileCoverage(packet, options.expectedFiles);
      return { packet, status: run.receipt.status };
    };
    result.baseline = invoke(options.inputDir, "baseline-full", true).packet;
    const plants = prepareMinimalPlants(options.out, options.vueDir);
    for (const plant of MINIMAL_PLANTS) {
      const dir = (plants.dirs as Record<string, string>)[plant.id];
      writeJson(join(dir, "negative-sources.json"), plant.files);
      const negative = invoke(dir, `${plant.id}-invalid`, false);
      assert.equal(negative.status, 1);
      assertFileCoverage(negative.packet, Object.keys(plant.files));
      assert.ok(
        vizeReportCatchesPlant(negative.packet, plant.expected.file, [plant.expected.diagnostic]),
        `missing exact invalid witness: ${plant.id}`,
      );
      for (const [file, source] of Object.entries(plant.files))
        writeFileSync(join(dir, file), repair(plant.id, source as string));
      const valid = invoke(dir, `${plant.id}-repaired`, false);
      assertFileCoverage(valid.packet, Object.keys(plant.files));
      assert.equal(valid.status, 0);
      assert.equal(valid.packet.errorCount, 0);
      assert.equal(valid.packet.warningCount, 0);
      result.witnesses[plant.id] = true;
    }
    const copy = join(options.out, "full-corpus");
    cpSync(options.inputDir, copy, {
      recursive: true,
      verbatimSymlinks: true,
      filter: (source) => !["node_modules", ".git", "target", "dist"].includes(basename(source)),
    });
    if (existsSync(join(options.inputDir, "node_modules")))
      symlinkSync(join(options.inputDir, "node_modules"), join(copy, "node_modules"), "dir");
    else {
      mkdirSync(join(copy, "node_modules"));
      symlinkSync(options.vueDir, join(copy, "node_modules", "vue"), "dir");
      const vueNamespace = join(dirname(options.vueDir), "@vue");
      if (existsSync(vueNamespace))
        symlinkSync(vueNamespace, join(copy, "node_modules", "@vue"), "dir");
    }
    const config = JSON.parse(readFileSync(join(copy, "tsconfig.json"), "utf8"));
    const full = prepareCorpusPlant(copy, config);
    const planted = invoke(full.dir, "full-corpus-invalid", false);
    assert.equal(planted.status, 1);
    assert.equal(planted.packet.fileCount, result.baseline.fileCount + 1);
    assert.ok(
      vizeCorpusPlantMatches(result.baseline, planted.packet),
      "corpus plant changed/missed diagnostics",
    );
    result.witnesses.fullCorpus = true;
    result.qualified = true;
  } catch (error) {
    result.reason = String(error);
  }
  writeJson(join(options.out, "qualification.json"), result);
  return result;
}
