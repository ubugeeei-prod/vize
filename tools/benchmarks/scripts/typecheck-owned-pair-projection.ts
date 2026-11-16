/** Untimed byte probes, retaining every transform and exact per-SFC coverage. */
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync } from "node:fs";
import { dirname, isAbsolute, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { sha256, writeJson } from "./typecheck-owned-pair-evidence.ts";
import { exceedsProjectionBudget, type ProbeRecord } from "./typecheck-corpus-pair-parse.ts";

const script = join(dirname(fileURLToPath(import.meta.url)), "content-mapper-byte-probe.ts");
// The official Vue mapper retains the project graph while the untimed probe covers every SFC.
// Keep that admission probe outside Node's default old-space ceiling on the 20k corpus.
export const PROBE_MAPPER_MAX_OLD_SPACE_MB = 32 * 1024;

/** Give Node-based reference mappers enough heap to cover the full owned corpus. */
export function probeMapperCommand(exec: string[]) {
  const [command, ...args] = exec;
  assert.ok(command, "content mapper exec must not be empty");
  return command === "node"
    ? [process.execPath, `--max-old-space-size=${PROBE_MAPPER_MAX_OLD_SPACE_MB}`, ...args]
    : exec;
}

/** Install an owned probe package that preserves the real mapper command. */
export function prepareOwnedProbe(project: string, mapper: "vize" | "vue") {
  const real = realpathSync(
    join(project, "node_modules", mapper === "vize" ? "vize" : "@vue/content-mapper"),
  );
  const manifest = JSON.parse(readFileSync(join(real, "package.json"), "utf8"));
  const contentMapper = manifest.typescript?.contentMapper;
  assert.ok(contentMapper?.exec?.length > 0);
  const mapperCommand = probeMapperCommand(contentMapper.exec);
  const name = `${mapper}-owned-pair-probe`,
    directory = join(project, "node_modules", name);
  mkdirSync(directory, { recursive: true });
  writeJson(join(directory, "package.json"), {
    name,
    version: "0.0.0",
    private: true,
    typescript: {
      contentMapper: {
        ...contentMapper,
        exec: [process.execPath, script, real, "--", ...mapperCommand],
      },
    },
  });
  writeJson(join(project, `tsconfig.${name}.json`), {
    extends: "./tsconfig.json",
    contentMappers: [{ package: name, extensions: [".vue"] }],
  });
  return `tsconfig.${name}.json`;
}
const EXTERNAL_PREFIX = "external:node_modules/";

/**
 * Give dependency SFCs a provider-independent identity.
 *
 * Nuxt's app program legitimately transforms framework SFCs reached through
 * the workload's linked `node_modules`. The host reports their physical
 * provider path, which is outside the generated project. Preserve those rows
 * in the proof instead of dropping them, while continuing to reject every
 * other out-of-project input.
 */
function probeFileIdentity(project: string, fileName: string): string {
  const relativeFile = relative(project, fileName);
  const file = relativeFile.replaceAll("\\", "/");
  assert.ok(file.endsWith(".vue"), `unexpected transformed input: ${file}`);
  if (file !== ".." && !file.startsWith("../") && !isAbsolute(relativeFile)) {
    return file.startsWith("node_modules/") ? `external:${file}` : file;
  }
  const normalized = fileName.replaceAll("\\", "/");
  const marker = "/node_modules/";
  const index = normalized.indexOf(marker);
  assert.ok(index >= 0, `unexpected transformed input: ${file}`);
  return `${EXTERNAL_PREFIX}${normalized.slice(index + marker.length)}`;
}

/** Admit every expected authored SFC and retain every dependency transform. */
export function ownedProbeRows(
  text: string,
  project: string,
  expected: number,
  expectedFiles: string[],
) {
  const byFile = new Map<string, { bytes: number; sourceBytes: number; templateBytes: number }>();
  for (const line of text.split("\n").filter(Boolean)) {
    const record = JSON.parse(line) as ProbeRecord;
    assert.ok(!("error" in record), `mapper transform failed: ${line}`);
    const file = probeFileIdentity(project, record.fileName);
    for (const key of ["bytes", "sourceBytes", "templateBytes"] as const)
      assert.ok(Number.isSafeInteger(record[key]) && record[key] >= 0, `${file}: invalid ${key}`);
    const sizes = {
      bytes: record.bytes,
      sourceBytes: record.sourceBytes,
      templateBytes: record.templateBytes,
    };
    if (byFile.has(file))
      assert.deepEqual(sizes, byFile.get(file), `unstable repeated projection: ${file}`);
    byFile.set(file, sizes);
  }
  assert.equal(expectedFiles.length, expected);
  const authoredFiles = [...byFile.keys()].filter((file) => !file.startsWith(EXTERNAL_PREFIX));
  assert.deepEqual(
    authoredFiles.sort(),
    [...expectedFiles].sort(),
    "actual transformed SFC set differs from authored input",
  );
  return [...byFile]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([file, sizes]) => ({ file, ...sizes }));
}
/** Compare the same authored SFC set across Vize base, Vize head and Vue. */
export function compareOwnedProjections(
  base: ReturnType<typeof ownedProbeRows>,
  head: ReturnType<typeof ownedProbeRows>,
  vue: ReturnType<typeof ownedProbeRows>,
) {
  assert.deepEqual(
    base.map((row) => row.file),
    vue.map((row) => row.file),
    "base/Vue transformed SFC set differs",
  );
  assert.deepEqual(
    head.map((row) => row.file),
    vue.map((row) => row.file),
    "head/Vue transformed SFC set differs",
  );
  for (const rows of [base, head])
    assert.deepEqual(
      rows.map(({ file, sourceBytes, templateBytes }) => ({ file, sourceBytes, templateBytes })),
      vue.map(({ file, sourceBytes, templateBytes }) => ({ file, sourceBytes, templateBytes })),
      "authored SFC/template bytes differ between mapper variants",
    );
  const rows = vue.map((row, index) => ({
    file: row.file,
    sourceBytes: row.sourceBytes,
    templateBytes: row.templateBytes,
    base: base[index].bytes,
    head: head[index].bytes,
    vue: row.bytes,
    exceedsVue: head[index].bytes > row.bytes,
  }));
  return {
    rows,
    violations: rows.filter(exceedsProjectionBudget),
    probeScript: { path: script, sha256: sha256(readFileSync(script)) },
  };
}

/** Refuse evidence when a Vize head projection breaks the ratcheted per-SFC byte budget. */
export function assertOwnedProjectionBudget(
  report: ReturnType<typeof compareOwnedProjections>,
): void {
  assert.equal(
    report.violations.length,
    0,
    `Vize head exceeds both Vue and Vize base for ${report.violations.map((row) => row.file).join(", ")}`,
  );
}
