import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { TYPECHECKER_TESTS } from "../../differential/typechecker.ts";

type Fixture = {
  repoRoot: string;
  resultDir: string;
  raw: {
    "rust-suites": {
      "vize_canon::fix_history_diagnostics": {
        testcases: Record<string, unknown>;
      };
    };
  };
};

const pack = "nuxt-plugin-injection-types";
const test = TYPECHECKER_TESTS[pack];

function changeClaim(fixture: Fixture, change: (coverage: Record<string, unknown>) => void) {
  const file = path.join(
    fixture.repoRoot,
    "tests/_fixtures/differential/typechecker/manifest.json",
  );
  const manifest = JSON.parse(fs.readFileSync(file, "utf8"));
  const entry = manifest.cases.find(
    (candidate: { id: string }) =>
      candidate.id === "typechecker/nuxt-plugin-injection-types/authored-dependency-identity",
  );
  assert(entry);
  change(entry.dialectCoverage);
  fs.writeFileSync(file, `${JSON.stringify(manifest, null, 2)}\n`);
}

export const dialectReadiness = {
  preRunChanges: [
    (fixture: Fixture) => {
      delete fixture.raw["rust-suites"]["vize_canon::fix_history_diagnostics"].testcases[test];
    },
    (fixture: Fixture) => {
      changeClaim(fixture, (coverage) => {
        coverage.registryRef = null;
      });
    },
    (fixture: Fixture) => {
      changeClaim(fixture, (coverage) => {
        const evidence = coverage.evidence as Array<Record<string, unknown>>;
        evidence[0].selector = "batch-typechecker-fixture-v1 full-workspace wrong_test";
      });
    },
  ],
  captureChanges: [
    [`typechecker-workspace/${pack}.json`, "cases.0.inputs.0.sha256", "0".repeat(64)],
  ] satisfies Array<[string, string, unknown]>,
  packetChanges: [
    (fixture: Fixture) => {
      const file = path.join(fixture.resultDir, "workspace-junit-ci.xml");
      fs.writeFileSync(
        file,
        fs
          .readFileSync(file, "utf8")
          .replace(
            `<testcase classname="vize_canon::fix_history_diagnostics" name="${test}"/>`,
            "",
          ),
      );
    },
  ],
};
