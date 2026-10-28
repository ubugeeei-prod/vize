/** Re-read an immutable completed campaign; this never runs a compiler or installs packages. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { hash, save } from "./canon-instance-directives-io.ts";
import { isRecord, packet } from "./canon-instance-directives-model.ts";
import { diagnostics } from "./canon-slot-silent-any-semantic.ts";
import { silentAnyCases } from "./canon-slot-silent-any-cases.ts";
type ObjectValue = Record<string, unknown>;
function object(value: unknown): ObjectValue {
  assert.ok(isRecord(value));
  return value;
}
function string(value: unknown): string {
  assert.ok(typeof value === "string");
  return value;
}
function numeric(value: unknown): number {
  assert.ok(typeof value === "number" && Number.isFinite(value));
  return value;
}
function array(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
function read(file: string): unknown {
  const value: unknown = JSON.parse(fs.readFileSync(file, "utf8"));
  return value;
}
function byteAt(text: string, line: number, column: number): number {
  const lines = text.split("\n"),
    tail = lines[line - 1];
  assert.ok(tail !== undefined);
  return Buffer.byteLength(
    lines.slice(0, line - 1).join("\n") + (line > 1 ? "\n" : "") + tail.slice(0, column - 1),
  );
}
function point(text: string, byte: number): { line: number; column: number } {
  const before = new TextDecoder("utf-8", { fatal: true }).decode(
    Buffer.from(text).subarray(0, byte),
  );
  const lines = before.split("\n"),
    tail = lines.at(-1);
  assert.ok(tail !== undefined);
  return { line: lines.length, column: tail.length + 1 };
}
function translatedByte(original: string, variant: string, generatedByte: number): number | null {
  if (variant === original) return generatedByte;
  let prefix = 0;
  const before = Buffer.from(original),
    after = Buffer.from(variant);
  while (prefix < before.length && prefix < after.length && before[prefix] === after[prefix])
    prefix++;
  let suffix = 0;
  while (
    suffix < before.length - prefix &&
    suffix < after.length - prefix &&
    before[before.length - 1 - suffix] === after[after.length - 1 - suffix]
  )
    suffix++;
  const newEnd = after.length - suffix,
    oldEnd = before.length - suffix;
  if (generatedByte < prefix) return generatedByte;
  if (generatedByte >= newEnd) return generatedByte - newEnd + oldEnd;
  return null;
}
const [rootArg, output] = process.argv.slice(2);
assert.ok(rootArg && output && process.argv.length === 4);
const root = path.resolve(rootArg),
  project = path.join(root, "project"),
  rows = silentAnyCases();
const artifacts = {
  originalRun: 40612314261,
  originalSource: "921bd721d813268c1f4f2ae906b0c294b32896d3",
  rawArtifact: 12783338444,
  rawBytes: 321984,
  rawSha256: "7cb2f00519d51425f844430065a06ccb3167bd487cbae879229669505d893028",
};
const calls: ObjectValue[] = [];
const labels = [
  "official-vue-tsc",
  ...["original", "unknown-fallback", "plain-any-calibration"].flatMap((variant) =>
    ["6.0.3", "7.0.2", "7.1.0-beta"].map((version) => variant + "-" + version),
  ),
  "public-cli-7.0.2",
  "public-mapper-7.0.2",
];
for (const label of labels) {
  const process = object(read(path.join(root, label + ".process.json")));
  const stdout = fs.readFileSync(path.join(root, label + ".stdout.bin")),
    stderr = fs.readFileSync(path.join(root, label + ".stderr.bin"));
  assert.equal(hash(stdout), process.stdoutSha256);
  assert.equal(hash(stderr), process.stderrSha256);
  assert.equal(process.signal, null);
  assert.equal(process.error, null);
  const status = numeric(process.status),
    cwd = string(process.cwd);
  let parsed: unknown = null,
    parseFailure: string | null = null;
  try {
    parsed = label.startsWith("public-cli")
      ? packet(stdout, status, cwd).diagnostics
      : diagnostics(stdout, status, cwd);
  } catch (error) {
    parseFailure = error instanceof Error ? error.message : String(error);
  }
  calls.push({
    label,
    process,
    parsed,
    parseFailure,
    stdoutBytes: stdout.length,
    stderrBytes: stderr.length,
    stdoutSha256: hash(stdout),
    stderrSha256: hash(stderr),
    stdout: parseFailure === null ? null : stdout.toString("utf8"),
    stderr: stderr.toString("utf8"),
  });
}
const byCase = rows.map((row) => {
  const rowRoot = path.join(project, row.id),
    authored = path.join(rowRoot, "src/Probe.vue"),
    component = path.join(rowRoot, "component.ts");
  assert.equal(fs.readFileSync(authored, "utf8"), row.source);
  assert.equal(fs.readFileSync(component, "utf8"), row.component);
  const responses = array(read(path.join(root, "rpc-" + row.id, "responses.json"))).map(object);
  assert.deepEqual(
    responses.map((response) => response.id),
    [1, 2, 3, 4],
  );
  const original = string(object(responses[2]?.result).text);
  const stored = fs.readFileSync(path.join(rowRoot, "src/original.tsx"), "utf8");
  assert.equal(stored, original);
  const mappings = array(object(responses[2]?.result).mappings).map((span) =>
    array(span).map(numeric),
  );
  const variants = ["original", "unknown-fallback", "plain-any-calibration"].map((variant) => {
    const text = fs.readFileSync(path.join(rowRoot, "src", variant + ".tsx"), "utf8");
    return {
      variant,
      bytes: Buffer.byteLength(text),
      sha256: hash(text),
      results: calls
        .filter((call) => string(call.label).startsWith(variant + "-"))
        .map((call) => {
          assert.equal(call.parseFailure, null);
          const errors = array(call.parsed)
            .map(object)
            .filter((d) => string(d.file).startsWith(row.id + "/"));
          return {
            compiler: string(call.label).slice(variant.length + 1),
            diagnostics: errors.map((diagnostic) => {
              const generatedByte = byteAt(
                text,
                numeric(diagnostic.line),
                numeric(diagnostic.column),
              );
              const oldByte = translatedByte(original, text, generatedByte);
              const owners =
                oldByte === null
                  ? []
                  : mappings
                      .filter((span) => {
                        const [g, length, , , kind] = span;
                        assert.ok(g !== undefined && length !== undefined);
                        return kind === 0 && oldByte >= g && oldByte < g + length;
                      })
                      .map((span) => {
                        const [g, , authored, authoredLength] = span;
                        assert.ok(
                          g !== undefined &&
                            authored !== undefined &&
                            authoredLength !== undefined &&
                            oldByte !== null,
                        );
                        const byte = authored + oldByte - g;
                        return byte < authored + authoredLength
                          ? { byte, ...point(row.source, byte) }
                          : null;
                      })
                      .filter((owner) => owner !== null);
              return {
                ...diagnostic,
                generatedByte,
                originalGeneratedByte: oldByte,
                authoredOwners: owners,
              };
            }),
          };
        }),
    };
  });
  return {
    id: row.id,
    family: row.family,
    stage: row.stage,
    source: row.source,
    component: row.component,
    sourceSha256: hash(row.source),
    componentSha256: hash(row.component),
    variants,
    official: array(calls.find((call) => call.label === "official-vue-tsc")?.parsed)
      .map(object)
      .filter((d) => string(d.file).startsWith(row.id + "/")),
    publicCli: array(calls.find((call) => call.label === "public-cli-7.0.2")?.parsed)
      .map(object)
      .filter((d) => string(d.file).startsWith(row.id + "/")),
  };
});
const api = ["7.0.2", "7.1.0-beta"].map((version) => {
  const folder = path.join(root, "api-" + version),
    receipt = object(read(path.join(folder, "receipt.json")));
  const raw = ["stdin", "stdout", "stderr"].map((stream) => {
    const bytes = fs.readFileSync(path.join(folder, stream + ".raw"));
    return { stream, bytes: bytes.length, sha256: hash(bytes) };
  });
  assert.equal(receipt.status, 0);
  assert.equal(receipt.signal, null);
  assert.equal(receipt.failure, null);
  assert.equal(receipt.trailingBytes, 0);
  assert.deepEqual(receipt.image, receipt.after);
  assert.equal(array(receipt.requests).length, 25);
  assert.equal(array(receipt.responses).length, 25);
  return { ...receipt, raw };
});
save(output, {
  schema: "vize.slotSilentAnyRecoveredCampaign.v1",
  artifacts,
  api,
  stock: read(path.join(root, "stock-calibration.json")),
  byCase,
  calls,
  providerLock: {
    bytes: fs.statSync(path.join(root, "package-lock.json")).size,
    sha256: hash(fs.readFileSync(path.join(root, "package-lock.json"))),
  },
  unexecutedRoutes: ["public-cli-7.1.0-beta", "public-mapper-7.1.0-beta"],
  timingUsable: false,
  sourceRepairQualified: false,
  originalCampaignStatus:
    "failed global-output classifier; immutable raw source and output retained",
});
console.log(
  JSON.stringify({
    output,
    cases: rows.length,
    calls: calls.length,
    failures: calls
      .filter((call) => call.parseFailure !== null)
      .map((call) => ({
        label: call.label,
        failure: call.parseFailure,
        stdout: call.stdout,
        stderr: call.stderr,
      })),
  }),
);
