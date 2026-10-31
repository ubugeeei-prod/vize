import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { mkdir, open } from "node:fs/promises";
import path from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";

type Row = Record<string, unknown>;
const sha = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const record = (value: unknown): Row => {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Row;
};

/** Public process events deliberately omit argv, environment, syscall operands and signals' data. */
export function publicEvent(line: string): Row | undefined {
  const prefix = /^(\d+)\s+([0-9:.]+)\s+(.*)$/.exec(line);
  if (!prefix) return;
  const [, pidText, at, body] = prefix;
  const pid = Number(pidText);
  const exec = /^(execve|execveat)\((?:[^,]*,\s*)?("(?:\\.|[^"\\])*")/.exec(body);
  if (exec) {
    const executable: unknown = JSON.parse(exec[2]);
    assert.equal(typeof executable, "string");
    return { event: "exec", pid, at, executable, completed: / = 0$/.test(body) };
  }
  const child = /^(?:clone3?|fork|vfork|<\.\.\. (?:clone3?|fork|vfork) resumed>).* = (\d+)$/.exec(
    body,
  );
  if (child) return { event: "child", parent: pid, pid: Number(child[1]), at };
  const exited = /^\+\+\+ exited with (\d+) \+\+\+$/.exec(body);
  if (exited) return { event: "exit", pid, at, status: Number(exited[1]) };
  const killed = /^\+\+\+ killed by (SIG[A-Z0-9]+)(?: \(core dumped\))? \+\+\+$/.exec(body);
  if (killed) return { event: "signal-exit", pid, at, signal: killed[1] };
  const signal = /^--- (SIG[A-Z0-9]+) /.exec(body);
  if (signal) return { event: "signal", pid, at, signal: signal[1] };
}

export function publicFrames(text: string): Row[] {
  return text
    .split("\n")
    .flatMap((line) => {
      const frame = /^#(\d+)\s+(0x[0-9a-f]+)\b/i.exec(line);
      if (!frame) return [];
      return [
        {
          frame: Number(frame[1]),
          pc: frame[2],
          module: /\sfrom\s+(\S+)$/.exec(line)?.[1] ?? null,
        },
      ];
    })
    .slice(0, 64);
}

async function fileIdentity(filename: string) {
  const hash = createHash("sha256");
  for await (const bytes of fs.createReadStream(filename)) hash.update(bytes);
  return { bytes: fs.statSync(filename).size, sha256: hash.digest("hex") };
}

export async function main(): Promise<void> {
  const output = path.resolve(process.argv[2]);
  const scratch = path.resolve(process.argv[3]);
  const source = process.cwd();
  assert.notEqual(output, scratch);
  assert.equal(process.platform, "linux");
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  await mkdir(output, { recursive: true });
  await mkdir(scratch, { recursive: true });
  const git = (...args: string[]) => {
    const result = spawnSync("git", args, { encoding: "utf8", maxBuffer: 1024 * 1024 });
    assert.equal(result.status, 0, "git identity read failed");
    return result.stdout.trim();
  };
  const sourceSha = "4a6e04d35cfbb3b68a437d81d970d971805243c0";
  const tree = "6176a1ff8a4d88a3f16a0367708f27fcf0344e82";
  const parents = [
    "2b6d84cd6839273876e8740fc0f3c2a3324e99ec",
    "adf15f42d84bab10bb3a61ad9b8fcc62e9b58a30",
  ];
  assert.equal(git("rev-parse", "HEAD"), sourceSha);
  assert.equal(git("rev-parse", "HEAD^{tree}"), tree);
  assert.equal(git("rev-list", "--parents", "-n", "1", "HEAD"), [sourceSha, ...parents].join(" "));
  assert.equal(git("diff", "--name-only", "HEAD"), "");
  assert.equal(
    git("rev-parse", "HEAD:npm/framework/nuxt/src/nuxt2-entry.test.ts"),
    "e90b2b5822d1cfc4ddb08cb25b816f1c618fe15d",
  );
  assert.equal(
    git("rev-parse", "HEAD:npm/framework/nuxt/package.json"),
    "93c6ac02978575b7b4e1aa0868429aaed180de4a",
  );
  const observer = ".github/actions/test-js-packages-with-history/native-fault-evidence.ts";
  assert.equal(
    sha(fs.readFileSync(observer)),
    "7ff68ac4c464b1a4a364be61a5b75d9e95f0ffca3add3d2338b4e734b930f5c3",
  );
  assert.equal(process.env.VIZE_JS_PACKAGE_GROUP, "builders");
  assert.equal(process.env.CARGO_INCREMENTAL, "0");
  assert.equal(process.env.CARGO_BUILD_JOBS, "12");
  for (const key of ["RUST_MIN_STACK", "TOKIO_WORKER_THREADS"])
    assert.equal(process.env[key], undefined, "no stack or worker-count override");
  const sourcePacket = {
    schema: "nuxt-pack-attribution-v1",
    sourceSha,
    tree,
    parents,
    originalRun: 41193345933,
    originalJob: 124514941142,
    driver: process.env.GITHUB_WORKFLOW_SHA,
    run: process.env.GITHUB_RUN_ID,
    observerSha256: sha(fs.readFileSync(observer)),
    helperSha256: sha(fs.readFileSync(fileURLToPath(import.meta.url))),
    commandAuthority:
      ".github/workflows/pr-js-checks.yml: Test JS package group; builders partition, unchanged arguments",
    packageTestBlob: "e90b2b5822d1cfc4ddb08cb25b816f1c618fe15d",
    packageManifestBlob: "93c6ac02978575b7b4e1aa0868429aaed180de4a",
    locks: Object.fromEntries(
      ["Cargo.lock", "pnpm-lock.yaml"].map((name) => [name, sha(fs.readFileSync(name))]),
    ),
    privacy:
      "raw process trace, command streams, cores and images remain runner-local; publish only selected event identities/hashes",
    regime:
      "one controlled observation; no cache clearing, stack override, preloader disable, dependency change or retry",
  };
  fs.writeFileSync(path.join(output, "source.json"), JSON.stringify(sourcePacket, null, 2) + "\n");
  const trace = path.join(scratch, "processes.strace");
  const stdout = await open(path.join(scratch, "stdout"), "w");
  const stderr = await open(path.join(scratch, "stderr"), "w");
  const startedAt = new Date().toISOString();
  const originalEnvironment = { ...process.env };
  for (const key of [
    "VIZE_CSS_MAP_SOURCE_SHA",
    "VIZE_CSS_MAP_PR_HEAD",
    "VIZE_CSS_MAP_REQUIRE_SOURCE",
  ])
    delete originalEnvironment[key];
  const result = await new Promise<{ status: number | null; signal: NodeJS.Signals | null }>(
    (resolve, reject) => {
      const child = spawn(
        "strace",
        [
          "-f",
          "-tt",
          "-e",
          "trace=process",
          "-o",
          trace,
          "node",
          observer,
          "--",
          "vp",
          "run",
          "--workspace-root",
          "test:js:builders",
        ],
        { env: originalEnvironment, stdio: ["ignore", stdout.fd, stderr.fd] },
      );
      child.once("error", reject);
      child.once("close", (status, signal) => resolve({ status, signal }));
    },
  );
  await stdout.close();
  await stderr.close();
  const finishedAt = new Date().toISOString();
  const events = fs.createWriteStream(path.join(output, "process-events.jsonl"));
  const executables = new Set<string>();
  let eventCount = 0;
  for await (const line of createInterface({
    input: fs.createReadStream(trace),
    crlfDelay: Infinity,
  })) {
    const event = publicEvent(line);
    if (!event) continue;
    events.write(JSON.stringify(event) + "\n");
    eventCount++;
    if (
      event.event === "exec" &&
      typeof event.executable === "string" &&
      /\/(?:vp|pnpm(?:\.native|\.cjs)?|node)$/.test(event.executable)
    )
      executables.add(event.executable);
  }
  await new Promise<void>((resolve, reject) => {
    events.once("error", reject);
    events.end(resolve);
  });
  const images = [];
  for (const executable of executables) {
    if (!fs.existsSync(executable) || !fs.statSync(executable).isFile()) continue;
    images.push({
      executable,
      realpath: fs.realpathSync(executable),
      ...(await fileIdentity(executable)),
    });
  }
  const observerDirectory = path.join(
    process.env.RUNNER_TEMP!,
    `js-native-fault-evidence-${process.env.GITHUB_RUN_ID}-${process.env.GITHUB_RUN_ATTEMPT}-${process.env.GITHUB_JOB}`,
  );
  let failureObserver: Row = { unavailable: true };
  const outcome = path.join(observerDirectory, "outcome.json");
  if (fs.existsSync(outcome)) {
    const value = record(JSON.parse(fs.readFileSync(outcome, "utf8")) as unknown);
    failureObserver = {
      coreCapture: value.coreCapture,
      sampledMapPackets: value.sampledMapPackets,
      coreCount: Array.isArray(value.cores) ? value.cores.length : null,
      outcomeAuthority: await fileIdentity(outcome),
    };
    const processDirectory = path.join(observerDirectory, "processes");
    const processImages = [];
    if (fs.existsSync(processDirectory))
      for (const name of fs.readdirSync(processDirectory)) {
        if (!name.endsWith(".json")) continue;
        const packet = record(
          JSON.parse(fs.readFileSync(path.join(processDirectory, name), "utf8")) as unknown,
        );
        const providers = Array.isArray(packet.providers) ? packet.providers.map(record) : [];
        processImages.push({
          pid: packet.pid,
          executable: packet.executable,
          providers: providers
            .filter(
              (row) =>
                typeof row.path === "string" &&
                (/\/fspy\//.test(row.path) || row.path === packet.executable),
            )
            .map((row) => ({
              path: row.path,
              sha256: row.sha256,
              bytes: row.bytes,
              mappedInode: row.mappedInode,
            })),
        });
      }
    fs.writeFileSync(
      path.join(output, "sampled-process-images.json"),
      JSON.stringify(processImages, null, 2) + "\n",
    );
    // Existing guarded core authority already disables frame arguments. Publish PCs/modules only.
    if (Array.isArray(value.cores))
      for (const [index, item] of value.cores.entries()) {
        const core = record(item);
        let frames: Row[] = [];
        const gdb =
          typeof core.file === "string"
            ? path.join(observerDirectory, `${path.basename(core.file)}-gdb.json`)
            : null;
        if (gdb && fs.existsSync(gdb)) {
          const packet = record(JSON.parse(fs.readFileSync(gdb, "utf8")) as unknown);
          if (typeof packet.stdout === "string") frames = publicFrames(packet.stdout);
        }
        fs.writeFileSync(
          path.join(output, `core-${index}.json`),
          JSON.stringify(
            {
              bytes: core.bytes,
              sha256: core.sha256,
              producer: core.producer,
              producerUnavailable: core.producerUnavailable,
              frames,
              rawCorePublished: false,
            },
            null,
            2,
          ) + "\n",
        );
      }
  }
  const observation = {
    ...result,
    startedAt,
    finishedAt,
    eventCount,
    images,
    failureObserver,
    privateTrace: await fileIdentity(trace),
    privateStdout: await fileIdentity(path.join(scratch, "stdout")),
    privateStderr: await fileIdentity(path.join(scratch, "stderr")),
    sourceDiffAfter: git("diff", "--name-only", "HEAD"),
    classification:
      result.status === 0
        ? "symptom not reproduced; no causal repair claim"
        : "command failed; classify actual process evidence, not automatically the original symptom",
  };
  fs.writeFileSync(
    path.join(output, "observation.json"),
    JSON.stringify(observation, null, 2) + "\n",
  );
  console.log(
    JSON.stringify({ sourceSha, run: process.env.GITHUB_RUN_ID, ...observation }, null, 2),
  );
  process.exitCode = result.status === 0 && result.signal === null ? 0 : 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
