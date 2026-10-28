/** Compare full fresh-generation packets; normal CLI timing and allocation probes stay separate. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { TextDecoder } from "node:util";
import { scopeBindingCases } from "./component-scope-binding-cases.ts";
type ObjectValue = Record<string, unknown>;
function isRecord(value: unknown): value is ObjectValue {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function record(value: unknown): ObjectValue {
  assert.ok(isRecord(value));
  return value;
}
function read(file: string): ObjectValue {
  const value: unknown = JSON.parse(fs.readFileSync(file, "utf8"));
  return record(value);
}
function string(value: unknown): string {
  assert.ok(typeof value === "string");
  return value;
}
function number(value: unknown): number {
  assert.ok(typeof value === "number" && Number.isFinite(value));
  return value;
}
function array(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
const sha = (value: Buffer | string): string => createHash("sha256").update(value).digest("hex");
function save(file: string, value: unknown): void {
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
}
function packet(bytes: Buffer): ObjectValue[] {
  const packets: ObjectValue[] = [];
  let offset = 0;
  while (offset < bytes.length) {
    const end = bytes.indexOf("\r\n\r\n", offset);
    assert.ok(end >= offset);
    const header = /^Content-Length: (\d+)$/iu.exec(bytes.subarray(offset, end).toString("ascii"));
    assert.ok(header);
    const length = Number(string(header[1])),
      start = end + 4;
    assert.ok(Number.isSafeInteger(length) && length >= 0 && start + length <= bytes.length);
    const value: unknown = JSON.parse(
      new TextDecoder("utf-8", { fatal: true }).decode(bytes.subarray(start, start + length)),
    );
    const response = record(value);
    assert.equal(response.jsonrpc, "2.0");
    assert.ok(!Object.hasOwn(response, "error") && Object.hasOwn(response, "result"));
    packets.push(response);
    offset = start + length;
  }
  assert.equal(offset, bytes.length);
  assert.deepEqual(
    packets.map((row) => row.id),
    [1, 2, 3, 4],
  );
  return packets;
}
function median(values: number[]): number {
  const ordered = [...values].sort((a, b) => a - b),
    result = ordered[Math.floor(ordered.length / 2)];
  assert.ok(result !== undefined);
  return result;
}
function span(value: ObjectValue, name: string): ObjectValue {
  const result = array(value.spans)
    .map(record)
    .find((row) => row.name === name);
  assert.ok(result, name + " must execute");
  return result;
}
export function main(args: string[]): void {
  const [root] = args;
  assert.ok(root && args.length === 1);
  const provenance = read(path.join(root, "provenance.json"));
  const binaries = read(path.join(root, "binaries.json"));
  const baseline = record(binaries.base),
    head = record(binaries.head);
  assert.notEqual(baseline.sha256, head.sha256);
  assert.equal(baseline.source, provenance.productBase);
  assert.equal(head.source, provenance.productHead);
  assert.deepEqual(record(baseline.cargoArtifact).features, record(head.cargoArtifact).features);
  const profileBinaries = ["base", "head"].map((role) =>
    read(path.join(root, role + "-profile-binary.json")),
  );
  const [profileBase, profileHead] = profileBinaries;
  assert.ok(profileBase && profileHead);
  assert.notEqual(profileBase.sha256, profileHead.sha256);
  assert.equal(
    new Set([
      baseline.targetDirectory,
      head.targetDirectory,
      ...profileBinaries.map((row) => row.targetDirectory),
    ]).size,
    4,
  );
  const overlays: unknown = JSON.parse(
    fs.readFileSync(path.join(root, "profiling-overlays.json"), "utf8"),
  );
  assert.equal(array(overlays).length, 4);
  for (const role of ["base", "head"]) {
    const binary = record(binaries[role]);
    assert.equal(sha(fs.readFileSync(string(binary.file))), binary.sha256);
    assert.equal(binary.fresh, false);
  }
  const profiles: ObjectValue[] = [];
  for (const { shape, count, scriptBindings } of scopeBindingCases) {
    const runs: ObjectValue[] = [];
    for (const mode of ["runtime-off", "allocation"])
      for (const round of [1, 2, 3]) {
        const base = read(
          path.join(root, `base-${shape}-${count}-${scriptBindings}-${mode}-${round}.json`),
        );
        const changed = read(
          path.join(root, `head-${shape}-${count}-${scriptBindings}-${mode}-${round}.json`),
        );
        for (const row of [base, changed]) {
          assert.equal(row.schema, "vize.component-scope-binding-profile.v1");
          assert.equal(row.shape, shape);
          assert.equal(row.count, count);
          assert.equal(row.scriptBindings, scriptBindings);
          assert.equal(row.mode, mode);
          assert.deepEqual(row.cold, { generationCalls: 1, warmups: 0, persistentCache: false });
        }
        assert.equal(base.source, changed.source);
        assert.equal(base.file, changed.file);
        assert.deepEqual(
          base.packet,
          changed.packet,
          "the entire generation packet must remain identical",
        );
        const baseAlloc = record(base.allocation),
          headAlloc = record(changed.allocation);
        if (mode === "allocation") {
          const before = span(base, "canon.component_binding_observer.with_bindings");
          const after = span(changed, "canon.component_binding_observer.with_bindings");
          const namesBefore = span(
            base,
            "canon.component_binding_observer.used_component_name_list",
          );
          const namesAfter = span(
            changed,
            "canon.component_binding_observer.used_component_name_list",
          );
          assert.ok(number(after.calls) <= number(before.calls));
          assert.ok(number(namesAfter.calls) <= number(namesBefore.calls));
          assert.ok(number(headAlloc.calls) <= number(baseAlloc.calls));
          assert.ok(number(headAlloc.requestedBytes) <= number(baseAlloc.requestedBytes));
          if (shape === "camel" || shape === "missing") {
            assert.ok(number(after.calls) < number(before.calls));
            assert.ok(number(headAlloc.requestedBytes) < number(baseAlloc.requestedBytes));
          }
          assert.equal(number(namesAfter.calls), number(namesBefore.calls));
        } else {
          assert.equal(baseAlloc.calls, 0);
          assert.equal(headAlloc.calls, 0);
          assert.deepEqual(base.spans, []);
          assert.deepEqual(changed.spans, []);
        }
        runs.push({
          mode,
          round,
          base: base.allocation,
          head: changed.allocation,
          sourceFunction:
            mode === "allocation"
              ? {
                  base: span(base, "canon.component_binding_observer.with_bindings"),
                  head: span(changed, "canon.component_binding_observer.with_bindings"),
                }
              : null,
        });
      }
    const input = read(
      path.join(root, `base-${shape}-${count}-${scriptBindings}-allocation-1.json`),
    );
    profiles.push({ shape, count, scriptBindings, sourceSha256: sha(string(input.source)), runs });
  }
  const normal: ObjectValue[] = [];
  const project = path.join(root, "normal-project");
  fs.mkdirSync(project);
  const config = path.join(project, "tsconfig.json");
  const compilerOptions = {
    strict: true,
    noEmit: true,
    target: "ESNext",
    module: "NodeNext",
    moduleResolution: "NodeNext",
    jsx: "preserve",
  };
  save(config, { compilerOptions, files: ["Bindings.vue"] });
  for (const { shape, count, scriptBindings } of scopeBindingCases) {
    const fixture = read(
      path.join(root, `base-${shape}-${count}-${scriptBindings}-allocation-1.json`),
    );
    const source = string(fixture.source),
      fileName = path.join(project, "Bindings.vue");
    fs.writeFileSync(fileName, source);
    const requests = [
      {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: { positionEncodings: ["utf-8"], locale: "en-US" },
      },
      {
        jsonrpc: "2.0",
        id: 2,
        method: "openProject",
        params: {
          configFileName: config,
          projectHandle: "binding-view",
          options: null,
          compilerOptions,
        },
      },
      {
        jsonrpc: "2.0",
        id: 3,
        method: "transform",
        params: { fileName, content: source, projectHandle: "binding-view" },
      },
      {
        jsonrpc: "2.0",
        id: 4,
        method: "closeProject",
        params: { projectHandle: "binding-view" },
      },
    ];
    const input = Buffer.concat(
      requests.map((request) => {
        const body = Buffer.from(JSON.stringify(request));
        return Buffer.concat([Buffer.from("Content-Length: " + body.length + "\r\n\r\n"), body]);
      }),
    );
    const rows: ObjectValue[] = [];
    for (const round of [1, 2, 3]) {
      const packets: Record<string, ObjectValue[]> = {};
      for (const role of round === 2 ? ["head", "base"] : ["base", "head"]) {
        const binary = record(binaries[role]),
          prefix = path.join(root, `normal-${role}-${shape}-${count}-${scriptBindings}-${round}`);
        const started = process.hrtime.bigint();
        const result = spawnSync(
          "/usr/bin/time",
          ["-f", "%e %U %S %M", "-o", prefix + ".time", string(binary.file), "content-mapper"],
          {
            input,
            cwd: project,
            env: process.env,
            timeout: 90_000,
            maxBuffer: 64 * 1024 * 1024,
          },
        );
        const wallNanoseconds = Number(process.hrtime.bigint() - started);
        fs.writeFileSync(prefix + ".stdin", input);
        fs.writeFileSync(prefix + ".stdout", result.stdout ?? Buffer.alloc(0));
        fs.writeFileSync(prefix + ".stderr", result.stderr ?? Buffer.alloc(0));
        save(prefix + ".process.json", {
          executable: binary,
          argv: ["content-mapper"],
          cwd: project,
          status: result.status,
          signal: result.signal,
          error: result.error?.message ?? null,
          wallNanoseconds,
        });
        assert.equal(result.error, undefined);
        assert.equal(result.signal, null);
        assert.equal(result.status, 0);
        const responses = packet(result.stdout);
        const generated = record(record(responses[2]).result);
        assert.equal(generated.extension, ".tsx");
        assert.ok(string(generated.text).includes("const choiceItem0"));
        assert.deepEqual(array(generated.diagnostics), []);
        assert.ok(array(generated.mappings).length > 0);
        packets[role] = responses;
        const timing = fs
          .readFileSync(prefix + ".time", "utf8")
          .trim()
          .split(/\s+/u)
          .map(Number);
        assert.equal(timing.length, 4);
        assert.ok(timing.every(Number.isFinite));
        save(prefix + ".packets.json", responses);
        rows.push({
          role,
          round,
          wallNanoseconds,
          timeElapsedSeconds: timing[0],
          userSeconds: timing[1],
          systemSeconds: timing[2],
          maximumProcessRssKiB: timing[3],
        });
      }
      assert.deepEqual(
        packets.base,
        packets.head,
        "all four normal mapper packets must remain identical",
      );
    }
    const byRole = (role: string): number[] =>
      rows.filter((row) => row.role === role).map((row) => number(row.wallNanoseconds));
    normal.push({
      shape,
      count,
      scriptBindings,
      sourceSha256: sha(source),
      baseMedianWallNs: median(byRole("base")),
      headMedianWallNs: median(byRole("head")),
      rows,
    });
  }
  save(path.join(root, "comparison.json"), {
    schema: "vize.component-scope-binding-comparison.v1",
    status: "pass",
    provenance,
    binaries,
    profiles,
    normal,
    scope: {
      normal:
        "fresh standalone Rust CLI process, initialize/open/one transform/close; no backend type checking; no persistent mapper cache, no warmups; OS page cache not flushed",
      allocation:
        "explicit common source overlays in profiling-only builds; counts/requested layout traffic are not process RSS; runtime-off profiling timings are not normal release timing",
      performanceGate:
        "normal wall measurements report-only; source allocation and exact complete-packet nonregression required",
    },
  });
}
main(process.argv.slice(2));
