/** Reconstruct the original completed campaign; this module never executes a product image. */
import assert from "node:assert/strict";
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { crc32, inflateRawSync } from "node:zlib";
import {
  fixture,
  loadSnapshot,
  readJson,
  relocate,
  safeRelative,
  sha256,
  validateOutput,
  validateProducer,
} from "./shipped-cli-nuxt-capsule.ts";
import {
  balancedOrder,
  labels,
  type Receipt,
  type RecordRow,
  type Snapshot,
} from "./shipped-cli-nuxt-protocol.ts";
import { publish } from "./shipped-cli-nuxt-report.ts";

type Campaign = {
  scope: string;
  archiveSha256: string;
  archiveFileCount: number;
  outputSha256: string;
  executionSha: string;
  publicationFailure: string;
  receipt: Receipt;
  receiptSha256: string;
  nodeSha256: string;
  nativeTypeScriptSha256: string;
  run: {
    id: number;
    head_sha: string;
    path: string;
    conclusion: string;
    status: string;
    html_url: string;
  };
  artifact: {
    id: number;
    name: string;
    digest: string;
    workflow_run: { id: number; head_sha: string };
  };
};

/** Read the digest-sealed GitHub artifact's ordinary, single-disk ZIP entries. */
export function readCampaignZip(bytes: Buffer): Map<string, Buffer> {
  const end = bytes.lastIndexOf(Buffer.from([0x50, 0x4b, 0x05, 0x06]));
  assert.ok(end >= 0, "missing ZIP central directory");
  assert.equal(bytes.readUInt16LE(end + 4), 0);
  assert.equal(bytes.readUInt16LE(end + 6), 0);
  assert.equal(end + 22 + bytes.readUInt16LE(end + 20), bytes.length);
  const count = bytes.readUInt16LE(end + 10);
  assert.notEqual(count, 0xffff, "ZIP64 is not a campaign fixture");
  assert.equal(bytes.readUInt16LE(end + 8), count);
  let cursor = bytes.readUInt32LE(end + 16);
  assert.equal(cursor + bytes.readUInt32LE(end + 12), end);
  const files = new Map<string, Buffer>();
  for (let index = 0; index < count; index++) {
    assert.equal(bytes.readUInt32LE(cursor), 0x02014b50);
    const flags = bytes.readUInt16LE(cursor + 8);
    assert.equal(flags & 1, 0, "encrypted ZIP entry");
    const method = bytes.readUInt16LE(cursor + 10);
    assert.ok(method === 0 || method === 8);
    const compressed = bytes.readUInt32LE(cursor + 20);
    const expanded = bytes.readUInt32LE(cursor + 24);
    assert.ok(expanded < 128 * 1024 * 1024);
    const nameLength = bytes.readUInt16LE(cursor + 28);
    const name = bytes.subarray(cursor + 46, cursor + 46 + nameLength).toString("utf8");
    safeRelative(name);
    assert.ok(!files.has(name), `duplicate ZIP entry ${name}`);
    const local = bytes.readUInt32LE(cursor + 42);
    assert.equal(bytes.readUInt32LE(local), 0x04034b50);
    assert.equal(bytes.readUInt16LE(local + 8), method);
    const start = local + 30 + bytes.readUInt16LE(local + 26) + bytes.readUInt16LE(local + 28);
    const encoded = bytes.subarray(start, start + compressed);
    assert.equal(encoded.length, compressed);
    const content =
      method === 8 ? inflateRawSync(encoded, { maxOutputLength: Math.max(expanded, 1) }) : encoded;
    assert.equal(content.length, expanded);
    assert.equal(crc32(content), bytes.readUInt32LE(cursor + 16));
    files.set(name, content);
    cursor += 46 + nameLength + bytes.readUInt16LE(cursor + 30) + bytes.readUInt16LE(cursor + 32);
  }
  assert.equal(cursor, end);
  return files;
}

export function loadRetainedCampaign() {
  const { receipt, snapshot } = loadSnapshot();
  const campaign = readJson(join(fixture, "retained-campaign.json")) as Campaign;
  const bytes = readFileSync(join(fixture, "retained-campaign.zip"));
  assert.equal(sha256(bytes), campaign.archiveSha256, "retained campaign ZIP digest differs");
  assert.equal(campaign.artifact.digest, `sha256:${campaign.archiveSha256}`);
  assert.equal(sha256(JSON.stringify(campaign.receipt, null, 2) + "\n"), campaign.receiptSha256);
  for (const key of [
    "snapshotSha256",
    "snapshotJsonSha256",
    "lockSha256",
    "packageSha256",
  ] as const)
    assert.equal(campaign.receipt[key], receipt[key], `retained workload recipe differs: ${key}`);
  assert.equal(campaign.artifact.workflow_run.id, campaign.run.id);
  assert.equal(campaign.artifact.workflow_run.head_sha, campaign.run.head_sha);
  assert.equal(campaign.run.path, ".github/workflows/03-shipped-cli-nuxt.yml");
  assert.equal(campaign.run.status, "completed");
  assert.equal(campaign.run.conclusion, "failure");
  const entries = readCampaignZip(bytes);
  assert.equal(entries.size, campaign.archiveFileCount);
  return { campaign, snapshot, entries };
}

export function verifyRetainedCampaign(
  entries: ReadonlyMap<string, Buffer>,
  campaign: Campaign,
  snapshot: Snapshot,
) {
  const read = (name: string) =>
    entries.get(name) ?? assert.fail(`missing campaign record ${name}`);
  const json = (name: string) => JSON.parse(read(name).toString("utf8"));
  const receipt = campaign.receipt;
  validateProducer(json("producer-run.json"), json("producer-artifact.json"), receipt);
  const pair = {
    baseSha: receipt.producer.baseSha,
    headSha: receipt.producer.headSha,
    sourceDateEpoch: receipt.producer.sourceDateEpoch,
  };
  assert.deepEqual(json("pair.json"), pair);
  const runtime = json("runtime.json");
  assert.equal(runtime.platform, "linux");
  assert.equal(runtime.arch, "x64");
  assert.equal(runtime.cpus.length, 32);
  assert.equal(runtime.driver.source, campaign.executionSha);
  assert.equal(runtime.driver.runId, String(campaign.run.id));
  assert.equal(runtime.driver.repository, receipt.producer.repository);
  assert.equal(runtime.nodeSha256, campaign.nodeSha256);
  assert.equal(runtime.nativeTypeScriptSha256, campaign.nativeTypeScriptSha256);
  assert.deepEqual(runtime.command, ["vize", "check", "--format", "json"]);
  assert.equal(runtime.project, join(runtime.work, "vize-p1-nuxt-scale-consumer"));
  assert.equal(runtime.dependencies, join(runtime.work, "vize-p1-nuxt-final-consumer"));
  assert.deepEqual(runtime.substitutions, [
    [receipt.originalProject, runtime.project],
    [receipt.originalDependencies, runtime.dependencies],
  ]);
  for (const [name, version] of Object.entries(receipt.runtime)) {
    const key = name === "nativeTypeScript" ? "@typescript/typescript-linux-x64" : name;
    assert.equal(runtime.installed[key], name === "node" ? `v${version}` : version);
  }
  const before = json("inputs-before.json");
  assert.deepEqual(json("inputs-after.json"), before, "before/after program inputs differ");
  assert.deepEqual(Object.keys(before).sort(), Object.keys(snapshot.programInputSha256).sort());
  for (const [file, expected] of Object.entries(snapshot.programInputSha256)) {
    const hash = file.startsWith("node_modules/")
      ? expected
      : sha256(relocate(snapshot.files[file], receipt, runtime.work));
    assert.equal(before[file], hash, `retained program input differs: ${file}`);
  }
  for (const label of labels) {
    const side = label.startsWith("base") ? "base" : "head";
    assert.deepEqual(json(`images/${label}/identity.json`), {
      schema: "vize.release-measure.cli.v1",
      source: side === "base" ? pair.baseSha : pair.headSha,
      profile: "release",
      target: receipt.producer.target,
      ...receipt.producer.cli[side],
    });
    const versions = read(`images/${label}/elf-versions.txt`).toString("utf8");
    const glibc = [...versions.matchAll(/GLIBC_(\d+)\.(\d+)/gu)].map((m) => [
      Number(m[1]),
      Number(m[2]),
    ]);
    assert.ok(
      glibc.length > 0 &&
        glibc.every(([major, minor]) => major < 2 || (major === 2 && minor <= 17)),
    );
  }
  const checkOutput = (name: string) => {
    const bytes = read(`${name}.stdout.json`);
    assert.equal(sha256(bytes), campaign.outputSha256, `${name}: complete output digest differs`);
    validateOutput(JSON.parse(bytes.toString("utf8")), snapshot, receipt, runtime.work);
  };
  for (const [index, label] of labels.entries()) checkOutput(`warmup-${index}-${label}`);
  const records = json("progress.json") as RecordRow[];
  assert.equal(records.length, 32);
  for (const [index, label] of balancedOrder().entries()) {
    const name = `${String(index).padStart(2, "0")}-${label}`;
    checkOutput(name);
    const time = json(`${name}.time.json`);
    for (const key of ["elapsed", "user", "system", "maxRssKiB"])
      assert.ok(Number.isFinite(time[key]) && time[key] >= 0);
    assert.deepEqual(
      records[index],
      { index, label, ...time, outputSha256: campaign.outputSha256 },
      `${name}: raw time/progress differs`,
    );
  }
  return { records, receipt, runtime };
}

export function replay(out: string) {
  const { campaign, snapshot, entries } = loadRetainedCampaign();
  const { records, receipt } = verifyRetainedCampaign(entries, campaign, snapshot);
  mkdirSync(out, { recursive: true });
  for (const [name, bytes] of entries) {
    safeRelative(name);
    const target = join(out, name);
    mkdirSync(join(target, ".."), { recursive: true });
    writeFileSync(target, bytes, { flag: "wx" });
  }
  publish(out, receipt, records, true);
  const note = `\nOriginal campaign: ${campaign.run.html_url}. Its measurements and complete input/output checks finished; publication failed afterward: ${campaign.publicationFailure}\n\nThis report reconstructs that exact retained ZIP (${campaign.archiveSha256}). No product was executed during replay. The old head remains disqualified for performance acceptance; the original primary/twin slowdowns are preserved.\n`;
  appendFileSync(join(out, "summary.md"), note);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, note);
  writeFileSync(
    join(out, "replay.json"),
    JSON.stringify(
      {
        schema: "vize.shipped-cli.nuxt.replay.v1",
        campaign,
        outputCount: 36,
        outputSha256: campaign.outputSha256,
        programInputCount: 1107,
        measurements: 32,
        performanceAcceptance: false,
        productExecution: false,
        inputManifestBeforeSha256: sha256(entries.get("inputs-before.json")!),
        inputManifestAfterSha256: sha256(entries.get("inputs-after.json")!),
        identityRecordSha256: Object.fromEntries(
          labels.map((label) => [label, sha256(entries.get(`images/${label}/identity.json`)!)]),
        ),
        replaySource: process.env.GITHUB_SHA ?? null,
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
}
