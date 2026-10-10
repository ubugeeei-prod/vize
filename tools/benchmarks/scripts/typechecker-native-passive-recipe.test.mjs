import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import { join } from "node:path";
import test from "node:test";
import { currentRequiredRun } from "./typechecker-native-source-recipe-fixture.mjs";

await test("passive original-law command is scoped and both native failures stop before downstream work", () => {
  const root = mkdtempSync(join(os.tmpdir(), "passive-recipe-control-"));
  try {
    const journal = join(root, "journal.jsonl");
    const mock = `#!${process.execPath}
import {appendFileSync} from "node:fs";
const name=process.argv[1].split("/").at(-1), args=process.argv.slice(2);
const passive=args.includes("lsp_reactive_diagnostics_cli");
appendFileSync(process.env.JOURNAL,JSON.stringify({name,args,capture:process.env.VIZE_INLAY_HINT_CAPTURE??null,passive:process.env.VIZE_LSP_PASSIVE_EVIDENCE??null,trace:process.env.VIZE_TRACE_EDITOR_PREPARATION??null})+"\\n");
if(name==="cargo" && (passive?process.env.FAIL_PASSIVE:process.env.FAIL_ORIGINAL)==="1")process.exit(passive?31:29);
if(name==="node")process.stdout.write("/authored/native");
`;
    for (const name of ["cargo", "node", "bash"]) {
      writeFileSync(join(root, name), mock);
      chmodSync(join(root, name), 0o755);
    }
    const recipe = join(root, "recipe.sh");
    writeFileSync(recipe, currentRequiredRun);
    const run = (failure) => {
      writeFileSync(journal, "");
      const result = spawnSync(
        "/bin/bash",
        ["--noprofile", "--norc", "-e", "-o", "pipefail", recipe],
        {
          env: {
            ...process.env,
            PATH: root + ":" + process.env.PATH,
            JOURNAL: journal,
            NATIVE_PHASE_SOURCE_ROOT: root,
            RUNNER_TEMP: root,
            VIZE_INLAY_HINT_CAPTURE: "original-capture-control",
            VIZE_LSP_PASSIVE_EVIDENCE: "original-passive-control",
            VIZE_TRACE_EDITOR_PREPARATION: "original-trace-control",
            FAIL_ORIGINAL: failure === "original" ? "1" : "0",
            FAIL_PASSIVE: failure === "passive" ? "1" : "0",
          },
          encoding: "utf8",
        },
      );
      assert.equal(result.error, undefined);
      const rows = readFileSync(journal, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
      return { result, rows };
    };
    const first = run("original");
    assert.equal(first.result.status, 29, first.result.stderr);
    assert.deepEqual(
      first.rows.map((row) => row.name),
      ["cargo"],
    );
    const second = run("passive");
    assert.equal(second.result.status, 31, second.result.stderr);
    assert.deepEqual(
      second.rows.map((row) => row.name),
      ["cargo", "cargo"],
    );
    assert.deepEqual(second.rows[0], first.rows[0]);
    assert.deepEqual(
      second.rows[1].args.filter((_, i, all) => all[i - 1] === "--test"),
      ["lsp_passive_evidence_cli", "lsp_reactive_diagnostics_cli"],
    );
    assert.deepEqual(
      {
        capture: second.rows[1].capture,
        passive: second.rows[1].passive,
        trace: second.rows[1].trace,
      },
      { capture: null, passive: join(root, "lsp-passive-3952"), trace: "1" },
    );
    const success = run("none");
    assert.equal(success.result.status, 0, success.result.stderr);
    assert.deepEqual(success.rows.slice(0, 2), second.rows);
    for (const row of success.rows.slice(2))
      assert.deepEqual(
        { capture: row.capture, passive: row.passive, trace: row.trace },
        {
          capture: "original-capture-control",
          passive: "original-passive-control",
          trace: "original-trace-control",
        },
      );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
