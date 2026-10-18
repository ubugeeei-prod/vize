import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { before, test } from "node:test";
import {
  cleanup,
  readJson,
  run,
  script,
  setup,
  unusableFailure,
  updateJson,
  writeVueTsc,
} from "./_helpers/typecheck-divergence-report-fixture.ts";
import { buildRustScript } from "./support/rust-script-build.ts";

const linuxOnly = { skip: process.platform !== "linux" };
before(() => {
  if (process.platform === "linux") buildRustScript(script, ["--rust-script-build-check"]);
});
const outerHarnessTimeoutMs = 6_000;
const deadlineReason = "whole-project deadline exhausted";
function artifactPath(fixture: ReturnType<typeof setup>) {
  return path.join(fixture.reportDir, "fixture-typecheck-divergence.json");
}
function killPidFileIfPresent(pidPath: string) {
  if (!fs.existsSync(pidPath)) return;
  const raw = fs.readFileSync(pidPath, "utf8").trim(),
    pid = Number(raw);
  if (!Number.isSafeInteger(pid) || pid <= 0 || String(pid) !== raw) return;
  try {
    process.kill(pid, "SIGKILL");
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code !== "ESRCH") throw e;
  }
}
function assertDeadline(fixture: ReturnType<typeof setup>, budget: number, phase: string) {
  const roots = fs.readdirSync(path.join(fixture.reportDir, "typecheck-deadlines"));
  assert.equal(roots.length, 1);
  const directory = path.join(fixture.reportDir, "typecheck-deadlines", roots[0]!);
  const result = readJson(path.join(directory, "deadline-result.json"));
  assert.deepEqual(
    {
      schema: result.schema,
      version: result.version,
      project: result.project,
      completed: result.completed,
      instrumentFailure: result.instrumentFailure,
      budgetMs: result.budgetMs,
    },
    {
      schema: "vize.fixtureTypecheckProjectDeadline",
      version: 1,
      project: "fixture",
      completed: false,
      instrumentFailure: true,
      budgetMs: budget,
    },
  );
  assert.ok(result.elapsedMs >= budget && result.elapsedMs < outerHarnessTimeoutMs);
  assert.equal(readJson(path.join(directory, "phase-current.json")).phase, phase);
  const supervisor = readJson(path.join(directory, "supervisor.json"));
  assert.equal(supervisor.timedOut, true);
  assert.equal(supervisor.cleanup.allWritersStopped, true);
  assert.equal(
    fs.existsSync(artifactPath(fixture)),
    false,
    "an incomplete project must not masquerade as a complete v7 packet",
  );
  return directory;
}
function commandStreams(result: ReturnType<typeof run>) {
  return `stdout:\n${result.stdout ?? "(not captured)"}\nstderr:\n${result.stderr ?? "(not captured)"}`;
}
function configureBudget(fixture: ReturnType<typeof setup>, budget: number) {
  updateJson(fixture.registryPath, (registry) => {
    registry.projects[0].typecheckPerformance.hangTimeoutMs = budget;
  });
}

test(
  "typecheck divergence report bounds a vue-tsc baseline that ignores SIGTERM",
  linuxOnly,
  () => {
    const fixture = setup(),
      childPid = path.join(fixture.fakeDir, "ignored-term.pid"),
      budget = 1200;
    try {
      configureBudget(fixture, budget);
      writeVueTsc(
        fixture.vueTsc,
        String.raw`import { spawn } from "node:child_process";
process.on("SIGTERM",()=>{});
const child=spawn(process.execPath,["-e","process.on('SIGTERM',()=>{});setInterval(()=>{},1000)"],{stdio:"inherit"});
fs.writeFileSync(${JSON.stringify(childPid)},String(child.pid));
process.stdout.write("baseline-started\n"); process.stderr.write("baseline-evidence\n");
setInterval(()=>{},1000);`,
        fixture.invocationPath,
      );
      const result = run(fixture, {}, [], { timeoutMs: outerHarnessTimeoutMs });
      assert.equal(result.error, undefined, commandStreams(result));
      assert.equal(result.status, 1, commandStreams(result));
      assert.equal(result.stderr, `${unusableFailure(deadlineReason)}\n`, commandStreams(result));
      const directory = assertDeadline(fixture, budget, "vue-tsc baseline");
      assert.equal(
        fs.readFileSync(path.join(directory, "command-1-stdout.bin"), "utf8"),
        "baseline-started\n",
      );
      assert.equal(
        fs.readFileSync(path.join(directory, "command-1-stderr.bin"), "utf8"),
        "baseline-evidence\n",
      );
      assert.equal(
        fs.existsSync(childPid),
        true,
        "the real descendant must have started before expiry",
      );
    } finally {
      killPidFileIfPresent(childPid);
      cleanup(fixture);
    }
  },
);

test(
  "typecheck divergence report closes inherited stdio children after checker exit",
  linuxOnly,
  () => {
    const fixture = setup();
    const childPidPath = path.join(fixture.fakeDir, "stdio-child.pid");
    try {
      fs.writeFileSync(
        fixture.vueTsc,
        `#!/usr/bin/env node
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const child = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000);"], { stdio: "inherit" });
if (typeof child.pid === "number") fs.writeFileSync(${JSON.stringify(childPidPath)}, String(child.pid));

if (process.argv.includes("--version")) {
  console.log("3.3.4");
  process.exit(0);
}

if (process.argv.includes("--listFilesOnly")) {
  console.log(path.join(process.cwd(), "src/App.vue"));
  process.exit(0);
}

process.stdout.write("src/App.vue(1,1): error TS2322: shared\\n");
process.exit(2);
`,
      );
      fs.chmodSync(fixture.vueTsc, 0o755);

      const startedAt = Date.now();
      const result = run(fixture, {}, ["--budget-mode", "record-only"], { timeoutMs: 8_000 });
      assert.equal(result.status, 0, result.stderr);
      assert.ok(
        Date.now() - startedAt < 8_000,
        "checker output capture must not wait for inherited stdio children after exit",
      );
      assert.equal(
        fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
        true,
      );
    } finally {
      killPidFileIfPresent(childPidPath);
      cleanup(fixture);
    }
  },
);

test(
  "typecheck divergence report retains a coverage-phase project deadline as an instrument failure",
  linuxOnly,
  () => {
    const fixture = setup(),
      budget = 1200;
    try {
      configureBudget(fixture, budget);
      writeVueTsc(
        fixture.vueTsc,
        String.raw`if(process.argv.includes("--listFilesOnly")){process.stdout.write("coverage-started\n");setInterval(()=>{},1000);} else {process.stdout.write("src/App.vue(1,1): error TS2322: shared\n");process.exit(2);}`,
        fixture.invocationPath,
      );
      const result = run(fixture, {}, [], { timeoutMs: outerHarnessTimeoutMs });
      assert.equal(result.error, undefined, commandStreams(result));
      assert.equal(result.status, 1, commandStreams(result));
      assert.equal(result.stderr, `${unusableFailure(deadlineReason)}\n`, commandStreams(result));
      const directory = assertDeadline(fixture, budget, "vue-tsc coverage baseline");
      const baseline = readJson(path.join(directory, "command-1-result.json"));
      assert.equal(baseline.completed, true);
      assert.equal(baseline.exitCode, 2);
      assert.equal(
        fs.readFileSync(path.join(directory, "command-2-stdout.bin"), "utf8"),
        "coverage-started\n",
      );
      assert.equal(fs.existsSync(path.join(directory, "active-mutation.json")), false);
    } finally {
      cleanup(fixture);
    }
  },
);

test("typecheck divergence report drains verbose vue-tsc coverage output", linuxOnly, () => {
  const fixture = setup();
  try {
    updateJson(
      fixture.registryPath,
      (registry) => (registry.projects[0].typecheckPerformance.hangTimeoutMs = 5_000),
    );
    writeVueTsc(
      fixture.vueTsc,
      `if (process.argv.includes("--listFilesOnly")) {
  fs.writeSync(1, process.cwd() + "/src/App.vue\\n");
  const filler = process.cwd() + "/src/generated-support-file.ts\\n";
  for (let index = 0; index < 50_000; index++) fs.writeSync(1, filler);
  process.exit(0);
}

let output = "src/App.vue(1,1): error TS2322: shared\\n";
const source = fs.readFileSync("src/App.vue", "utf8");
const marker = "__vize_typecheck_mutation_probe: string = 1";
if (source.includes(marker)) {
  const line = source.slice(0, source.indexOf(marker)).split(/\\r?\\n/).length;
  output += \`src/App.vue(\${line},1): error TS2322: Type 'number' is not assignable to type 'string'.\\n\`;
}
process.stdout.write(output);
process.exit(2);`,
      fixture.invocationPath,
    );

    const result = run(fixture, {}, [], { timeoutMs: outerHarnessTimeoutMs });
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(artifactPath(fixture));
    assert.equal(artifact.baseline.coverageRunError, null);
    assert.equal(artifact.baseline.coverage.baselineVueFileCount, 1);
    assert.equal(artifact.baseline.coverage.verdict, "usable");
    assert.equal(artifact.budget.verdict, "passed");
  } finally {
    cleanup(fixture);
  }
});

test("record-only mode still warns and retains complete project timeout custody", linuxOnly, () => {
  const fixture = setup(),
    budget = 1200;
  try {
    configureBudget(fixture, budget);
    writeVueTsc(
      fixture.vueTsc,
      String.raw`process.on("SIGTERM",()=>{});setInterval(()=>{},1000);`,
      fixture.invocationPath,
    );
    const result = run(fixture, {}, ["--budget-mode", "record-only"], {
      timeoutMs: outerHarnessTimeoutMs,
    });
    assert.equal(result.error, undefined, commandStreams(result));
    assert.equal(result.status, 0, commandStreams(result));
    assert.equal(result.stderr, "", commandStreams(result));
    assert.ok(
      result.stdout.includes(
        `::warning title=Typecheck divergence budget not enforced::${unusableFailure(deadlineReason)}`,
      ),
      commandStreams(result),
    );
    assertDeadline(fixture, budget, "vue-tsc baseline");
  } finally {
    cleanup(fixture);
  }
});

test(
  "baseline and coverage spend one shared project allowance without resetting",
  linuxOnly,
  () => {
    const fixture = setup(),
      budget = 1200;
    try {
      configureBudget(fixture, budget);
      writeVueTsc(
        fixture.vueTsc,
        String.raw`const coverage=process.argv.includes("--listFilesOnly");process.stdout.write((coverage?"coverage":"baseline")+"-started\n");await new Promise(resolve=>setTimeout(resolve,800));if(coverage){console.log(process.cwd()+"/src/App.vue");}else{process.stdout.write("src/App.vue(1,1): error TS2322: shared\n");}process.exit(coverage?0:2);`,
        fixture.invocationPath,
      );
      const result = run(fixture, {}, [], { timeoutMs: outerHarnessTimeoutMs });
      assert.equal(result.error, undefined, commandStreams(result));
      assert.equal(result.status, 1, commandStreams(result));
      assert.equal(result.stderr, `${unusableFailure(deadlineReason)}\n`, commandStreams(result));
      const directory = assertDeadline(fixture, budget, "vue-tsc coverage baseline");
      assert.equal(readJson(path.join(directory, "command-1-result.json")).exitCode, 2);
      assert.equal(
        fs.readFileSync(path.join(directory, "command-2-stdout.bin"), "utf8"),
        "coverage-started\n",
      );
      assert.equal(fs.existsSync(path.join(directory, "command-2-result.json")), false);
    } finally {
      cleanup(fixture);
    }
  },
);

test("a deadline during a broken mutation restores all original authored bytes", linuxOnly, () => {
  const fixture = setup(),
    budget = 2000,
    source = path.join(fixture.fixtureRoot, "src/App.vue");
  const original = Buffer.from("<template>\r\n世界\r\n</template>\r\n");
  fs.writeFileSync(source, original);
  try {
    configureBudget(fixture, budget);
    const fake = fs.readFileSync(fixture.vize, "utf8");
    const marker = 'const files = listVueFiles("src").sort();';
    assert.equal(fake.split(marker).length, 2);
    fs.writeFileSync(
      fixture.vize,
      fake.replace(
        marker,
        String.raw`if(fs.readFileSync("src/App.vue","utf8").includes("__vize_typecheck_mutation_probe")){process.stderr.write("broken mutation entered\n");setInterval(()=>{},1000);await new Promise(()=>{}); }` +
          "\n" +
          marker,
      ),
    );
    const result = run(fixture, {}, [], { timeoutMs: outerHarnessTimeoutMs });
    assert.equal(result.error, undefined, commandStreams(result));
    assert.equal(result.status, 1, commandStreams(result));
    const directory = assertDeadline(fixture, budget, "Vize mutation run");
    assert.deepEqual(fs.readFileSync(source), original);
    assert.equal(fs.existsSync(path.join(directory, "active-mutation.json")), false);
    assert.equal(readJson(path.join(directory, "mutation-recovery.json")).restored, true);
    const stderr = fs
      .readdirSync(directory)
      .filter((f) => /^command-\d+-stderr\.bin$/.test(f))
      .map((f) => fs.readFileSync(path.join(directory, f), "utf8"))
      .join("");
    assert.equal(stderr, "broken mutation entered\n");
  } finally {
    cleanup(fixture);
  }
});

test("failed supervisor custody quarantines an otherwise complete v7 packet", linuxOnly, () => {
  const fixture = setup();
  try {
    const previous = fs.readFileSync(fixture.vueTsc, "utf8");
    const marker = 'if (process.argv.includes("--version"))';
    assert.equal(previous.split(marker).length, 2);
    fs.writeFileSync(
      fixture.vueTsc,
      previous.replace(
        marker,
        `import { join as joinEvidencePath } from "node:path";
const states = ${JSON.stringify(path.join(fixture.reportDir, "typecheck-deadlines"))};
` +
          `fs.mkdirSync(joinEvidencePath(states,fs.readdirSync(states)[0],"supervisor.json"),{recursive:true});
` +
          marker,
      ),
    );
    const result = run(fixture, {}, [], { timeoutMs: outerHarnessTimeoutMs });
    assert.equal(result.error, undefined, commandStreams(result));
    assert.equal(result.status, 1, commandStreams(result));
    assert.match(result.stderr, /Is a directory/, commandStreams(result));
    const states = path.join(fixture.reportDir, "typecheck-deadlines");
    const worker = readJson(path.join(states, fs.readdirSync(states)[0]!, "worker-result.json"));
    assert.equal(worker.ok, true, "the original successful flow must have completed");
    assert.equal(worker.artifact.version, 7);
    assert.equal(fs.existsSync(artifactPath(fixture)), false);
    const archived = path.join(fixture.reportDir, "previous-typecheck-comparisons", "fixture");
    assert.equal(fs.readdirSync(archived).length, 1);
    const packet = readJson(path.join(archived, fs.readdirSync(archived)[0]!, "comparison.json"));
    assert.deepEqual(
      packet,
      worker.artifact,
      "retain every completed byte without crediting custody failure",
    );
  } finally {
    cleanup(fixture);
  }
});

test("failed cleanup cannot block the supervisor on reaping or stream joins", () => {
  const source = fs.readFileSync(
    path.resolve(path.dirname(script), "../../support/typecheck_project_deadline.rs"),
    "utf8",
  );
  const start = source.indexOf("let status = match &cleanup {");
  const end = source.indexOf("// Restoration is mandatory", start);
  assert.ok(start >= 0 && end > start);
  const failure = source.slice(start, end).split("Err(_) => {")[1];
  assert.ok(failure);
  const compact = failure.replace(/\s+/g, "");
  assert.match(compact, /child\.try_wait\(\)/);
  assert.doesNotMatch(compact, /child\.wait\(\)/);
  assert.doesNotMatch(compact, /\.join\(\)/);
  const cleanup = source.indexOf("cleanup?;", end);
  assert.ok(cleanup >= 0 && source.indexOf("out.join()", end) > cleanup);
});

test(
  "real process, CPU, session ownership and physical recovery unit controls execute",
  linuxOnly,
  () => {
    const started = performance.now();
    for (let execution = 1; execution <= 8; execution++) {
      const remaining = Math.ceil(180_000 - (performance.now() - started));
      assert.ok(remaining > 0, "all control executions share the original 180-second envelope");
      console.log(`deadline control execution ${execution}/8: unfiltered Rust tests`);
      const result = spawnSync("rust-script", ["--test", script], {
        encoding: "utf8",
        timeout: remaining,
        maxBuffer: 2 * 1024 * 1024,
      });
      if (result.stdout != null) process.stdout.write(result.stdout);
      if (result.stderr != null) process.stderr.write(result.stderr);
      assert.equal(result.error, undefined, commandStreams(result));
      assert.equal(result.status, 0, result.stderr);
      for (const name of [
        "real_cpu_worker_is_stopped_and_both_raw_streams_survive",
        "sequential_real_children_share_one_parent_deadline",
        "separate_owned_process_groups_die_and_unrelated_sentinel_survives",
        "deadline_recovers_every_original_authored_byte_after_stopping_writers",
        "worker_refuses_restoration_while_a_separate_group_can_write",
        "replaced_source_identity_refuses_recovery_without_overwriting_foreign_bytes",
        "normal_worker_keeps_complete_raw_bytes_and_does_not_become_a_timeout",
        "failed_supervisor_receipt_still_restores_active_mutation",
        "unknown_authored_bytes_are_not_overwritten_by_recovery",
        "bounded_stream_retains_an_explicit_incomplete_prefix_and_refuses_success",
      ]) {
        assert.ok(result.stdout.includes(name), name);
      }
      assert.match(result.stdout, /0 failed; 0 ignored;/);
    }
  },
);
