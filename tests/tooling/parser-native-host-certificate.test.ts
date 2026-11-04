import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile, root } from "./support/github-workflows.ts";

void test("the adjacent-attribute certificate refuses a version-matching foreign host image", () => {
  const action = parse(readRepoFile(".github/actions/test-native-canon/action.yml")) as {
    runs: { steps: { name?: string; run?: string }[] };
  };
  const step = action.runs.steps.find(
    ({ name }) => name === "Test legal SFC attribute boundaries with the existing beta checker",
  );
  const script = step?.run?.match(
    /node --input-type=module - "\$capture" <<'NODE'\n([\s\S]*?)\nNODE(?:\n|$)/u,
  )?.[1];
  assert.ok(script, "execute the actual qualification producer");
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-parser-host-certificate-"));
  try {
    const packageRoot = path.join(temporary, "node_modules/@typescript/typescript-linux-x64");
    const binary = path.join(packageRoot, "lib/tsc");
    const capture = path.join(temporary, "capture");
    fs.mkdirSync(path.dirname(binary), { recursive: true });
    fs.mkdirSync(capture);
    fs.writeFileSync(binary, "synthetic foreign image: never executed\n");
    fs.writeFileSync(
      path.join(packageRoot, "package.json"),
      JSON.stringify({ name: "@typescript/typescript-linux-x64", version: "7.1.0-beta" }),
    );
    const name =
      "content_mapper_tsgo_cli::recovered_attribute_boundary_11731::legal_quoted_boundaries_keep_neighbor_and_unused_diagnostics";
    fs.writeFileSync(
      path.join(capture, "test.stdout"),
      `test ${name} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n`,
    );
    const cells: string[] = [];
    for (const variant of ["reported", "valid", "same-file", "unused"]) {
      for (const newline of ["\n", "\r\n"]) {
        for (const unused of [false, true]) {
          let stdout = "";
          if (variant === "same-file") {
            stdout +=
              "Gap.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.\n";
          }
          if (variant === "unused" && unused) {
            stdout +=
              "Gap.vue(3,7): error TS6133: 'unused' is declared but its value is never read.\n";
          }
          if (variant !== "valid") {
            stdout +=
              "Other.vue(2,7): error TS2322: Type 'string' is not assignable to type 'number'.\n";
          }
          cells.push(
            `11731 cell variant=${variant} newline=${JSON.stringify(newline)} noUnusedLocals=${unused} provider=${binary} status=exit status: ${variant === "valid" ? 0 : 2} stdout=${JSON.stringify(stdout)}`,
          );
        }
      }
    }
    fs.writeFileSync(path.join(capture, "test.stderr"), cells.join("\n") + "\n");
    fs.writeFileSync(path.join(capture, "test.status"), "0\n");
    const result = spawnSync(process.execPath, ["--input-type=module", "-", capture], {
      cwd: root,
      input: script,
      encoding: "utf8",
      env: {
        PATH: process.env.PATH,
        GITHUB_SHA: "f".repeat(40),
        GITHUB_RUN_ID: "1",
        GITHUB_RUN_ATTEMPT: "1",
        VIZE_TEST_RECOVERED_ATTRIBUTE_CONTENT_MAPPER_TSGO: binary,
      },
    });
    assert.equal(result.error, undefined);
    const receipt = JSON.parse(fs.readFileSync(path.join(capture, "test-run.json"), "utf8"));
    assert.equal(
      receipt.qualified,
      false,
      "a package version cannot authenticate executable bytes",
    );
    assert.notEqual(result.status, 0);
    assert.equal(
      receipt.provider,
      null,
      "a rejected image cannot become qualified provider metadata",
    );
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
