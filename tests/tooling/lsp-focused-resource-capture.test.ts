import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  packetCommand,
  runLifecycle,
  type LifecycleProcesses,
  type Observer,
  type SessionReceipt,
} from "../../tools/support/compat/github/lsp-focused-lifecycle.ts";
import { receiptJson } from "../../tools/support/compat/github/lsp-focused-campaign.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const campaignModule = new URL(
  "../../tools/support/compat/github/lsp-focused-campaign.ts",
  import.meta.url,
).pathname;

type Mode = "success" | "packet-spawn-failure" | "observer-spawn-failure";

test("the traced original packet step runs the TypeScript lifecycle", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "lsp-focused-projects.yml"));
  const step = workflow.jobs["original-projects"].steps.find(
    (candidate: { id?: string }) => candidate.id === "lifecycle",
  );
  assert.match(step.run, /^node tools\/support\/compat\/github\/lsp-focused-lifecycle\.ts$/m);
  assert.doesNotMatch(step.run, /python/);
});

test("the focused LSP workflow runs every step without Python", () => {
  const text = readRepoFile(".github", "workflows", "lsp-focused-projects.yml");
  assert.doesNotMatch(text, /python/iu);
  const steps: { run?: string }[] = parse(text).jobs["original-projects"].steps;
  const commands = steps.flatMap((step) =>
    (step.run ?? "").split("\n").filter((line) => line.includes("lsp-focused-campaign.ts")),
  );
  assert.deepEqual(
    commands.map((line) => line.split(" ").at(-1)),
    ["plan", "dependencies", "verify-resources", "refuse"],
  );
});

test("campaign receipts escape non-ASCII text like the original receipts", () => {
  assert.equal(receiptJson({ name: "café 😀" }), '{\n  "name": "caf\\u00e9 \\ud83d\\ude00"\n}\n');
});

function campaignFixture(mutation: "none" | "shard" | "executable" | "dirty"): string {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "lsp-focused-campaign-")));
  const write = (file: string, value: unknown) => {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(
      path.join(root, file),
      typeof value === "string" ? value : `${JSON.stringify(value)}\n`,
    );
  };
  const source = "a".repeat(40);
  const binary = "b".repeat(64);
  write("sampler.mjs", "export {};\n");
  write("target/ci/vize", "server");
  write("provider", "tsc");
  const samplerSha256 = createHash("sha256").update("export {};\n").digest("hex");
  const projects = [{ id: "misskey", revision: "r1", shardIndex: 12 }];
  write("c/plan.json", {
    sourceRevision: source,
    shardCount: 145,
    preparationTrace: true,
    projects,
  });
  write("c/sessions.json", [
    { id: "misskey", exitStatus: 0, wireExitStatus: 0, resourceExitStatus: 0 },
  ]);
  write("c/dependencies.json", [{ id: "misskey", exitStatus: 0 }]);
  write("target/ci/vize.differential-build.json", { sourceRevision: source, binarySha256: binary });
  write("c/misskey/resource-summary.json", {
    sourceRevision: source,
    samplerSource: "sampler.mjs",
    samplerSha256,
    rawSamples: 1,
  });
  const provider = mutation === "executable" ? "/elsewhere" : path.join(root, "provider");
  write("c/misskey/resource-samples.jsonl", {
    processes: [{ executable: path.join(root, "target/ci/vize") }, { executable: provider }],
  });
  write("c/misskey/lsp-lifecycle-summary.json", {
    commitSha: source,
    evidence: { vizeBinary: { sha256: binary } },
    shard: { index: 12, count: mutation === "shard" ? 146 : 145 },
    projects: [{ id: "misskey", revision: "r1", status: "ok" }],
  });
  const git = (...args: string[]) => execFileSync("git", args, { cwd: root, stdio: "ignore" });
  git("init", "-q");
  git("add", "-A");
  git(
    "-c",
    "user.name=t",
    "-c",
    "user.email=t@t",
    "-c",
    "commit.gpgsign=false",
    "commit",
    "-qm",
    "x",
  );
  if (mutation === "dirty") fs.appendFileSync(path.join(root, "target/ci/vize"), "changed");
  return root;
}

for (const [mutation, expected] of [
  ["none", [0, 0]],
  ["shard", [0, 1]],
  ["executable", [1, 0]],
  ["dirty", [0, 1]],
] as const) {
  test(`campaign custody checks refuse ${mutation === "none" ? "nothing" : mutation}`, () => {
    const root = campaignFixture(mutation);
    try {
      const statuses = ["verify-resources", "refuse"].map(
        (command) =>
          spawnSync(process.execPath, [campaignModule, command], {
            cwd: root,
            env: {
              ...process.env,
              CAMPAIGN_DIR: "c",
              EXPECTED_SOURCE: "a".repeat(40),
              VIZE_LSP_BIN: "target/ci/vize",
              CORSA_PATH: "provider",
            },
            stdio: "ignore",
          }).status,
      );
      assert.deepEqual(
        statuses.map((status) => (status === 0 ? 0 : 1)),
        expected,
      );
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
}

async function replay(mode: Mode) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-focused-lifecycle-"));
  try {
    const directory = path.join(root, "misskey");
    fs.mkdirSync(directory);
    const provider = path.join(root, "provider");
    fs.writeFileSync(provider, "fixture provider identity");
    fs.writeFileSync(
      path.join(root, "plan.json"),
      JSON.stringify({
        preparationTrace: true,
        shardCount: 145,
        projects: [{ id: "misskey", shardIndex: 12 }],
      }),
    );
    const calls: string[][] = [];
    const streams: number[] = [];
    const events: string[] = [];
    let status: number | null = null;
    const observer: Observer = {
      poll: () => status,
      signal: (signal) => void events.push(`signal ${signal}`),
      wait: async (timeoutMs) => {
        events.push(`wait ${timeoutMs}`);
        status = 0;
        return status;
      },
      kill: () => void events.push("kill"),
    };
    const processes: LifecycleProcesses = {
      run(command, options = {}) {
        calls.push([...command]);
        if (command === packetCommand) {
          assert.equal(options.env?.FIXTURE_SHARD_COUNT, "145");
          assert.equal(options.env?.FIXTURE_SHARD_INDEX, "12");
          assert.equal(options.env?.FIXTURE_REPORT_DIR, directory);
          streams.push(options.stdout ?? -1, options.stderr ?? -1);
          if (mode === "packet-spawn-failure") throw new Error("fixture packet spawn failure");
        }
        return 0;
      },
      observe(command, stdout, stderr) {
        assert.deepEqual(command.slice(0, 3), ["node", "--input-type=module", "--eval"]);
        assert.match(command[3] ?? "", /lsp-resource-sampler\.ts/);
        assert.deepEqual(command.slice(4), [String(process.pid), directory]);
        streams.push(stdout, stderr);
        if (mode === "observer-spawn-failure") throw new Error("fixture observer spawn failure");
        return observer;
      },
    };
    let failure: unknown = null;
    try {
      await runLifecycle(root, provider, processes);
    } catch (error) {
      failure = error;
    }
    const sessions = JSON.parse(
      fs.readFileSync(path.join(root, "sessions.json"), "utf8"),
    ) as SessionReceipt[];
    const stderr = fs.readFileSync(path.join(directory, "test.stderr"), "utf8");
    const resourceError = path.join(directory, "resource-error.txt");
    return {
      failure,
      calls,
      streams,
      events,
      receipt: sessions[0],
      stderr,
      resourceError: fs.existsSync(resourceError) ? fs.readFileSync(resourceError, "utf8") : null,
      provider: JSON.parse(fs.readFileSync(path.join(root, "provider.json"), "utf8")) as {
        path: string;
      },
      expectedProvider: fs.realpathSync(provider),
    };
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

for (const mode of ["success", "packet-spawn-failure", "observer-spawn-failure"] as const) {
  test(`the lifecycle retains ${mode} and always closes its streams`, async () => {
    const result = await replay(mode);
    assert.equal(result.provider.path, result.expectedProvider);
    if (mode === "packet-spawn-failure") {
      assert.match(String(result.failure), /all selected raw packet outcomes retained/);
    } else {
      assert.equal(result.failure, null);
    }
    assert.equal(result.calls.filter((call) => call[1] === "--test").length, 1);
    for (const fd of result.streams) assert.throws(() => fs.fstatSync(fd), /EBADF/);
    const receipt = result.receipt;
    assert.ok(receipt);
    assert.deepEqual(receipt.command, packetCommand);
    assert.equal(receipt.wireExitStatus, 0);
    if (mode === "packet-spawn-failure") {
      assert.equal(receipt.exitStatus, null);
      assert.match(receipt.commandError ?? "", /fixture packet spawn failure/);
      assert.match(result.stderr, /fixture packet spawn failure/);
    } else {
      assert.equal(receipt.exitStatus, 0);
      assert.equal(receipt.commandError, null);
    }
    if (mode === "observer-spawn-failure") {
      assert.equal(receipt.resourceExitStatus, -1);
      assert.match(result.resourceError ?? "", /fixture observer spawn failure/);
    } else {
      assert.deepEqual(result.events, ["signal SIGTERM", "wait 10000"]);
      assert.equal(receipt.resourceExitStatus, 0);
    }
  });
}
