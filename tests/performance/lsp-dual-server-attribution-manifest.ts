import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { repoRoot } from "../_helpers/realworld-patch.ts";
import { PROFILE_BUILD_RECIPE, binaryRelativePath } from "../differential/build-receipt.ts";

type JsonRecord = Record<string, unknown>;

function record(value: unknown, label: string): JsonRecord {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), `${label} is required`);
  return value as JsonRecord;
}

function array(value: unknown, label: string): unknown[] {
  assert.ok(Array.isArray(value), `${label} must be an array`);
  return value;
}

function finiteNumber(value: unknown, label: string): number {
  assert.ok(typeof value === "number" && Number.isFinite(value), `${label} must be finite`);
  return value;
}

function sha256File(filePath: string): string {
  return createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

function sha256Text(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function readJson(filePath: string): unknown {
  const bytes = fs.readFileSync(filePath);
  assert.ok(bytes.length > 0, `${filePath} is empty`);
  return JSON.parse(bytes.toString("utf8"));
}

function gitHead(): string {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

function profileRole(
  command: readonly unknown[],
): "vue-language-server" | "typescript-language-server" {
  const joined = command.filter((item): item is string => typeof item === "string").join(" ");
  if (joined.includes("vue-language-server")) return "vue-language-server";
  if (joined.includes("typescript-language-server")) return "typescript-language-server";
  throw new Error(`CPU profile process has no product role: ${joined}`);
}

const configuredRoot = process.env.LSP_DUAL_ATTRIBUTION_DIR;
const configuredBinary = process.env.VIZE_LSP_BIN;
const expectedSource = process.env.EXPECTED_SOURCE ?? "";
assert.ok(configuredRoot && path.isAbsolute(configuredRoot));
assert.ok(configuredBinary && path.isAbsolute(configuredBinary));
const root = path.resolve(configuredRoot);
const binary = path.resolve(configuredBinary);
assert.match(expectedSource, /^[0-9a-f]{40}$/u);
assert.equal(gitHead(), expectedSource);
assert.ok(path.isAbsolute(root) && root !== path.parse(root).root);
assert.ok(path.isAbsolute(binary) && fs.statSync(binary).size > 0);
const expectedBinaryPath = binaryRelativePath("ci-opt");
assert.equal(path.relative(repoRoot, binary), expectedBinaryPath);

const reportPath = path.join(root, "report/report.json");
const report = record(readJson(reportPath), "benchmark report");
assert.equal(report.status, "passed");
assert.equal(report.commit, expectedSource);

const buildReceiptPath = `${binary}.differential-build.json`;
const buildReceipt = record(readJson(buildReceiptPath), "instrumented build receipt");
const binarySha256 = sha256File(binary);
assert.equal(buildReceipt.recipe, PROFILE_BUILD_RECIPE);
assert.equal(buildReceipt.binaryPath, expectedBinaryPath);
assert.equal(buildReceipt.sourceRevision, expectedSource);
assert.equal(buildReceipt.binarySha256, binarySha256);

const sourceToolReceipt = record(report.sourceToolReceipt, "report.sourceToolReceipt");
const sourceToolReceiptSha256 = sourceToolReceipt.sha256;
assert.ok(typeof sourceToolReceiptSha256 === "string");
const sourceToolInputs = { ...sourceToolReceipt };
delete sourceToolInputs.sha256;
assert.equal(sha256Text(JSON.stringify(sourceToolInputs)), sourceToolReceiptSha256);
assert.equal(sourceToolInputs.sourceSha, expectedSource);
const reportVizeBinary = record(sourceToolInputs.vizeBinary, "source tool Vize binary");
assert.equal(reportVizeBinary.configuredBinary, binary);
assert.equal(reportVizeBinary.relativeBinaryPath, expectedBinaryPath);
assert.equal(reportVizeBinary.binarySha256, binarySha256);
assert.equal(reportVizeBinary.receiptSha256, sha256File(buildReceiptPath));
assert.deepEqual(reportVizeBinary.receipt, buildReceipt);

const resolvedTools = record(sourceToolInputs.resolvedTools, "resolved tools");
for (const product of [
  "vueLanguageTools",
  "typescriptLanguageServer",
  "vueTypescriptPlugin",
  "typescript",
]) {
  const tool = record(resolvedTools[product], `resolved tool ${product}`);
  assert.ok(typeof tool.version === "string" && tool.version.length > 0);
  for (const identityKind of ["manifest", "entrypoint"]) {
    const identity = record(tool[identityKind], `${product}.${identityKind}`);
    const identityPath = identity.path;
    assert.ok(typeof identityPath === "string" && path.isAbsolute(identityPath));
    assert.ok(fs.statSync(identityPath).size > 0);
    assert.equal(identity.sha256, sha256File(identityPath));
  }
}

const vizeDirectory = path.join(root, "vize");
const vizeProfiles = fs
  .readdirSync(vizeDirectory)
  .filter((name) => /^session-\d+\.json$/u.test(name))
  .sort();
assert.equal(vizeProfiles.length, 2, "attribution must retain fresh and warm Vize sessions");
const vize = vizeProfiles.map((name) => {
  const filePath = path.join(vizeDirectory, name);
  const profile = record(readJson(filePath), name);
  const schemaVersion = finiteNumber(profile.schema_version, `${name}.schema_version`);
  assert.ok(schemaVersion >= 1);
  const allocation = record(profile.allocation, `${name}.allocation`);
  const calls = finiteNumber(allocation.calls, `${name}.allocation.calls`);
  const requestedBytes = finiteNumber(
    allocation.requested_bytes,
    `${name}.allocation.requested_bytes`,
  );
  const failures = finiteNumber(allocation.failures, `${name}.allocation.failures`);
  assert.ok(calls > 0 && requestedBytes > 0 && failures >= 0);
  const spans = array(profile.spans, `${name}.spans`).map((value) => record(value, "span"));
  assert.ok(spans.length > 0, `${name} omitted function spans`);
  const majorSelfSpans = spans
    .map((span) => {
      const wall = record(span.wall_ns, "span.wall_ns");
      const allocation = record(span.alloc, "span.alloc");
      assert.ok(typeof span.key === "string" && span.key.length > 0);
      return {
        key: span.key,
        count: finiteNumber(span.count, "span.count"),
        selfNs: finiteNumber(wall.self, "span.wall_ns.self"),
        selfAllocationCalls: finiteNumber(allocation.self_calls, "span.alloc.self_calls"),
        selfAllocationBytes: finiteNumber(allocation.self_bytes, "span.alloc.self_bytes"),
      };
    })
    .sort((left, right) => right.selfNs - left.selfNs)
    .slice(0, 10);
  return {
    file: name,
    sha256: sha256File(filePath),
    schemaVersion,
    allocation: { calls, requestedBytes, failures },
    majorSelfSpans,
  };
});

const products = record(report.products, "report.products");
const vueProduct = record(products["vue-language-tools"], "Vue product report");
const vueSamples = [
  ...array(vueProduct.fresh, "Vue fresh samples"),
  ...array(vueProduct.warm, "Vue warm samples"),
];
const processRoles = new Map<number, { role: string; birthTicks: string }>();
for (const sampleValue of vueSamples) {
  const sample = record(sampleValue, "Vue sample");
  const rss = record(sample.rss, "Vue sample RSS");
  const processRoots = array(rss.roots, "Vue RSS roots");
  assert.equal(processRoots.length, 2, "each Vue sample must retain Vue and TypeScript roots");
  const sampleRoles: string[] = [];
  for (const rootValue of processRoots) {
    const processRoot = record(rootValue, "Vue process root");
    const members = array(processRoot.members, "Vue process members");
    assert.equal(members.length, processRoot.processes);
    const rootMembers = members.filter(
      (memberValue) => record(memberValue, "Vue process member").role === "root",
    );
    assert.equal(rootMembers.length, 1, "each process tree must retain exactly one root identity");
    for (const memberValue of rootMembers) {
      const member = record(memberValue, "Vue process member");
      const pid = finiteNumber(member.pid, "process pid");
      assert.equal(pid, processRoot.rootPid);
      const birthTicks = member.birthTicks;
      assert.ok(typeof birthTicks === "string" && /^\d+$/u.test(birthTicks));
      const identity = {
        role: profileRole(array(member.command, "process command")),
        birthTicks,
      };
      sampleRoles.push(identity.role);
      const previous = processRoles.get(pid);
      assert.ok(
        previous == null ||
          (previous.role === identity.role && previous.birthTicks === identity.birthTicks),
        `PID ${pid} was reused across profiled process identities`,
      );
      processRoles.set(pid, identity);
    }
  }
  assert.deepEqual(sampleRoles.sort(), ["typescript-language-server", "vue-language-server"]);
}
assert.equal(
  [...processRoles.values()].filter((identity) => identity.role === "vue-language-server").length,
  2,
);
assert.equal(
  [...processRoles.values()].filter((identity) => identity.role === "typescript-language-server")
    .length,
  2,
);

const vueDirectory = path.join(root, "vue");
const cpuProfiles = fs
  .readdirSync(vueDirectory)
  .filter((name) => name.endsWith(".cpuprofile"))
  .sort();
assert.equal(cpuProfiles.length, 4, "Vue and TypeScript LS must profile fresh and warm roots");
const node = cpuProfiles.map((name) => {
  const match = /^CPU\.\d+\.\d+\.(\d+)\.\d+\.\d+\.cpuprofile$/u.exec(name);
  assert.ok(match, `unrecognized CPU profile name: ${name}`);
  const pid = Number(match[1]);
  const identity = processRoles.get(pid);
  assert.ok(identity, `CPU profile PID ${pid} is absent from process-tree custody`);
  const filePath = path.join(vueDirectory, name);
  const profile = record(readJson(filePath), name);
  const nodes = array(profile.nodes, `${name}.nodes`);
  const cpuSamples = array(profile.samples, `${name}.samples`);
  const timeDeltas = array(profile.timeDeltas, `${name}.timeDeltas`);
  assert.ok(nodes.length > 0 && cpuSamples.length > 0);
  assert.equal(cpuSamples.length, timeDeltas.length);
  return {
    file: name,
    pid,
    role: identity.role,
    birthTicks: identity.birthTicks,
    sha256: sha256File(filePath),
    nodes: nodes.length,
    samples: cpuSamples.length,
  };
});
assert.equal(node.filter((item) => item.role === "vue-language-server").length, 2);
assert.equal(node.filter((item) => item.role === "typescript-language-server").length, 2);

const manifest = {
  schema: "vize.lsp.dual-server.attribution",
  version: 1,
  sourceSha: expectedSource,
  boundary: {
    comparable: ["same-wire latency", "complete process-tree RSS"],
    attributionOnly: ["Vize allocation/function spans", "Vue and TypeScript LS CPU profiles"],
  },
  instrumentedVize: {
    binary,
    binarySha256,
    buildReceipt,
    buildReceiptSha256: sha256File(buildReceiptPath),
    sessions: vize,
  },
  nodeCpuProfiles: node,
  sourceToolReceiptSha256,
  benchmarkReport: { path: reportPath, sha256: sha256File(reportPath) },
};
fs.writeFileSync(path.join(root, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
