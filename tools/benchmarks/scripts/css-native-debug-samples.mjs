import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import {
  sampledEventCount,
  samplingPlan,
  validateSamplePacket,
} from "./css-native-cpu-profile.mjs";
import { streamedDigest as digest, validateDebugIdentity } from "./css-native-debug-build.mjs";

const script = fileURLToPath(import.meta.url);
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const read = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
async function run(configFile, destination) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.platform, "linux");
  assert.equal(process.version, "v24.14.0");
  destination = path.resolve(destination);
  assert.equal(fs.existsSync(destination), false);
  fs.mkdirSync(destination, { recursive: true });
  const config = read(configFile);
  assert.equal(config.kind, "debug-source");
  const harnessSource = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  assert.equal(harnessSource, process.env.GITHUB_SHA);
  assert.equal(config.harnessSource, harnessSource);
  const receipt = {
    schema: "vize.css.native-debug-cpu-samples",
    version: 1,
    status: "attempt",
    harnessSource,
    plan: samplingPlan(),
    configFile,
    configSha256: digest(configFile),
    images: config.images,
    commands: [],
    controls: [],
    samples: [],
    analyses: [],
    environment: { node: process.version, kernel: os.release(), cpu: os.cpus()[0].model },
    note: "Separate optimized debug-source profiling. Only exclusive leaf samples describe self cost; inclusive source/inline allocation paths overlap and are never summed. No release-image speed or allocation-count authority.",
  };
  const retain = () => write(path.join(destination, "receipt.json"), receipt);
  retain();
  const command = async (argv, name, seconds = 180) => {
    const output = path.join(destination, `${name}.stdout`),
      error = path.join(destination, `${name}.stderr`);
    const fds = [fs.openSync(output, "w"), fs.openSync(error, "w")];
    const row = { name, argv, startedAt: new Date().toISOString() };
    receipt.commands.push(row);
    retain();
    console.log(JSON.stringify({ stage: "command-start", name }));
    let result;
    try {
      result = await new Promise((resolve, reject) => {
        const child = spawn(
          "timeout",
          ["--signal=TERM", "--kill-after=5s", `${seconds}s`, ...argv],
          { stdio: ["ignore", ...fds], shell: false },
        );
        child.once("error", reject);
        child.once("close", (status, signal) => resolve({ status, signal }));
      });
    } finally {
      for (const fd of fds) fs.closeSync(fd);
    }
    Object.assign(row, result, {
      endedAt: new Date().toISOString(),
      stdoutSha256: digest(output),
      stderrSha256: digest(error),
    });
    retain();
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, error);
    console.log(JSON.stringify({ stage: "command-complete", name }));
    return fs.readFileSync(output, "utf8");
  };
  try {
    const perf = process.env.VIZE_CSS_PERF;
    assert.ok(perf && path.isAbsolute(perf));
    receipt.perf = {
      file: perf,
      sha256: digest(perf),
      version: await command([perf, "version", "--build-options"], "perf-version"),
    };
    receipt.perfEventParanoid = fs
      .readFileSync("/proc/sys/kernel/perf_event_paranoid", "utf8")
      .trim();
    for (const [side, image] of Object.entries(config.images)) {
      const native = createRequire(import.meta.url)(image.transportBinary);
      validateDebugIdentity(
        side,
        image.producer,
        digest(image.transportBinary),
        native.getNativeBuildToken(),
      );
      const sections = await command(
        ["readelf", "--sections", "--wide", image.transportBinary],
        `${side}-elf-sections`,
      );
      assert.match(sections, /\.debug_info\s/u);
      assert.match(sections, /\.debug_line\s/u);
      await command(["readelf", "--notes", image.transportBinary], `${side}-elf-notes`);
      for (const row of config.rows)
        for (const operation of ["authored", "processed"]) {
          const source = operation === "authored" ? row.source : row.processed;
          const result = native.compileCss(source, row.options);
          receipt.controls.push({
            side,
            id: row.id,
            operation,
            source,
            options: row.options,
            result,
          });
          retain();
          assert.deepEqual(result, row.expected[operation]);
        }
    }
    const worker = path.join(path.dirname(script), "css-native-cpu-profile.mjs");
    const fifoRoot = fs.mkdtempSync(path.join(process.env.RUNNER_TEMP, "css-perf-control-"));
    for (const [ordinal, item] of samplingPlan().entries()) {
      assert.equal(digest(configFile), receipt.configSha256);
      const name = `${ordinal}-${item.side}-${item.operation}-${item.id}`,
        data = path.join(destination, `${name}.perf.data`),
        packetFile = path.join(destination, `${name}.json`);
      const control = path.join(fifoRoot, `${ordinal}-control`),
        ack = path.join(fifoRoot, `${ordinal}-ack`);
      execFileSync("mkfifo", [control, ack]);
      assert.equal(fs.existsSync(data), false);
      await command(
        [
          perf,
          "record",
          "--event",
          "cpu-clock:u",
          "--freq",
          "99",
          "--call-graph",
          "dwarf,8192",
          "--delay=-1",
          `--control=fifo:${control},${ack}`,
          "--output",
          data,
          "--",
          process.execPath,
          worker,
          "--worker",
          configFile,
          String(ordinal),
          packetFile,
          control,
          ack,
        ],
        `${name}-record`,
        60,
      );
      validateSamplePacket(read(packetFile), config, item);
      const events = await command(
        [perf, "script", "--hide-call-graph", "--fields", "time", "--input", data],
        `${name}-events`,
      );
      receipt.samples.push({
        ...item,
        sampleCount: sampledEventCount(events),
        dataSha256: digest(data),
        packetSha256: digest(packetFile),
      });
      retain();
      fs.unlinkSync(control);
      fs.unlinkSync(ack);
    }
    assert.equal(receipt.samples.length, 24);
    receipt.captureComplete = true;
    retain();
    let next = 0;
    // Decode independently after complete raw capture. Four bounded workers
    // avoid repeating the previous serial 676-second symbol-report phase.
    const analyse = async () => {
      for (;;) {
        const ordinal = next++;
        if (ordinal >= receipt.plan.length) return;
        const item = receipt.plan[ordinal],
          name = `${ordinal}-${item.side}-${item.operation}-${item.id}`,
          data = path.join(destination, `${name}.perf.data`);
        const flat = await command(
          [
            perf,
            "report",
            "--stdio",
            "--no-children",
            "--no-inline",
            "--call-graph",
            "none",
            "--full-source-path",
            "--show-nr-samples",
            "--sort",
            "dso,symbol,srcline",
            "--percent-limit",
            "0",
            "--input",
            data,
          ],
          `${name}-flat`,
        );
        assert.match(
          flat,
          /\.rs:[1-9][0-9]*/u,
          "real source lines required, not only ELF section presence",
        );
        await command(
          [
            perf,
            "report",
            "--stdio",
            "--children",
            "--inline",
            "--call-graph",
            "graph,0.5,caller,srcline",
            "--full-source-path",
            "--show-nr-samples",
            "--sort",
            "dso,symbol,srcline",
            "--percent-limit",
            "0.5",
            "--input",
            data,
          ],
          `${name}-source-inline-callers`,
        );
        await command(
          [perf, "script", "--fields", "+srcline", "--input", data],
          `${name}-source-stacks`,
        );
        await command([perf, "buildid-list", "--input", data], `${name}-buildids`);
        receipt.analyses.push({ ordinal, ...item });
        retain();
      }
    };
    const results = await Promise.allSettled(Array.from({ length: 4 }, () => analyse()));
    for (const result of results) if (result.status === "rejected") throw result.reason;
    assert.equal(receipt.analyses.length, 24);
    assert.equal(receipt.controls.length, 12);
    receipt.status = "complete";
    retain();
  } catch (error) {
    receipt.status = "failed";
    receipt.failure = { name: error.name, message: error.message };
    retain();
    throw error;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === script)
  await run(...process.argv.slice(2));
