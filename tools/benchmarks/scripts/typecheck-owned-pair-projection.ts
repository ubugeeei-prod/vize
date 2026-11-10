/** Untimed byte probes, retaining every transform and exact per-SFC coverage. */
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { sha256, writeJson } from "./typecheck-owned-pair-evidence.ts";
import type { ProbeRecord } from "./typecheck-corpus-pair-parse.ts";

const script = join(dirname(fileURLToPath(import.meta.url)), "content-mapper-byte-probe.ts");
export function prepareOwnedProbe(project: string, mapper: "vize" | "vue") {
  const real = realpathSync(
    join(project, "node_modules", mapper === "vize" ? "vize" : "@vue/content-mapper"),
  );
  const manifest = JSON.parse(readFileSync(join(real, "package.json"), "utf8"));
  const contentMapper = manifest.typescript?.contentMapper;
  assert.ok(contentMapper?.exec?.length > 0);
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
        exec: [process.execPath, script, real, "--", ...contentMapper.exec],
      },
    },
  });
  writeJson(join(project, `tsconfig.${name}.json`), {
    extends: "./tsconfig.json",
    contentMappers: [{ package: name, extensions: [".vue"] }],
  });
  return `tsconfig.${name}.json`;
}
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
    const file = relative(project, record.fileName);
    assert.ok(
      !file.startsWith("../") && file.endsWith(".vue"),
      `unexpected transformed input: ${file}`,
    );
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
  assert.equal(byFile.size, expected, "not every expected SFC reached the mapper");
  assert.equal(expectedFiles.length, expected);
  assert.deepEqual(
    [...byFile.keys()].sort(),
    [...expectedFiles].sort(),
    "actual transformed SFC set differs from authored input",
  );
  return [...byFile]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([file, sizes]) => ({ file, ...sizes }));
}
export function compareOwnedProjections(
  base: ReturnType<typeof ownedProbeRows>,
  head: ReturnType<typeof ownedProbeRows>,
  vue: ReturnType<typeof ownedProbeRows>,
) {
  assert.deepEqual(
    base.map((row) => row.file),
    vue.map((row) => row.file),
  );
  assert.deepEqual(
    head.map((row) => row.file),
    vue.map((row) => row.file),
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
    violations: rows.filter((row) => row.exceedsVue),
    probeScript: { path: script, sha256: sha256(readFileSync(script)) },
  };
}

export function assertOwnedProjectionBudget(
  report: ReturnType<typeof compareOwnedProjections>,
): void {
  assert.equal(
    report.violations.length,
    0,
    `Vize head exceeds Vue for ${report.violations.map((row) => row.file).join(", ")}`,
  );
}
