import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../../differential/build-receipt.ts";

type ProcessPacket = {
  command: string;
  cwd: string;
  args: string[];
  status: number | null;
  signal: string | null;
  error: { name: string; message: string } | null;
  stdout: string;
  stderr: string;
};
type Input = { path: string; source: string; sha256: string };
type BeforeCase = {
  id: string;
  role: string;
  inputs: Input[];
  on: ProcessPacket;
  off: ProcessPacket;
  plain: ProcessPacket;
  observationError?: string;
};
type BeforeReport = {
  schema: string;
  qualified: false;
  corpusSha256: string;
  cases: BeforeCase[];
  failures: unknown[];
};
type FoundationCase = {
  id: string;
  inputs: Input[];
  source: Partial<Record<"on" | "off" | "plain", ProcessPacket>>;
  beforeDoctor?: ProcessPacket;
  sourceDoctor?: ProcessPacket;
  after?: { path: string; sha256: string }[];
  matched?: true;
  publicBefore?: BeforeCase;
};

const argv = process.argv.slice(2);
function option(name: string): string {
  const index = argv.indexOf(name);
  const value = index >= 0 ? argv[index + 1] : undefined;
  assert(value, "Missing " + name);
  return value;
}
function hash(value: string | Uint8Array): string {
  return createHash("sha256").update(value).digest("hex");
}
const root = path.resolve(fileURLToPath(new URL("../../../", import.meta.url)));
const beforePath = path.resolve(option("--before"));
const output = path.resolve(option("--output"));
const beforeBytes = fs.readFileSync(beforePath);
const before = JSON.parse(beforeBytes.toString("utf8")) as BeforeReport;
assert.equal(before.schema, "vize.cross-file.nuxt-exclusive-id.public-before.v1");
assert.equal(before.qualified, false);
assert.equal(
  before.corpusSha256,
  "2681f03c914880af56bdd419e326cf8867ab4f2909241272153a51fd6478cb32",
);
assert.equal(before.cases.length, 13);
assert.deepEqual(before.failures, []);

const identity = expectedBuildIdentity(root);
const receipt = JSON.parse(
  fs.readFileSync(path.join(root, identity.binaryPath + ".differential-build.json"), "utf8"),
);
validateBuildReceipt(receipt, identity);
const candidate = path.join(root, identity.binaryPath);
const env: NodeJS.ProcessEnv = { ...process.env, NO_COLOR: "1", VIZE_LOG: "warn" };
delete env.FORCE_COLOR;
const report: {
  schema: string;
  role: string;
  qualified: false;
  publicFixAccepted: false;
  foundationParityPassed: boolean;
  before: { path: string; sha256: string };
  producer: { identity: typeof identity; receipt: unknown };
  cases: FoundationCase[];
  controls: FoundationCase[];
} = {
  schema: "vize.nuxt-render.foundation-parity.v1",
  role: "producer foundation only; no10919exclusion or publicfix qualification",
  qualified: false,
  publicFixAccepted: false,
  foundationParityPassed: false,
  before: { path: beforePath, sha256: hash(beforeBytes) },
  producer: { identity, receipt },
  cases: [],
  controls: [],
};
fs.mkdirSync(path.dirname(output), { recursive: true });
function persist(): void {
  fs.writeFileSync(output, JSON.stringify(report, null, 2) + "\n");
}
function run(command: string, args: string[], cwd: string): ProcessPacket {
  const packet = spawnSync(command, args, {
    cwd,
    env,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  return {
    command,
    cwd,
    args,
    status: packet.status,
    signal: packet.signal,
    error: packet.error ? { name: packet.error.name, message: packet.error.message } : null,
    stdout: packet.stdout ?? "",
    stderr: packet.stderr ?? "",
  };
}
function valid(packet: ProcessPacket): void {
  assert.equal(packet.error, null);
  assert.equal(packet.signal, null);
  assert([0, 1].includes(packet.status ?? -1));
}
function compareCase(original: BeforeCase, controls = false): void {
  assert.equal(original.observationError, undefined, original.id);
  const row: FoundationCase = { id: original.id, inputs: original.inputs, source: {} };
  if (controls) row.publicBefore = original;
  (controls ? report.controls : report.cases).push(row);
  for (const input of original.inputs) {
    assert.equal(hash(input.source), input.sha256);
    assert.equal(hash(fs.readFileSync(path.join(original.on.cwd, input.path))), input.sha256);
  }
  for (const role of ["on", "off", "plain"] as const) {
    const baseline = original[role];
    valid(baseline);
    assert.equal(baseline.cwd, original.on.cwd);
    const observed = run(candidate, baseline.args, baseline.cwd);
    row.source[role] = observed;
    persist();
    valid(observed);
    assert.equal(observed.status, baseline.status, original.id + "/" + role + "/status");
    assert.equal(observed.stdout, baseline.stdout, original.id + "/" + role + "/whole stdout");
    assert.equal(observed.stderr, baseline.stderr, original.id + "/" + role + "/whole stderr");
  }
  const args = ["doctor", "src", "--format", "json", "--exit-zero"];
  row.beforeDoctor = run(original.on.command, args, original.on.cwd);
  row.sourceDoctor = run(candidate, args, original.on.cwd);
  persist();
  valid(row.beforeDoctor);
  valid(row.sourceDoctor);
  assert.equal(row.beforeDoctor.status, 0);
  assert.equal(row.sourceDoctor.status, 0);
  assert.deepEqual(JSON.parse(row.sourceDoctor.stdout), JSON.parse(row.beforeDoctor.stdout));
  assert.equal(row.sourceDoctor.stderr, row.beforeDoctor.stderr);
  row.after = original.inputs.map((input) => ({
    path: input.path,
    sha256: hash(fs.readFileSync(path.join(original.on.cwd, input.path))),
  }));
  assert.deepEqual(
    row.after,
    original.inputs.map(({ path, sha256 }) => ({ path, sha256 })),
  );
  row.matched = true;
  persist();
}
try {
  for (const original of before.cases) compareCase(original);
  const originalCorpusBytes = fs.readFileSync(
    new URL("../../_fixtures/cross-file/nuxt-exclusive-id-10919.json", import.meta.url),
  );
  assert.equal(hash(originalCorpusBytes), before.corpusSha256);
  const originalCorpus = JSON.parse(originalCorpusBytes.toString("utf8")) as {
    originalFiles: Record<string, string>;
  };
  const controls = JSON.parse(
    fs.readFileSync(
      new URL(
        "../../_fixtures/cross-file/nuxt-render-foundation-controls-10919.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as {
    schema: string;
    originalCorpusBlob: string;
    qualified: false;
    cases: {
      id: string;
      role: string;
      replaceFiles: Record<string, string>;
      patterns: string[];
      expectedDuplicateIds: string[];
    }[];
  };
  assert.equal(controls.schema, "vize.nuxt-render.foundation-controls.v1");
  assert.equal(controls.originalCorpusBlob, "73570f897554578f8b21a93bb23c586ab3761514");
  assert.equal(controls.qualified, false);
  assert.equal(controls.cases.length, 3);
  const baseline = before.cases[0].on.command;
  for (const control of controls.cases) {
    const cwd = path.join(path.dirname(output), "nuxt-render-foundation-controls", control.id);
    const inputs: Input[] = [];
    for (const [name, source] of Object.entries({
      ...originalCorpus.originalFiles,
      ...control.replaceFiles,
    })) {
      assert(!path.isAbsolute(name) && !name.split("/").includes(".."));
      const target = path.join(cwd, name);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, source);
      inputs.push({ path: name, source, sha256: hash(source) });
    }
    const flags = [
      "lint",
      "--no-config",
      "--locale",
      "en",
      "--help-level",
      "none",
      "--format",
      "json",
    ];
    const original: BeforeCase = {
      id: control.id,
      role: control.role,
      inputs,
      on: run(baseline, [...flags, "--cross-file", ...control.patterns], cwd),
      off: run(baseline, [...flags, ...control.patterns], cwd),
      plain: run(
        baseline,
        ["lint", "-f", "plain", "--no-config", "--cross-file", ...control.patterns],
        cwd,
      ),
    };
    compareCase(original, true);
    const messages = (
      JSON.parse(original.on.stdout) as { messages?: { message?: string }[] }[]
    ).flatMap((file) => file.messages ?? []);
    for (const id of control.expectedDuplicateIds) {
      assert(
        messages.some((message) => message.message?.includes("Element ID '" + id + "'")),
        control.id + "/" + id,
      );
    }
    const offMessages = (
      JSON.parse(original.off.stdout) as { messages?: { message?: string }[] }[]
    ).flatMap((file) => file.messages ?? []);
    assert(
      !offMessages.some((message) => message.message?.includes("vize:croquis/cf/duplicate-id")),
    );
  }
  report.foundationParityPassed = true;
} finally {
  persist();
}
console.log(
  JSON.stringify({
    role: report.role,
    qualified: false,
    foundationParityPassed: report.foundationParityPassed,
    cases: report.cases.length,
    controls: report.controls.length,
  }),
);
