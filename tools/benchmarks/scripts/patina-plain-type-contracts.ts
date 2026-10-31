import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import type { RuntimePacketContract } from "./patina-plain-type-runtime-counts.ts";

export const LEGACY_PACKET_REVISION = "6cd169dba2c41daef87cf2a1eabe49a2fb470858";
export const LEGACY_PACKET_SHA = "c72d860c16caa1fae043ddde528155a32f3de1ced1de02ad5a32fcb4b72f8c3f";
export const ERROR_PACKET_SHA = "fb234217aea275ae713a1e8a3174c970cd55ab8a76ed833b4135564f96a1a09c";
export const PACKET_MODULE_SUFFIX = "\nmod packets;\n";
const sha = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");
const record = (value: unknown): Record<string, unknown> => {
  assert(value && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
};

export function packetTestContract(bytes: Buffer): RuntimePacketContract {
  const digest = sha(bytes);
  if (digest === LEGACY_PACKET_SHA) return "runtime-warning-v1";
  assert.equal(digest, ERROR_PACKET_SHA, "packet test needs an audited original source contract");
  return "runtime-error-v2";
}

export function planPacketReplay(parent: Buffer, child: Buffer | undefined, legacy: Buffer) {
  assert.equal(packetTestContract(legacy), "runtime-warning-v1");
  const declarations = parent.toString().match(/^mod packets;$/gmu) ?? [];
  if (child) {
    assert.equal(declarations.length, 1, "existing original packet module must be registered once");
    return { parent, child, overlay: false, contract: packetTestContract(child) };
  }
  assert.equal(declarations.length, 0, "missing child cannot replace a registered original test");
  return {
    parent: Buffer.concat([parent, Buffer.from(PACKET_MODULE_SUFFIX)]),
    child: legacy,
    overlay: true,
    contract: "runtime-warning-v1" as const,
  };
}

const caseNames = [
  "active",
  "javascript",
  "declaration-ts",
  "declaration-mts",
  "declaration-cts",
  "default",
  "host-disabled",
  "rule-disabled",
  "unavailable",
  "no-query",
];
const ids = ["promise", "reactivity"].flatMap((kind) => caseNames.map((name) => `${kind}/${name}`));

/** Locate untouched byte spans after JSON.parse validates the complete document. */
function packetSpans(bytes: Buffer) {
  const text = bytes.toString("utf8");
  assert.deepEqual(Buffer.from(text), bytes, "packet document must be exact UTF-8");
  const document = record(JSON.parse(text) as unknown);
  assert.deepEqual(Object.keys(document).sort(), ["packets", "provider_version"]);
  assert.equal(document.provider_version, "7.0.2");
  assert(Array.isArray(document.packets) && document.packets.length === 20);
  const rows = (document.packets as unknown[]).map(record);
  assert.deepEqual(
    rows.map((row) => `${row.kind}/${row.case}`),
    ids,
  );
  let depth = 0,
    quoted = false,
    escaped = false,
    token = -1,
    arrayStart = -1;
  const rootKeys: string[] = [];
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (quoted) {
      if (escaped) escaped = false;
      else if (c === "\\") escaped = true;
      else if (c === '"') {
        quoted = false;
        if (depth === 1) {
          let j = i + 1;
          while (/\s/u.test(text[j] ?? "!")) j++;
          if (text[j] === ":") {
            const key: unknown = JSON.parse(text.slice(token, i + 1));
            assert.equal(typeof key, "string");
            rootKeys.push(key as string);
            if (key === "packets") {
              j++;
              while (/\s/u.test(text[j] ?? "!")) j++;
              assert.equal(text[j], "[");
              assert.equal(arrayStart, -1);
              arrayStart = j;
            }
          }
        }
      }
    } else if (c === '"') {
      quoted = true;
      token = i;
    } else if (c === "{" || c === "[") depth++;
    else if (c === "}" || c === "]") depth--;
  }
  assert.deepEqual(
    rootKeys.sort(),
    ["packets", "provider_version"],
    "duplicate root fields refused",
  );
  assert(arrayStart >= 0);
  const spans: Array<readonly [number, number]> = [];
  depth = 1;
  quoted = false;
  escaped = false;
  let start = -1;
  for (let i = arrayStart + 1; i < text.length && depth; i++) {
    const c = text[i];
    if (quoted) {
      if (escaped) escaped = false;
      else if (c === "\\") escaped = true;
      else if (c === '"') quoted = false;
    } else if (c === '"') quoted = true;
    else if (c === "{" || c === "[") {
      if (depth === 1) {
        assert.equal(c, "{");
        start = i;
      }
      depth++;
    } else if (c === "}" || c === "]") {
      depth--;
      if (depth === 1) {
        assert.equal(c, "}");
        assert(start >= 0);
        spans.push([
          Buffer.byteLength(text.slice(0, start)),
          Buffer.byteLength(text.slice(0, i + 1)),
        ]);
        start = -1;
      }
    }
  }
  assert.equal(spans.length, 20);
  const raw = spans.map(([start, end], index) => {
    const cell = bytes.subarray(start, end);
    assert.deepEqual(JSON.parse(cell.toString()) as unknown, rows[index]);
    return cell;
  });
  const gaps = spans.map(([start], index) =>
    bytes.subarray(index ? spans[index - 1]![1] : 0, start),
  );
  gaps.push(bytes.subarray(spans[19]![1]));
  return { rows, raw, gaps };
}

const oldHelp =
  "Type-aware lint rules were skipped because the native compiler could not be started. Install a native TypeScript runtime in this project or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH` to its executable.";
const newHelp =
  "Requested type-aware lint could not initialize its native runtime. Check the reported failure, install a compatible native TypeScript runtime, or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH` to its executable.";

function unavailable(kind: string, source: string, root: string, contract: RuntimePacketContract) {
  const failed = contract === "runtime-error-v2";
  const result = {
    filename: path.join(root, `${kind}-unavailable.tsx`),
    error_count: failed ? 1 : 0,
    warning_count: failed ? 0 : 1,
    diagnostics: [
      {
        rule_name: "type/corsa-runtime",
        severity: failed ? "error" : "warning",
        message: `Configured Corsa executable does not exist: ${path.join(root, "missing-provider")} (from configuration)`,
        start: 0,
        end: 0,
        help: failed ? newHelp : oldHelp,
        labels: [],
        fix: null,
      },
    ],
  };
  return { kind, case: "unavailable", source, expected: result, actual: result };
}

export function verifyPacketTransition(
  before: Buffer,
  after: Buffer,
  baseContract: RuntimePacketContract,
  headContract: RuntimePacketContract,
  inputs: ReadonlyMap<string, string>,
  root: string,
) {
  assert(["runtime-warning-v1", "runtime-error-v2"].includes(baseContract));
  assert(["runtime-warning-v1", "runtime-error-v2"].includes(headContract));
  assert(
    !(baseContract === "runtime-error-v2" && headContract === "runtime-warning-v1"),
    "runtime error contract cannot regress to warning",
  );
  const base = packetSpans(before),
    head = packetSpans(after);
  const migrating = baseContract !== headContract;
  for (let i = 0; i < base.gaps.length; i++)
    assert.deepEqual(base.gaps[i], head.gaps[i], "packet envelope/framing changed");
  const cells = ids.map((id, index) => {
    const oldRow = base.rows[index]!,
      newRow = head.rows[index]!;
    const changed = migrating && id.endsWith("/unavailable");
    if (id.endsWith("/unavailable")) {
      const kind = id.split("/")[0]!;
      const source = inputs.get(`${kind}.tsx.txt`);
      assert.equal(typeof source, "string");
      assert.deepEqual(
        oldRow,
        unavailable(kind, source as string, root, baseContract),
        `${id} complete before contract`,
      );
      assert.deepEqual(
        newRow,
        unavailable(kind, source as string, root, headContract),
        `${id} complete after contract`,
      );
    }
    if (!changed)
      assert.deepEqual(
        base.raw[index],
        head.raw[index],
        `${id} entire original cell bytes changed`,
      );
    return {
      id,
      changed,
      beforeSha256: sha(base.raw[index]!),
      afterSha256: sha(head.raw[index]!),
      ...(changed ? { before: oldRow, after: newRow } : {}),
    };
  });
  if (!migrating)
    assert.deepEqual(before, after, "same contract requires all twenty document bytes unchanged");
  assert.equal(cells.filter((cell) => cell.changed).length, migrating ? 2 : 0);
  return {
    baseContract,
    headContract,
    packetCount: 20,
    unchanged: migrating ? 18 : 20,
    changed: migrating ? 2 : 0,
    beforeSha256: sha(before),
    afterSha256: sha(after),
    cells,
  };
}
