import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { checkWorkflowJobSources, readRepoFile } from "./support/github-workflows.ts";

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
  const workflow = checkWorkflowJobSources();
  const action = readRepoFile(
    ".github",
    "actions",
    "upload-davinci-compact-storage-bench-reports",
    "action.yml",
  );
  const jobs = (parse(workflow) as { jobs?: Record<string, WorkflowJob> }).jobs ?? {};
  const actionSteps = (parse(action) as { runs?: { steps?: WorkflowStep[] } }).runs?.steps ?? [];
  const steps = jobs["rust-davinci-allocations"]?.steps ?? [];
  const gateIndex = steps.findIndex((step) => step.name === "Davinci allocation gate");

  assert.notEqual(gateIndex, -1);
  assert.match(
    steps[gateIndex].run ?? "",
    /cargo bench -p vize_l1_to_l2 --bench davinci_storage -- --quick/,
  );
  assert.match(
    steps[gateIndex].run ?? "",
    /cargo bench -p vize_patina --bench davinci_markup -- --quick/,
  );
  assert.match(
    steps[gateIndex].run ?? "",
    /rust-script tools\/commands\/davinci\/bench-compare\.rs --bench s1_to_s2_lower_vfor_three_aliases --bench s1_to_s2_emit_von_two_per_bucket --bench s1_to_s2_emit_p2_11_dom_surface --bench s1_to_s2_emit_helper_rich_large --bench s1_to_s2_pass_template_complexity --bench patina_jsx_markup_one_root --bench patina_s2_markup_one_root/,
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

test("editor script qualification runs the original readers and paired measurements on tracked inputs", (t) => {
  const workflow = parse(readRepoFile(".github", "workflows", "editor-script-queries.yml")) as {
    jobs: Record<string, WorkflowJob>;
  };
  const root = mkdtempSync(join(tmpdir(), "vize-editor-script-queries-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  git(root, ["init", "-q"]);
  const sources = ["App.vue", "two lines\nin this name.vue.txt"];
  for (const name of [...sources, "notes.txt", "untracked.vue"])
    writeFileSync(join(root, name), "");
  git(root, ["add", "--", ...sources, "notes.txt"]);
  const bin = join(root, "bin");
  mkdirSync(bin);
  const log = join(root, "cargo.jsonl");
  writeFileSync(
    join(bin, "cargo"),
    [
      "#!/usr/bin/env node",
      "const fs = require('node:fs');",
      "const input = process.env.VIZE_SCRIPT_QUERY_INPUTS;",
      "const manifest = input && fs.readFileSync(input, 'utf8').split('\\0');",
      "fs.appendFileSync(process.env.CARGO_INVOCATIONS, JSON.stringify({args:process.argv.slice(2),manifest})+'\\n');",
    ].join("\n"),
    { mode: 0o755 },
  );
  for (const step of workflow.jobs["editor-products"].steps ?? []) {
    if (!step.run) continue;
    const result = spawnSync("bash", ["-e", "-o", "pipefail", "-c", step.run], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        RUNNER_TEMP: root,
        PATH: bin + delimiter + process.env.PATH,
        CARGO_INVOCATIONS: log,
        VIZE_SCRIPT_QUERY_INPUTS: "",
      },
    });
    assert.equal(result.status, 0, result.stderr);
  }
  const calls: { args: string[]; manifest?: string[] }[] = readFileSync(log, "utf8")
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line));
  function invocation(
    command: string,
    pkg: string,
    features: string[],
    required: string[],
    target = "--lib",
  ) {
    const call = calls.find(({ args }) => {
      const enabled = args[args.indexOf("--features") + 1]?.split(/[ ,]+/);
      return (
        args[0] === command &&
        args[args.indexOf("-p") + 1] === pkg &&
        features.every((feature) => enabled?.includes(feature)) &&
        required.every((flag) => args.includes(flag))
      );
    });
    assert.ok(call, JSON.stringify({ command, pkg, features, required }));
    assert.ok(call.args.includes("--locked"));
    assert.equal(call.args[call.args.indexOf("--profile") + 1], "ci-opt");
    assert.ok(call.args.includes(target));
    return call;
  }
  const editor = ["experimental-script-queries", "micro-profiling"];
  const lint = ["--no-deps", "-D", "warnings"];
  invocation("clippy", "vize_l1", ["authored-script-queries"], lint);
  invocation("clippy", "vize_maestro", editor, lint);
  for (const suite of ["server::document_structure::symbols::tests", "ide::file_rename::"]) {
    invocation("test", "vize_maestro", editor, [suite]);
  }
  const scriptRules = ["authored-script-rules"];
  invocation("clippy", "vize_patina", scriptRules, lint);
  invocation("test", "vize_patina", scriptRules, ["rules::script::authored"], "--doc");
  const checker = ["authored-script-diagnostics"];
  const combined = ["experimental-checker-script-queries", "micro-profiling"];
  invocation("clippy", "vize_canon", checker, lint);
  invocation("clippy", "vize_maestro", combined, lint);
  for (const corpus of [
    invocation("test", "vize_patina", scriptRules, ["authored_script_rules"], "--test"),
    invocation("test", "vize_maestro", editor, ["source_project::script_queries::"]),
    invocation("test", "vize_canon", checker, ["script_parse::"]),
    invocation("test", "vize_maestro", combined, ["source_project::script_queries::"]),
  ]) {
    assert.deepEqual(corpus.manifest, [...sources, ""]);
    assert.ok(corpus.args.includes("--"));
    const options = corpus.args.slice(corpus.args.indexOf("--") + 1);
    for (const flag of ["--include-ignored", "--nocapture", "--test-threads=1"]) {
      assert.ok(options.includes(flag));
    }
  }
});

test("client and server target qualification exercises original inputs and serialized allocation windows", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "client-server-decisions.yml")) as {
    jobs: Record<string, WorkflowJob>;
  };
  const commands = workflow.jobs["target-products"].steps?.find((step) =>
    step.run?.includes("VIZE_CLIENT_SERVER_INPUTS"),
  )?.run;
  assert.ok(commands);
  assert.match(
    commands,
    /suite=\(-p vize_atelier_sfc --features vize_l4\/client-server-decisions --test client_server_decisions\)/,
  );
  assert.match(commands, /export VIZE_CLIENT_SERVER_INPUTS="\$RUNNER_TEMP\/[^"]+"/);
  assert.match(
    commands,
    /git ls-files -z -- '\*\.vue' '\*\.vue\.txt' > "\$VIZE_CLIENT_SERVER_INPUTS"/,
  );
  const publication = commands
    .split("\n")
    .find((line) => line.includes("VIZE_CLIENT_SERVER_REQUIRE_OUTPUT=1"));
  assert.ok(publication);
  for (const flag of [
    "VIZE_CLIENT_SERVER_TIMINGS=1",
    "cargo test",
    '"${suite[@]}"',
    "--include-ignored",
    "--test-threads=1",
  ]) {
    assert.ok(publication.includes(flag), flag);
  }
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
