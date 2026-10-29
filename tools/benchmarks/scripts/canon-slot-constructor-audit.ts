/** Semantic admission for the zero-argument slot-constructor call shape. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { isRecord, jsonValue, type PacketObservation } from "./canon-instance-directives-model.ts";
import { save } from "./canon-instance-directives-io.ts";

export type Row = { file: string; line: number; column: number; code: number; message: string };
type Tuple = [number, number, number, number, number, number];

function tuples(value: unknown): Tuple[] {
  assert.ok(Array.isArray(value));
  return value.map((row: unknown) => {
    assert.ok(
      Array.isArray(row) &&
        row.length === 6 &&
        row.every(
          (number: unknown) =>
            typeof number === "number" && Number.isSafeInteger(number) && number >= 0,
        ),
    );
    return row as Tuple;
  });
}

function text(result: Record<string, unknown>): string {
  const value = result.text;
  if (typeof value !== "string") throw new TypeError("packet text must be a string");
  return value;
}

function directConstructorOwners(source: string, helper: string): string[] {
  const marker = "new (undefined as unknown as ",
    owners: string[] = [];
  let offset = 0;
  while (offset < source.length) {
    const start = source.indexOf(marker, offset);
    if (start < 0) break;
    const fragment = source.slice(start, start + 768),
      helperOffset = fragment.indexOf(helper),
      typeEnd = fragment.indexOf("<", marker.length),
      typeHead = typeEnd < 0 ? "" : fragment.slice(marker.length, typeEnd);
    if (helperOffset >= marker.length && typeHead.endsWith(helper)) {
      const owner = fragment.slice(helperOffset).match(/typeof\s+([A-Za-z_$][\w$]*)/u)?.[1];
      assert.ok(owner, `${helper} must name the constructor expression it owns`);
      owners.push(owner);
    }
    offset = start + marker.length;
  }
  return owners;
}

function bytes(source: string, start: number, end: number): string {
  return Buffer.from(source).subarray(start, end).toString("utf8");
}

function offset(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0)
    throw new TypeError(`semantic link ${field} must be a non-negative safe integer`);
  return value;
}

function mappingContract(result: Record<string, unknown>) {
  const source = text(result);
  return tuples(result.mappings).map(
    ([generated, length, original, originalLength, kind, features]) => ({
      generated: bytes(source, generated, generated + length),
      original,
      originalLength,
      kind,
      features,
    }),
  );
}

function directiveContract(result: Record<string, unknown>) {
  if (result.diagnosticDirectives === undefined) return null;
  assert.ok(isRecord(result.diagnosticDirectives));
  const source = text(result);
  return {
    unused: result.diagnosticDirectives.unusedExpectDirectiveDiagnostics,
    directives: tuples(result.diagnosticDirectives.directives).map(
      ([original, originalLength, start, end, policy, unused]) => ({
        original,
        originalLength,
        generated: bytes(source, start, end),
        policy,
        unused,
      }),
    ),
  };
}

function semanticLinkContract(result: Record<string, unknown>) {
  if (result.semanticLinks === undefined) return null;
  assert.ok(Array.isArray(result.semanticLinks));
  const source = text(result);
  return result.semanticLinks.map((link: unknown) => {
    assert.ok(isRecord(link));
    const sourceStart = offset(link.sourceStart, "sourceStart"),
      sourceLength = offset(link.sourceLength, "sourceLength"),
      targetStart = offset(link.targetStart, "targetStart"),
      targetLength = offset(link.targetLength, "targetLength"),
      kind = link.kind;
    if (typeof kind !== "string") throw new TypeError("semantic link kind must be a string");
    return {
      source: bytes(source, sourceStart, sourceStart + sourceLength),
      target: bytes(source, targetStart, targetStart + targetLength),
      kind,
    };
  });
}

export function auditPackets(
  before: Record<string, unknown>,
  after: Record<string, unknown>,
  component = "WitnessChild",
) {
  const baseText = text(before),
    headText = text(after),
    legacy = `__VizeSlotConstructor<typeof ${component}>)({`,
    directConstructorBranches = {
      withoutArguments: directConstructorOwners(headText, "__VizeSlotConstructorWithoutArguments"),
      withArguments: directConstructorOwners(headText, "__VizeSlotConstructorWithArguments"),
    },
    headSemanticLinks = semanticLinkContract(after);
  assert.deepEqual(
    directConstructorBranches,
    {
      withoutArguments: [component, component],
      withArguments: [component, component],
    },
    "both inference phases must retain both direct branches under the exact component owner",
  );
  assert.ok(!headText.includes(legacy), "the unconditional first argument must be gone");
  assert.ok(
    headText.includes("type __VizeSlotConstructorArgumentLists<C> ="),
    "the complete constructor overload set is inspected",
  );
  assert.ok(
    headText.includes("type __VizeSlotConstructorWithoutArguments<C> =") &&
      headText.includes("type __VizeSlotConstructorWithArguments<C> ="),
    "both structurally valid direct constructor branches are emitted",
  );
  assert.ok(
    headText.includes("type __VizeNativeSlotsFactory<C> = [C] extends [new"),
    "constructor unions use one non-distributive native factory",
  );
  assert.notEqual(headText, baseText, "the source repair must change the virtual program");
  assert.deepEqual(
    mappingContract(after),
    mappingContract(before),
    "all mapped authored bytes and feature contracts survive",
  );
  assert.deepEqual(
    directiveContract(after),
    directiveContract(before),
    "no code-agnostic diagnostic suppression may be added",
  );
  assert.deepEqual(
    semanticLinkContract(after),
    semanticLinkContract(before),
    "semantic links retain their exact generated endpoints",
  );
  assert.deepEqual(after.diagnostics, before.diagnostics, "mapper-owned diagnostics stay exact");
  assert.equal(after.extension, before.extension);
  return {
    baseBytes: Buffer.byteLength(baseText),
    headBytes: Buffer.byteLength(headText),
    mappings: tuples(after.mappings).length,
    directives: directiveContract(after)?.directives.length ?? 0,
    semanticLinks: headSemanticLinks?.length ?? 0,
    directConstructorBranches,
  };
}

export function cliContract(observation: PacketObservation) {
  return {
    diagnostics: observation.diagnostics,
    result: {
      ...observation.result,
      files: observation.result.files.map(({ virtualTs: _virtualTs, ...file }) => file),
    },
  };
}

export function native(stdout: Buffer, status: number | null, project: string): Row[] {
  const source = new TextDecoder("utf8", { fatal: true }).decode(stdout),
    rows: Row[] = [];
  for (const match of source.matchAll(/^(.+?)\((\d+),(\d+)\): error TS(\d+): (.*)$/gmu)) {
    const [, file, line, column, code, message] = match;
    assert.ok(file && line && column && code && message !== undefined);
    rows.push({
      file: path.relative(project, path.resolve(project, file)).replaceAll("\\", "/"),
      line: +line,
      column: +column,
      code: +code,
      message,
    });
  }
  assert.equal(rows.length, (source.match(/error TS\d+:/gu) ?? []).length);
  assert.equal(status, rows.length ? 2 : 0);
  return rows;
}

function materializeSdk(project: string, provider: string, binary: string | null) {
  const modules = path.join(project, "node_modules");
  fs.mkdirSync(modules);
  for (const name of fs.readdirSync(path.join(provider, "node_modules")))
    if (name !== "vize")
      fs.symlinkSync(path.join(provider, "node_modules", name), path.join(modules, name), "dir");
  const source = path.join(provider, "node_modules/vize"),
    target = path.join(modules, "vize");
  fs.mkdirSync(target);
  for (const name of fs.readdirSync(source))
    if (name !== "package.json") fs.symlinkSync(path.join(source, name), path.join(target, name));
  const manifest = jsonValue(fs.readFileSync(path.join(source, "package.json"), "utf8"));
  assert.ok(
    isRecord(manifest) &&
      isRecord(manifest.typescript) &&
      isRecord(manifest.typescript.contentMapper),
  );
  if (binary !== null) manifest.typescript.contentMapper.exec = [binary, "content-mapper"];
  save(path.join(target, "package.json"), manifest);
}

export function sdk(project: string, provider: string, binary: string) {
  materializeSdk(project, provider, binary);
}

/** Preserve the installed package's own content-mapper entrypoint for the public baseline. */
export function installedSdk(project: string, provider: string) {
  materializeSdk(project, provider, null);
}

export function selectSdk(project: string, binary: string) {
  const file = path.join(project, "node_modules/vize/package.json"),
    value = jsonValue(fs.readFileSync(file, "utf8"));
  assert.ok(
    isRecord(value) && isRecord(value.typescript) && isRecord(value.typescript.contentMapper),
  );
  value.typescript.contentMapper.exec = [binary, "content-mapper"];
  save(file, value);
}
