import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

interface WorkflowStep {
  name?: string;
  if?: string;
  uses?: string;
  run?: string;
  with?: Record<string, unknown>;
}

interface WorkflowJob {
  steps?: WorkflowStep[];
}

test("check workflow uploads Davinci allocation bench reports", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const action = readRepoFile(
    ".github",
    "actions",
    "upload-davinci-compact-storage-bench-reports",
    "action.yml",
  );
  const jobs = (parse(workflow) as { jobs?: Record<string, WorkflowJob> }).jobs ?? {};
  const actionSteps = (parse(action) as { runs?: { steps?: WorkflowStep[] } }).runs?.steps ?? [];
  const steps = jobs["clippy-and-test"]?.steps ?? [];
  const gateIndex = steps.findIndex((step) => step.name === "Davinci allocation gate");

  assert.notEqual(gateIndex, -1);
  assert.match(
    steps[gateIndex].run ?? "",
    /cargo bench -p vize_s1_to_s2 --bench davinci_storage -- --quick/,
  );
  assert.match(
    steps[gateIndex].run ?? "",
    /cargo bench -p vize_patina --bench davinci_markup -- --quick/,
  );
  assert.match(
    steps[gateIndex].run ?? "",
    /rust-script tools\/commands\/davinci\/bench-compare\.rs --bench s1_to_s2_lower_vfor_three_aliases --bench s1_to_s2_emit_von_two_per_bucket --bench s1_to_s2_emit_p2_11_dom_surface --bench s1_to_s2_pass_template_complexity --bench patina_jsx_markup_one_root --bench patina_s2_markup_one_root/,
  );
  assert.match(
    steps[gateIndex].run ?? "",
    /cargo bench -p vize_musea --bench davinci_art -- --quick && rust-script tools\/commands\/davinci\/bench-compare\.rs --bench musea_parse_art_define_art --bench musea_parse_art_legacy_attrs --bench musea_parse_art_inline$/,
  );
  assert.deepEqual(steps[gateIndex + 1], {
    name: "Upload Davinci allocation bench reports",
    if: "${{ always() }}",
    uses: "./.github/actions/upload-davinci-compact-storage-bench-reports",
  });
  assert.deepEqual(actionSteps, [
    {
      name: "Upload reports",
      uses: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
      with: {
        name: "davinci-allocation-bench-reports",
        path: "tools/benchmarks/results/davinci/*.json",
        "if-no-files-found": "error",
        "retention-days": 14,
      },
    },
  ]);
  assert.match(
    action,
    /actions\/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a\s*# v7\.0\.1/,
  );
});

test("Vapor Criterion dispatch validates revisions and selects only its suite", (t) => {
  const workflow = parse(readRepoFile(".github", "workflows", "criterion-bench.yml")) as {
    on?: Record<string, { inputs?: Record<string, unknown> }>;
    jobs?: Record<string, WorkflowJob>;
  };
  assert.deepEqual(Object.keys(workflow.on ?? {}), ["workflow_dispatch"]);
  assert.ok(workflow.on?.workflow_dispatch?.inputs?.vapor_only);
  const steps = workflow.jobs?.["criterion-ab"]?.steps ?? [];
  const validation = steps.find((step) => step.name === "Validate exact benchmark commits")?.run;
  const impact = steps.find((step) => step.name === "Select affected Criterion suites")?.run;
  assert.ok(validation);
  assert.ok(impact);

  const root = mkdtempSync(join(tmpdir(), "vize-vapor-bench-workflow-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const source = join(root, "source");
  const base = join(root, "base");
  const head = join(root, "head");
  mkdirSync(source);
  git(source, ["init", "-q", "--initial-branch=main"]);
  const commit = (message: string) => {
    git(source, ["add", "."]);
    git(source, [
      "-c",
      "user.name=Vize",
      "-c",
      "user.email=vize@example.com",
      "commit",
      "-qm",
      message,
    ]);
    return git(source, ["rev-parse", "HEAD"]);
  };
  writeFileSync(join(source, "fixture"), "base");
  const baseSha = commit("base");
  writeFileSync(join(source, "fixture"), "head");
  const headSha = commit("head");
  writeFileSync(join(source, "fixture"), "later");
  const laterSha = commit("later");
  git(source, ["checkout", "-q", "--orphan", "unrelated"]);
  git(source, ["rm", "-q", "-rf", "."]);
  writeFileSync(join(source, "unrelated"), "other history");
  const unrelatedSha = commit("unrelated");
  git(root, ["clone", "-q", source, base]);
  git(root, ["clone", "-q", source, head]);
  git(base, ["checkout", "-q", baseSha]);
  git(head, ["checkout", "-q", headSha]);

  const validate = (baseValue: string, headValue: string, dispatchValue: string) =>
    spawnSync("bash", ["-e", "-o", "pipefail", "-c", validation], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        BASE_SHA: baseValue,
        HEAD_SHA: headValue,
        DISPATCH_SHA: dispatchValue,
      },
    });
  assert.equal(validate(baseSha, headSha, headSha).status, 0);
  assert.notEqual(validate(baseSha, headSha, baseSha).status, 0);
  assert.notEqual(validate(baseSha, laterSha, laterSha).status, 0);
  git(base, ["checkout", "-q", unrelatedSha]);
  assert.notEqual(validate(unrelatedSha, headSha, headSha).status, 0);

  const output = join(root, "github-output");
  const selection = join(root, "criterion-impact.json");
  const env = {
    ...process.env,
    GITHUB_WORKSPACE: root,
    GITHUB_OUTPUT: output,
    PR_BASE_SHA: baseSha,
    PR_HEAD_SHA: headSha,
    VAPOR_ONLY: "true",
  };
  const vapor = spawnSync("bash", ["-e", "-o", "pipefail", "-c", impact], {
    cwd: root,
    encoding: "utf8",
    env,
  });
  assert.equal(vapor.status, 0, vapor.stderr);
  assert.deepEqual(JSON.parse(readFileSync(selection, "utf8")).selected, ["vize_atelier_vapor"]);
  assert.match(readFileSync(output, "utf8"), /has_suites=true/);

  const bin = join(root, "bin");
  mkdirSync(bin);
  writeFileSync(
    join(bin, "node"),
    '#!/bin/sh\nprintf \'%s\\n\' \'{"selected":["vize_glyph"],"reason":"from impact script"}\' > "$GITHUB_WORKSPACE/criterion-impact.json"\nprintf \'%s\\n\' \'has_suites=true\' >> "$GITHUB_OUTPUT"\n',
    { mode: 0o755 },
  );
  const affected = spawnSync("bash", ["-e", "-o", "pipefail", "-c", impact], {
    cwd: root,
    encoding: "utf8",
    env: { ...env, VAPOR_ONLY: "false", PATH: `${bin}${delimiter}${process.env.PATH ?? ""}` },
  });
  assert.equal(affected.status, 0, affected.stderr);
  assert.deepEqual(JSON.parse(readFileSync(selection, "utf8")).selected, ["vize_glyph"]);
});

function git(cwd: string, args: string[]): string {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return result.stdout.trim();
}
