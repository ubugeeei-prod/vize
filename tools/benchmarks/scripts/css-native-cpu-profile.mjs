import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  campaign,
  attributionOrder,
  inputsFromOriginal,
  validateArtifact,
  validateReplayImage,
} from "./css-native-attribution.mjs";
import { profileVectors } from "./css-source-map-profile.mjs";

const script = fileURLToPath(import.meta.url);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const digest = (file) => hash(fs.readFileSync(file));
const read = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");

export function samplingPlan() {
  return attributionOrder().flatMap((item) =>
    profileVectors().map(({ id, batch }) => ({ ...item, id, calls: 1000 * batch })),
  );
}
export function validateSamplePacket(packet, config, item) {
  assert.equal(packet.status, "complete");
  assert.deepEqual(packet.plan, item);
  assert.deepEqual(packet.producer, config.images[item.side].producer);
  const row = config.rows.find((row) => row.id === item.id);
  assert.ok(row);
  assert.equal(packet.source, item.operation === "authored" ? row.source : row.processed);
  assert.deepEqual(packet.options, row.options);
  assert.deepEqual(packet.before, row.expected[item.operation]);
  assert.deepEqual(packet.after, row.expected[item.operation]);
  assert.equal(packet.calls, item.calls);
  assert.equal(packet.codeUnits, item.calls * packet.before.code.length);
}
export function sampledEventCount(text) {
  const rows = text.split("\n").filter((line) => line.trim());
  assert.ok(rows.length > 0, "actual perf samples are required");
  for (const row of rows) assert.match(row, /^\s*\d+\.\d+:?\s*$/u);
  return rows.length;
}
function worker(configFile, ordinal, destination) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(process.version, campaign.node);
  const config = read(configFile),
    item = samplingPlan()[Number(ordinal)];
  assert.ok(item);
  const image = config.images[item.side],
    native = createRequire(import.meta.url)(image.transportBinary);
  validateReplayImage(
    item.side,
    image.producer,
    digest(image.transportBinary),
    native.getNativeBuildToken(),
  );
  const row = config.rows.find((row) => row.id === item.id),
    source = item.operation === "authored" ? row.source : row.processed;
  const packet = {
    schema: "vize.css.native-cpu-sample-workload",
    version: 1,
    status: "attempt",
    harnessSource: config.harnessSource,
    plan: item,
    producer: image.producer,
    source,
    options: row.options,
    calls: 0,
    codeUnits: 0,
  };
  const retain = () => write(destination, packet);
  retain();
  packet.before = native.compileCss(source, row.options);
  retain();
  assert.deepEqual(packet.before, row.expected[item.operation]);
  for (let warm = 0; warm < 100; warm++) native.compileCss(source, row.options);
  for (let call = 0; call < item.calls; call++) {
    packet.codeUnits += native.compileCss(source, row.options).code.length;
    packet.calls++;
  }
  packet.after = native.compileCss(source, row.options);
  packet.status = "complete";
  retain();
  validateSamplePacket(packet, config, item);
}

function command(argv, root, name, record) {
  const output = path.join(root, `${name}.stdout`),
    error = path.join(root, `${name}.stderr`),
    descriptors = [fs.openSync(output, "w"), fs.openSync(error, "w")];
  let result;
  const attempt = { argv, startedAt: new Date().toISOString() };
  record.commands.push(attempt);
  write(path.join(root, "receipt.json"), record);
  try {
    result = spawnSync("timeout", ["--signal=TERM", "--kill-after=5s", "45s", ...argv], {
      stdio: ["ignore", ...descriptors],
      shell: false,
    });
  } finally {
    for (const fd of descriptors) fs.closeSync(fd);
  }
  Object.assign(attempt, {
    endedAt: new Date().toISOString(),
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdoutSha256: digest(output),
    stderrSha256: digest(error),
  });
  write(path.join(root, "receipt.json"), record);
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, `actual command failed; retained ${error}`);
  return fs.readFileSync(output, "utf8");
}
async function run(destination) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.version, campaign.node);
  destination = path.resolve(destination);
  assert.equal(fs.existsSync(destination), false);
  fs.mkdirSync(destination, { recursive: true });
  const harnessSource = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  assert.equal(harnessSource, process.env.GITHUB_SHA);
  const record = {
    schema: "vize.css.native-cpu-sampling",
    version: 1,
    status: "attempt",
    harnessSource,
    campaign,
    plan: samplingPlan(),
    commands: [],
    controls: [],
    samples: [],
    environment: { node: process.version, kernel: os.release(), cpu: os.cpus()[0].model },
    note: "Existing-image sampling only; no new product build or timing/speed authority. Source-line attribution requires actual DWARF, never symbol-name guesses.",
  };
  const retain = () => write(path.join(destination, "receipt.json"), record);
  retain();
  try {
    const headers = {
      Authorization: `Bearer ${process.env.GH_TOKEN}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
    };
    assert.ok(process.env.GH_TOKEN);
    const endpoint = `https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/${campaign.artifact}`;
    const metadata = await fetch(endpoint, { headers, signal: AbortSignal.timeout(30000) });
    assert.equal(metadata.status, 200);
    const meta = await metadata.json();
    write(path.join(destination, "original-artifact-metadata.json"), meta);
    validateArtifact(meta);
    const redirect = await fetch(endpoint + "/zip", {
      headers,
      redirect: "manual",
      signal: AbortSignal.timeout(30000),
    });
    assert.equal(redirect.status, 302);
    const location = redirect.headers.get("location");
    assert.ok(location?.startsWith("https://"));
    const download = await fetch(location, { signal: AbortSignal.timeout(120000) });
    assert.equal(download.status, 200);
    const archive = Buffer.from(await download.arrayBuffer());
    assert.equal(archive.length, campaign.bytes);
    assert.equal(hash(archive), campaign.digest);
    const temporary = fs.mkdtempSync(path.join(process.env.RUNNER_TEMP, "css-native-cpu-")),
      zip = path.join(temporary, "original.zip"),
      origin = path.join(temporary, "original");
    fs.writeFileSync(zip, archive);
    execFileSync("unzip", ["-q", zip, "-d", origin]);
    const rows = inputsFromOriginal(origin),
      original = read(path.join(origin, "receipt.json")),
      images = {};
    for (const side of ["base", "head"]) {
      const producer = original.builds.find((image) => image.side === side),
        transportBinary = path.join(origin, `${side}-image`, path.basename(producer.binary));
      const native = createRequire(import.meta.url)(transportBinary);
      validateReplayImage(side, producer, digest(transportBinary), native.getNativeBuildToken());
      images[side] = { producer, transportBinary };
      for (const row of rows[side]) {
        for (const operation of ["authored", "processed"]) {
          const source = operation === "authored" ? row.source : row.processed;
          const result = native.compileCss(source, row.options);
          record.controls.push({
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
      const sections = command(
        ["readelf", "--sections", "--wide", transportBinary],
        destination,
        `${side}-elf-sections`,
        record,
      );
      command(["readelf", "--notes", transportBinary], destination, `${side}-elf-notes`, record);
      assert.match(sections, /\.symtab\s/u);
      assert.match(sections, /\.eh_frame\s/u);
      record[`${side}DebugInfo`] =
        /\.debug_info\s/u.test(sections) && /\.debug_line\s/u.test(sections);
    }
    const config = { harnessSource, rows: rows.base, images },
      configFile = path.join(destination, "inputs.json");
    write(configFile, config);
    record.inputsSha256 = digest(configFile);
    record.archiveVerified = true;
    record.originalReceiptSha256 = digest(path.join(origin, "receipt.json"));
    const perf = process.env.VIZE_CSS_PERF;
    assert.ok(perf && path.isAbsolute(perf));
    record.perf = {
      file: perf,
      sha256: digest(perf),
      version: command([perf, "version", "--build-options"], destination, "perf-version", record),
    };
    record.perfEventParanoid = fs
      .readFileSync("/proc/sys/kernel/perf_event_paranoid", "utf8")
      .trim();
    record.node = { file: process.execPath, sha256: digest(process.execPath) };
    retain();
    for (const [ordinal, item] of samplingPlan().entries()) {
      assert.equal(digest(configFile), record.inputsSha256);
      const name = `${ordinal}-${item.side}-${item.operation}-${item.id}`,
        output = path.join(destination, `${name}.json`),
        data = path.join(destination, `${name}.perf.data`);
      assert.equal(fs.existsSync(data), false);
      command(
        [
          perf,
          "record",
          "--event",
          "cpu-clock:u",
          "--freq",
          "99",
          "--call-graph",
          "dwarf,8192",
          "--output",
          data,
          "--",
          process.execPath,
          script,
          "--worker",
          configFile,
          String(ordinal),
          output,
        ],
        destination,
        `${name}-record`,
        record,
      );
      const packet = read(output);
      validateSamplePacket(packet, config, item);
      command(
        [
          perf,
          "report",
          "--stdio",
          "--no-children",
          "--show-nr-samples",
          "--sort",
          "dso,symbol,srcline",
          "--percent-limit",
          "0",
          "--input",
          data,
        ],
        destination,
        `${name}-flat`,
        record,
      );
      command(
        [
          perf,
          "report",
          "--stdio",
          "--children",
          "--show-nr-samples",
          "--sort",
          "dso,symbol",
          "--percent-limit",
          "0.5",
          "--input",
          data,
        ],
        destination,
        `${name}-children`,
        record,
      );
      command([perf, "script", "--input", data], destination, `${name}-stacks`, record);
      const events = command(
        [perf, "script", "--hide-call-graph", "--fields", "time", "--input", data],
        destination,
        `${name}-events`,
        record,
      );
      command([perf, "buildid-list", "--input", data], destination, `${name}-buildids`, record);
      const sampleCount = sampledEventCount(events);
      record.samples.push({
        ...item,
        sampleCount,
        dataSha256: digest(data),
        packetSha256: digest(output),
      });
      retain();
      console.log(JSON.stringify({ stage: "sample-complete", ordinal, ...item, sampleCount }));
    }
    assert.equal(record.samples.length, 24);
    assert.equal(record.controls.length, 12);
    record.status =
      record.baseDebugInfo && record.headDebugInfo
        ? "source-samples-complete"
        : "symbol-samples-complete-no-source-lines";
    retain();
  } catch (error) {
    record.status = "failed";
    record.failure = { name: error.name, message: error.message };
    retain();
    throw error;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === script) {
  if (process.argv[2] === "--worker") worker(...process.argv.slice(3));
  else await run(process.argv[2]);
}
