import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import { serializeFailure } from "../../tools/support/release/typed_defaults/observation.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const project = join(root, "tsconfig.release-observers.json");
const checker = join(root, "tools/support/typescript/check-project.ts");

void test("failure packets retain Error fields across provider realms and retain non-Error throws", () => {
  const foreign: unknown = runInNewContext("new TypeError('foreign provider failure')");
  assert.deepEqual(serializeFailure(foreign), {
    name: "TypeError",
    message: "foreign provider failure",
  });
  assert.deepEqual(serializeFailure("provider failure"), {
    name: "Error",
    message: "provider failure",
  });
});

void test("release observers pass the real strict compiler and reject malformed API inputs", () => {
  const check = (config: string) => {
    const result = spawnSync(process.execPath, [checker, config], {
      encoding: "utf8",
      shell: false,
    });
    assert.equal(result.error, undefined);
    return result;
  };
  const actual = check(project);
  assert.equal(actual.status, 0, actual.stdout + actual.stderr);
  const temporary = mkdtempSync(join(root, ".vize-observer-types-"));
  try {
    const witness = join(temporary, "witness.ts");
    const config = join(temporary, "tsconfig.json");
    writeFileSync(join(temporary, "package.json"), JSON.stringify({ type: "module" }));
    writeFileSync(config, JSON.stringify({ extends: project, files: [witness], include: [] }));
    const observer = JSON.stringify(join(root, "tools/support/release/typed_defaults/observe.ts"));
    const contracts = JSON.stringify(
      join(root, "tools/support/release/typed_defaults/observation.ts"),
    );
    const reports = JSON.stringify(join(root, "tools/support/release/lint_reports/observe.ts"));
    const prefix = `import {observePackets, type TypedFixtures} from ${observer};
import {observeReports, type ReportScenario} from ${reports};
import type {Distribution} from ${contracts};
declare const fixtures: TypedFixtures;
declare const report: ReportScenario;
declare const distribution: Distribution;
`;
    writeFileSync(witness, prefix + "observePackets(fixtures, distribution, '.', '.');\n");
    const valid = check(config);
    assert.equal(valid.status, 0, valid.stdout + valid.stderr);
    writeFileSync(
      witness,
      prefix + "observePackets(fixtures, {member: {binary: 'cli'}}, '.', '.');\n",
    );
    const missingCustody = check(config);
    assert.notEqual(missingCustody.status, 0);
    assert.match(missingCustody.stdout + missingCustody.stderr, /TS2739/u);
    writeFileSync(
      witness,
      prefix +
        "observeReports({vectors: [{...report, source: 123}]}, distribution, '.', '.', () => {});\n",
    );
    const invalidSource = check(config);
    assert.notEqual(invalidSource.status, 0);
    assert.match(invalidSource.stdout + invalidSource.stderr, /TS2322/u);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
