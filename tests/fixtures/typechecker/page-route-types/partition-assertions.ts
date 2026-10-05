import assert from "node:assert/strict";
import type { CheckReport, VizeCheckResult } from "../../../_helpers/realworld-typecheck.ts";
export type Topology = "shared-leaf" | "independent";
type FullReport = Omit<CheckReport, "files"> & {
  files: Array<CheckReport["files"][number] & { virtualTs: string }>;
};
export type Observation = Omit<VizeCheckResult, "report"> & { report: FullReport };
export const ROOTS = ["Root0.vue", "Root1.vue", "Root2.vue", "Root3.vue"];
export const HELPER_IMPORT = "import('vue-router/auto-routes')";
export const SHARED = "export const snapshot: RouteScopedGlobal = undefined!;\n";

function diagnostic(
  text: string,
  needle: string,
  token: string,
  code: number,
  message: string,
): string {
  const at = text.indexOf(needle);
  assert.notEqual(at, -1, needle);
  assert.equal(text.indexOf(needle, at + needle.length), -1, needle);
  assert.ok(needle.includes(token));
  const offset = at + needle.indexOf(token);
  const line = text.slice(0, offset).split("\n").length;
  const column = offset - text.lastIndexOf("\n", offset - 1);
  return `error:${line}:${column} [TS${code}] ${message}`;
}

export function assertReport(
  result: Observation,
  topology: Topology,
  sources: Record<string, string>,
  imported: boolean,
  broken: boolean,
): void {
  const files = topology === "shared-leaf" ? [...ROOTS, "shared.ts"] : ROOTS;
  assert.deepEqual(
    result.report.files.map((file) => file.file),
    files,
  );
  assert.equal(result.report.fileCount, files.length);
  assert.equal(result.report.warningCount, 0);
  assert.equal(result.report.programs.length, 1);
  const [program] = result.report.programs;
  assert.equal(program.root, ".");
  assert.equal(program.tsconfig, "tsconfig.json");
  assert.deepEqual(program.files, [
    ...ROOTS,
    "globals.d.ts",
    ...(topology === "shared-leaf" ? ["shared.ts"] : []),
  ]);
  let errors = 0;
  for (const file of result.report.files) {
    assert.equal(typeof file.virtualTs, "string", `complete virtual TS: ${file.file}`);
    const expected: string[] = [];
    if (file.file === "shared.ts") {
      assert.equal(file.virtualTs, SHARED, "plain shared leaf retains complete authored text");
      if (!imported)
        expected.push(
          diagnostic(
            SHARED,
            "RouteScopedGlobal",
            "RouteScopedGlobal",
            2304,
            "Cannot find name 'RouteScopedGlobal'.",
          ),
        );
    } else if (topology === "shared-leaf") {
      if (imported)
        expected.push(
          diagnostic(
            sources[file.file],
            "const scopedMismatch: string",
            "scopedMismatch",
            2322,
            "Type 'RouteScopedGlobal' is not assignable to type 'string'.",
          ),
        );
      expected.push(
        diagnostic(
          sources[file.file],
          "const unrelatedMismatch: string",
          "unrelatedMismatch",
          2322,
          "Type 'number' is not assignable to type 'string'.",
        ),
      );
    } else {
      if (!imported)
        expected.push(
          diagnostic(
            sources[file.file],
            "const scoped: RouteScopedGlobal",
            "RouteScopedGlobal",
            2304,
            "Cannot find name 'RouteScopedGlobal'.",
          ),
        );
      if (broken)
        expected.push(
          diagnostic(
            sources[file.file],
            "const numeric: string",
            "numeric",
            2322,
            "Type 'number' is not assignable to type 'string'.",
          ),
        );
    }
    assert.deepEqual(file.diagnostics, expected, `complete ordered diagnostics: ${file.file}`);
    errors += expected.length;
    const shouldImport = imported && file.file === ROOTS[0];
    assert.equal(file.virtualTs.includes(HELPER_IMPORT), shouldImport, file.file);
    if (shouldImport)
      assert.equal(file.virtualTs.split(HELPER_IMPORT).length, 2, "one implicit helper site");
  }
  assert.equal(result.report.errorCount, errors);
  assert.equal(result.status, errors === 0 ? 0 : 1, result.stderr || result.stdout);
  for (const file of result.report.files)
    assert.ok(result.stderr.includes(file.virtualTs), `complete raw virtual stderr: ${file.file}`);
}

export function assertOnlyInjectedCall(before: Observation, after: Observation): void {
  const typed = `useRoute<${HELPER_IMPORT}._RouteNamesForFilePath<${JSON.stringify(ROOTS[0])}>>()`;
  assert.deepEqual(after.report.programs, before.report.programs);
  for (let index = 0; index < before.report.files.length; index += 1) {
    const original = before.report.files[index];
    const adapted = after.report.files[index];
    assert.equal(adapted.file, original.file);
    if (original.file === ROOTS[0]) {
      assert.equal(
        original.virtualTs.split("useRoute()").length,
        2,
        "unique bare call in complete virtual TS",
      );
      assert.equal(adapted.virtualTs, original.virtualTs.replace("useRoute()", typed));
    } else
      assert.equal(
        adapted.virtualTs,
        original.virtualTs,
        `unchanged complete virtual TS: ${original.file}`,
      );
  }
}
