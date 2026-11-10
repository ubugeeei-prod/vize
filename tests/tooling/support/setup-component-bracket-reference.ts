import assert from "node:assert/strict";
import { closedSetupReference, type ExposurePacket } from "./script-setup-default-expose.ts";
import {
  isTypeScriptSource,
  moduleShapeFactsReference,
  type ParseModule,
  typescriptErasureReference,
} from "./typescript-erasure-reference.ts";

type ComponentModuleShape = NonNullable<ExposurePacket["moduleShape"]> & {
  defaultExportStart?: number;
  defaultExportKeywordEnd?: number;
};

type RuntimeOptionRow = {
  target: string;
  options: { inlineTemplate?: unknown; [key: string]: unknown };
  single?: ExposurePacket[];
  batch?: { results: ExposurePacket[] };
  stats?: { outputBytes: number };
  [key: string]: unknown;
};

type MaybeRefNormalization = { imports: number; reads: number };

function normalizeExternalMaybeRefRead(value: unknown, changed: MaybeRefNormalization): unknown {
  if (Array.isArray(value)) {
    return value.map((child) => normalizeExternalMaybeRefRead(child, changed));
  }
  if (!value || typeof value !== "object") return value;
  const node = value as Record<string, unknown>;
  if (
    node.type === "CallExpression" &&
    (node.callee as { type?: unknown; name?: unknown } | undefined)?.type === "Identifier" &&
    (node.callee as { name?: unknown }).name === "_unref" &&
    Array.isArray(node.arguments) &&
    node.arguments.length === 1 &&
    (node.arguments[0] as { type?: unknown; name?: unknown }).type === "Identifier" &&
    (node.arguments[0] as { name?: unknown }).name === "n"
  ) {
    changed.reads += 1;
    return {
      type: "MemberExpression",
      object: normalizeExternalMaybeRefRead(node.arguments[0], changed),
      computed: false,
      property: { type: "Identifier", name: "value" },
    };
  }
  const out = Object.fromEntries(
    Object.entries(node).map(([key, child]) => [
      key,
      normalizeExternalMaybeRefRead(child, changed),
    ]),
  );
  if (
    node.type === "ImportDeclaration" &&
    (node.source as { value?: unknown } | undefined)?.value === "vue" &&
    Array.isArray(out.specifiers)
  ) {
    out.specifiers = out.specifiers.filter((specifier) => {
      const isUnref =
        (specifier as { type?: unknown }).type === "ImportSpecifier" &&
        (specifier as { imported?: { name?: unknown } }).imported?.name === "unref" &&
        (specifier as { local?: { name?: unknown } }).local?.name === "_unref";
      if (isUnref) changed.imports += 1;
      return !isUnref;
    });
  }
  return out;
}

// Vue 3.5 treats a ref factory imported from another module as MaybeRef. The
// DOM inline renderer therefore adds one `unref` helper and unwraps the one
// interpolated read, while the authored increment remains `n.value++`.
function assertExternalMaybeRefReadProgram(actual: unknown, preserved: unknown): void {
  const changed = { imports: 0, reads: 0 };
  const normalized = normalizeExternalMaybeRefRead(actual, changed);
  assert.deepEqual(changed, { imports: 1, reads: 1 });
  assert.deepEqual(normalized, preserved, "only the external MaybeRef read advances");
}

// Advance one preserved non-inline setup component read by the independently
// specified #10877 edit. The reference owns the complete packet: code and each
// affected byte offset are reconstructed, while maps and all other fields must
// remain byte-for-byte equal to the preserved producer.
export function setupComponentBracketReference(
  packet: ExposurePacket,
  component: string,
): ExposurePacket {
  assert.match(component, /^[$A-Z_a-z][$\w]*$/, "the setup component must be an identifier");
  const out = structuredClone(packet);
  const before = `$setup.${component}`;
  const start = packet.code.indexOf(before);
  if (start === -1) return out;
  assert.equal(
    packet.code.indexOf(before, start + before.length),
    -1,
    `the preserved packet must contain one ${before} read`,
  );
  const after = `$setup[${JSON.stringify(component)}]`;
  out.code = packet.code.slice(0, start) + after + packet.code.slice(start + before.length);
  const byteDelta = Buffer.byteLength(after) - Buffer.byteLength(before);
  const moduleShape = out.moduleShape as ComponentModuleShape | undefined;
  if (moduleShape) {
    for (const key of [
      "defaultExportStart",
      "defaultExportKeywordEnd",
      "defaultExportEnd",
    ] as const) {
      const offset = moduleShape[key];
      if (typeof offset === "number" && offset > start) moduleShape[key] = offset + byteDelta;
    }
  }
  return out;
}

// `erased` is the current row: a TypeScript source's packets advance by the
// per-piece erasure only after its program, offsets and map check out.
export function advanceRuntimeOptionReference<T extends RuntimeOptionRow>(
  row: T,
  sourceFiles: readonly string[],
  erased: RuntimeOptionRow,
  parse: ParseModule,
): T {
  assert.equal(erased.target, row.target);
  assert.deepEqual(erased.options, row.options);
  const out = structuredClone(row);
  const advance = (packets: ExposurePacket[], current: ExposurePacket[] | undefined) =>
    packets.map((packet, index) => {
      const closed = closedSetupReference(packet, sourceFiles[index]);
      if (!isTypeScriptSource(sourceFiles[index])) {
        return moduleShapeFactsReference(closed, parse);
      }
      assert.ok(current?.[index], "the current row holds every TypeScript packet");
      const assertProgram =
        row.target === "dom" && row.options.inlineTemplate === true && index === 2
          ? assertExternalMaybeRefReadProgram
          : undefined;
      return typescriptErasureReference(closed, current[index], parse, assertProgram);
    });
  if (out.single) out.single = advance(out.single, erased.single);
  if (out.batch) out.batch.results = advance(out.batch.results, erased.batch?.results);
  if (row.target === "dom" && row.options.inlineTemplate === undefined && out.single && out.batch) {
    out.single[7] = setupComponentBracketReference(out.single[7], "Lazy");
    out.batch.results[7] = setupComponentBracketReference(out.batch.results[7], "Lazy");
  }
  const beforePackets = row.single;
  if (out.stats && out.single && beforePackets) {
    out.stats.outputBytes += out.single.reduce(
      (bytes, packet, index) =>
        bytes + Buffer.byteLength(packet.code) - Buffer.byteLength(beforePackets[index].code),
      0,
    );
  }
  return out;
}

export function assertDefaultRuntimeOptionPackets(
  before: RuntimeOptionRow[],
  actual: RuntimeOptionRow[],
  sourceFiles: readonly string[],
  parse: ParseModule,
): void {
  const defaults = (rows: RuntimeOptionRow[]) =>
    rows.filter(
      (row) =>
        row.options.runtimeModuleName === undefined && row.options.runtimeGlobalName === undefined,
    );
  const withoutTiming = (row: unknown) => {
    if (!row) return row;
    const { timeMs: _timeMs, ...rest } = row as Record<string, unknown>;
    return rest;
  };
  const comparable = (row: RuntimeOptionRow) => ({
    target: row.target,
    options: row.options,
    single: row.single,
    batch: withoutTiming(row.batch),
    stats: withoutTiming(row.stats),
    function: row.function,
    functionBatch: withoutTiming(row.functionBatch),
    functionStats: withoutTiming(row.functionStats),
    syntax: row.syntax,
  });
  const current = defaults(actual);
  const advance = (row: RuntimeOptionRow, index: number) => {
    const erased = current[index];
    assert.ok(erased, "the current matrix retains every baseline row");
    return advanceRuntimeOptionReference(row, sourceFiles, erased, parse);
  };
  assert.deepEqual(current.map(comparable), defaults(before).map(advance).map(comparable));
}
