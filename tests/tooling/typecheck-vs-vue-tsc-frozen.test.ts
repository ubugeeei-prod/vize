import assert from "node:assert/strict";
import fs from "node:fs";
import { execFileSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createRequire } from "node:module";
import {
  compareFindings,
  diagnostics,
  gnuTime,
  hash,
  inventory,
  orders,
  providerPins,
  source,
  statusConsistent,
  assertStatusStable,
  assessStderr,
} from "../../tools/benchmarks/scripts/typecheck-vs-vue-tsc-frozen-evidence.ts";
import {
  assertControl,
  controlSource,
  specification,
} from "../../tools/benchmarks/scripts/typecheck-vs-vue-tsc-frozen.ts";
import {
  command,
  environment,
} from "../../tools/benchmarks/scripts/typecheck-vs-vue-tsc-frozen-command.ts";
import {
  recordReplayBeforeInspection,
  seal,
  validateNativeArgs,
} from "../../tools/benchmarks/scripts/typecheck-vs-vue-tsc-frozen-inputs.ts";
import {
  classifyStreams,
  nativePacketClean,
  collect,
  readCall,
} from "../../tools/benchmarks/scripts/typecheck-vs-vue-tsc-frozen-native.ts";
import {
  assertInputs,
  inputSnapshot,
  sourceBodies,
  readProviderVersions,
} from "../../tools/benchmarks/scripts/typecheck-vs-vue-tsc-frozen-projects.ts";
function temporary<T>(run: (root: string) => T): T {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "comparative-pure-")));
  try {
    return run(root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}
const resource =
  "\tUser time (seconds): 1.23\n\tSystem time (seconds): 0.00\n\tMaximum resident set size (kbytes): 100\n";
test("GNU command CPU preserves quantization, zero system, duplicate/missing/malformed refusal", () => {
  assert.deepEqual(gnuTime(resource), {
    user: "1.23",
    system: "0.00",
    cpuMs: 1230,
    resolutionMs: 10,
    childHighWaterKiB: 100,
    rssScope: "GNU time child high-water; not process-tree simultaneous peak",
  });
  for (const text of [
    resource + "User time (seconds): 1.23\n",
    resource.replace("System time (seconds): 0.00", "missing"),
    resource.replace("1.23", "1.2"),
    resource.replace("1.23", "nan"),
    resource.replace("1.23", "0.00"),
    resource.replace("kbytes): 100", "kbytes): 0"),
  ])
    assert.throws(() => gnuTime(text));
});
test("exact whole packet, multiline messages, duplicate findings and unknown fields remain visible", () => {
  const packet = {
    fileCount: 1,
    errorCount: 2,
    warningCount: 0,
    programs: [{ future: "not dropped" }],
    extra: { unknown: 1 },
    files: [
      {
        file: "a.vue",
        diagnostics: ["error:6:10 [TS2339] missing", "error:6:10 [TS2339] missing"],
      },
    ],
  };
  const vize = diagnostics("vize", JSON.stringify(packet), "/fixture");
  assert.deepEqual(vize.packet, packet);
  const vue = diagnostics("vue-tsc", "a.vue(6,10): error TS2339: missing\n", "/fixture");
  const result = compareFindings(vize.findings, vue.findings);
  assert.equal(result.qualified, false);
  assert.equal(result.matched, 1);
  assert.equal(result.unmatched.length, 1);
  assert.equal(result.registryToleranceApplied, false);
  assert.equal(
    compareFindings(vue.findings, [{ ...vue.findings[0], message: "different text" }]).unmatched
      .length,
    2,
  );
  assert.deepEqual(
    diagnostics(
      "vue-tsc",
      "a.vue(6,10): error TS2322: Type X\n  detail one\n  detail two\nunparsed\n",
      "/fixture",
    ).findings[0].message,
    "Type X\n  detail one\n  detail two",
  );
  assert.deepEqual(diagnostics("vue-tsc", "unparsed\n", "/fixture").unclassified, ["unparsed"]);
  assert.throws(() =>
    diagnostics("vize", JSON.stringify({ ...packet, errorCount: 0 }), "/fixture"),
  );
});
test("balanced four pairs, direct primary argv and real first/final controls are fixed", () => {
  assert.deepEqual(orders, [
    ["vize", "vue-tsc"],
    ["vue-tsc", "vize"],
    ["vue-tsc", "vize"],
    ["vize", "vue-tsc"],
  ]);
  const project = {
    id: "x",
    cwd: "/fixture",
    root: "/fixture",
    tsconfig: "tsconfig.json",
    control: "/fixture/Control.vue",
    sourceInventory: { root: "/fixture", rows: [], sha256: "" },
  };
  assert.deepEqual(specification(project, "vize", "/vize", "/provider", "/native"), {
    executable: "/vize",
    cwd: "/fixture",
    args: [
      "check",
      "--no-config",
      "--format",
      "json",
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      "/native",
    ],
  });
  assert.deepEqual(specification(project, "vue-tsc", "/vize", "/provider", "/native").args, [
    "/provider/node_modules/vue-tsc/bin/vue-tsc.js",
    "--noEmit",
    "--pretty",
    "false",
    "-p",
    "tsconfig.json",
  ]);
  const lines = controlSource(true).split("\n");
  assert.equal(lines[5], "  <div>{{ value.definitelyMissing + acceptNumber('bad') }}</div>");
  assert.equal(lines[68], lines[5]);
  assert.equal(lines.filter((line) => /<(?:span|div)/u.test(line)).length, 64);
  const rows = [6, 69].flatMap((line) =>
    [2339, 2345].map((code) => ({
      file: project.control,
      line,
      column: 10,
      severity: "error",
      code,
      message: "retained",
      end: null,
      related: null,
    })),
  );
  const observation = {
    packet: null,
    findings: rows,
    unclassified: [],
    endAndStructuredRelated: "raw",
  };
  assertControl(observation, project.control, true);
  assert.throws(() =>
    assertControl({ ...observation, findings: rows.slice(0, 2) }, project.control, true),
  );
  assert.throws(() => assertControl(observation, project.control, false));
});
test("whole provider fixture bytes and immutable source/SDK closure stay pinned", () => {
  const fixture = new URL("../_fixtures/tooling/typecheck-vs-vue-tsc-provider/", import.meta.url);
  assert.equal(hash(fs.readFileSync(new URL("package.json.txt", fixture))), providerPins.package);
  assert.equal(hash(fs.readFileSync(new URL("package-lock.json.txt", fixture))), providerPins.lock);
  assert.notEqual(
    hash(fs.readFileSync(new URL("package.json.txt", fixture)).subarray(1)),
    providerPins.package,
  );
  assert.equal(source, "33eb41ea8a39d997383e797fea3dca5e482c0a9d");
  assert.equal(sourceBodies.length, 13);
  assert.equal(
    sourceBodies.filter(
      (row) => row.file.startsWith("npm/cli/") && row.file !== "npm/cli/package.json",
    ).length,
    10,
  );
  for (const row of sourceBodies) {
    assert.equal(hash(execFileSync("git", ["show", source + ":" + row.file])), row.sha256);
  }
});
test("raw native replay and attempted unknown invocation survive postguard refusal", () =>
  temporary((root) => {
    assert.throws(() =>
      recordReplayBeforeInspection(
        root,
        {
          stdout: Buffer.from("full raw"),
          stderr: Buffer.from("error TS5097: retained\n"),
          status: 2,
          signal: null,
        },
        "12",
        () => {
          throw new Error("changed input");
        },
      ),
    );
    assert.equal(fs.readFileSync(path.join(root, "native-replay.stdout.bin"), "utf8"), "full raw");
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(root, "native-replay.status.json"), "utf8")).status,
      2,
    );
    const observer = path.join(root, "observer/.bin/forward.ts");
    fs.mkdirSync(path.dirname(observer), { recursive: true });
    fs.writeFileSync(observer, "retained observer");
    const image = path.join(root, "image");
    fs.writeFileSync(image, "not executed");
    fs.writeFileSync(
      path.join(root, "invocation.json"),
      JSON.stringify({
        imageBefore: hash("not executed"),
        executable: image,
        nodeExecutable: fs.realpathSync(process.execPath),
        nodeVersion: process.version,
        nodeSha256: hash(fs.readFileSync(process.execPath)),
        canonicalForwarder: observer,
        argv: ["--api", "--async"],
      }),
    );
    assert.throws(() => readCall(root, image, root, root), /request.json/u);
    assert(fs.existsSync(path.join(root, "invocation.json")));
    assert.throws(() => validateNativeArgs(["--api"], "/x/tsconfig.json"));
  }));
test("potential-input seal detects whole body/mode/symlink/root changes", () =>
  temporary((root) => {
    fs.writeFileSync(path.join(root, "a.ts"), "one");
    const before = seal([root], null);
    fs.writeFileSync(path.join(root, "a.ts"), "two");
    assert.notDeepEqual(seal([root], null), before);
    fs.writeFileSync(path.join(root, "extra.ts"), "extra");
    assert.notDeepEqual(seal([root], null), before);
    fs.symlinkSync("a.ts", path.join(root, "alias"));
    assert.notDeepEqual(seal([root], null), before);
    fs.symlinkSync(os.tmpdir(), path.join(root, "escape"));
    assert.throws(() => seal([root], null), /escaped/u);
  }));
test("unexpected policy refuses before environment projection and unrelated secrets are absent", () =>
  temporary((root) => {
    for (const input of [
      { GOGC: "500" },
      { NODE_OPTIONS: "--disable-warning=DEP0040 --disable-warning=DEP0169" },
    ])
      assert.throws(() => environment(root, input), /unexpected inherited policy/u);
    const env = environment(root, { PATH: process.env.PATH, GITHUB_TOKEN: "fixture-secret" });
    assert.equal(env.HOME, path.join(root, "home"));
    assert.equal(env.LC_ALL, "C");
    assert.equal((env as NodeJS.ProcessEnv).GITHUB_TOKEN, undefined);
  }));

test("command failure keeps complete streams/status before any verdict", async () => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "comparative-command-")));
  try {
    const fake = path.join(root, "fake.cjs");
    fs.writeFileSync(
      fake,
      "#!" +
        process.execPath +
        "\nprocess.stdout.write('retained body');process.stderr.write('retained error');process.exitCode=3;\n",
    );
    fs.chmodSync(fake, 0o755);
    const out = path.join(root, "call"),
      env = environment(path.join(root, "cache"), { PATH: process.env.PATH });
    await assert.rejects(
      command(out, { executable: process.execPath, args: [fake], cwd: root }, env, 10_000, fake),
    );
    assert.equal(fs.readFileSync(path.join(out, "stdout.bin"), "utf8"), "retained body");
    assert.equal(fs.readFileSync(path.join(out, "stderr.bin"), "utf8"), "retained error");
    assert.equal(JSON.parse(fs.readFileSync(path.join(out, "process.json"), "utf8")).status, 3);
    assert(inventory(root).rows.length > 0);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
test("only manual frontdoor executes the fixed cohort; all failure raw is uploaded", () => {
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/typecheck-vs-vue-tsc.yml", import.meta.url),
    "utf8",
  );
  assert.match(workflow, /if: \$\{\{ github.event_name == 'workflow_dispatch' \}\}/u);
  assert.match(workflow, /git worktree add --detach.*33eb41ea8a39d997383e797fea3dca5e482c0a9d/u);
  assert.match(workflow, /node-version: "24.14.0"/u);
  assert.match(workflow, /toolchain: "1.99.1"/u);
  assert.match(workflow, /--exclude=target/u);
  assert.match(workflow, /--exclude=provider\/node_modules/u);
  assert.doesNotMatch(workflow, /latest|schedule:|maxFalse|drop_caches/u);
});

test("two sibling cohorts preserve independent complete observation receipts", () =>
  temporary((root) => {
    for (const name of ["retained-2500-calls", "misskey-810faa-calls"]) {
      const calls = path.join(root, name);
      fs.mkdirSync(calls);
      const result = collect(calls, "/unused-native", root, root);
      assert.equal(result.complete, false);
      assert.deepEqual(
        JSON.parse(fs.readFileSync(path.join(root, name + "-observations.json"), "utf8")),
        result,
      );
    }
    assert.deepEqual(fs.readdirSync(root).sort(), [
      "misskey-810faa-calls",
      "misskey-810faa-calls-observations.json",
      "retained-2500-calls",
      "retained-2500-calls-observations.json",
    ]);
  }));

test("whole physical source/config/dependency/JS seals refuse same-size mutations", () =>
  temporary((root) => {
    const project = path.join(root, "project"),
      provider = path.join(root, "provider");
    fs.mkdirSync(project);
    fs.mkdirSync(provider);
    fs.mkdirSync(path.join(project, "node_modules"));
    const files = [
      path.join(project, "source.ts"),
      path.join(project, "tsconfig.json"),
      path.join(project, "node_modules/dependency.js"),
      path.join(provider, "typescript.js"),
    ];
    for (const file of files) fs.writeFileSync(file, "alpha");
    const before = inputSnapshot(project, provider);
    for (const file of files) {
      fs.writeFileSync(file, "bravo");
      assert.throws(
        () => assertInputs(before, inputSnapshot(project, provider), []),
        /identity changed/u,
      );
      fs.writeFileSync(file, "alpha");
    }
    const output = path.join(project, "declared.tsbuildinfo");
    fs.writeFileSync(output, "cache");
    const after = inputSnapshot(project, provider);
    assertInputs(before, after, [output]);
    assert(after.project.rows.some((row) => row.path === "declared.tsbuildinfo"));
    fs.unlinkSync(output);
    fs.symlinkSync("source.ts", output);
    assert.throws(
      () => assertInputs(before, inputSnapshot(project, provider), [output]),
      /cannot mask/u,
    );
  }));
test("status drift, clean-output failure and every unknown stream remain unqualified", () => {
  const clean = diagnostics("vue-tsc", "", "/fixture");
  assert.equal(statusConsistent(0, clean), true);
  assert.equal(statusConsistent(2, clean), false);
  assert.throws(() => assertStatusStable(0, 2), /status drift/u);
  assert.deepEqual(assessStderr("mystery runtime note\n").unresolvedLines, [
    "mystery runtime note",
  ]);
  const streams = classifyStreams(
    Buffer.from("no terminal LF"),
    Buffer.from("error TS5097: retained\n"),
    "/fixture",
    "/fixture",
    "/provider",
  );
  assert.equal(streams.stdout.rows[0].raw, "no terminal LF");
  assert.deepEqual(streams.stderr.groups, [['["5097","global"]', 1]]);
  assert.equal(nativePacketClean(0, streams), false);
  assert.equal(
    nativePacketClean(
      0,
      classifyStreams(Buffer.alloc(0), Buffer.alloc(0), "/fixture", "/fixture", "/provider"),
    ),
    true,
  );
});

test("physical pinned package metadata remains readable when exports hides package.json", () =>
  temporary((root) => {
    for (const name of [
      "vue",
      "vue-tsc",
      "typescript",
      "typescript-native",
      "vize",
      "@vizejs/native",
    ]) {
      const dir = path.join(root, "node_modules", name);
      fs.mkdirSync(dir, { recursive: true });
      fs.writeFileSync(
        path.join(dir, "package.json"),
        JSON.stringify({ name, version: "1.2.3", exports: { ".": "./entry.cjs" } }),
      );
    }
    const require = createRequire(path.join(root, "package.json"));
    assert.throws(() => require.resolve("vize/package.json"), /not defined by "exports"/u);
    assert.deepEqual(readProviderVersions(root), {
      vue: "1.2.3",
      "vue-tsc": "1.2.3",
      typescript: "1.2.3",
      "typescript-native": "1.2.3",
      vize: "1.2.3",
      "@vizejs/native": "1.2.3",
    });
  }));
