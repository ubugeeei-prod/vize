/** Semantic admission for the zero-argument slot-constructor call shape. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  attrValuePredecessors,
  linePrefixAt,
} from "../../support/compat/content-mapper/attr-value-transition.ts";
import { authoredDirectives } from "../../support/compat/content-mapper/producer-unmapped-ignores.ts";
import { parenthesisDiagnosticMappings } from "../../../tests/tooling/support/content-mapper-parenthesis-mappings-11675.ts";
import { isRecord, jsonValue, type PacketObservation } from "./canon-instance-directives-model.ts";
import { save } from "./canon-instance-directives-io.ts";
import { assertConstructorWrapperMappings } from "./canon-slot-constructor-wrapper-mappings.ts";

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

export function canonicalMappedGenerated(source: string): string {
  return source.replace(
    /(\bconst __vize_required_slots_\d+\s*:\s*)\$RequiredSlots<\$SlotContract_(\d+),/gu,
    "$1__VizeRequiredSlots<__VizeSlotContract_$2,",
  );
}

function mappingContract(result: Record<string, unknown>) {
  const source = text(result);
  return tuples(result.mappings).map(
    ([generated, length, original, originalLength, kind, features]) => {
      assert.ok(generated + length <= Buffer.byteLength(source), "mapping fits complete program");
      return {
        generated: canonicalMappedGenerated(bytes(source, generated, generated + length)),
        original,
        originalLength,
        kind,
        features,
      };
    },
  );
}

export function assertMappedContracts(
  before: Record<string, unknown>,
  after: Record<string, unknown>,
) {
  const original = tuples(before.mappings);
  const present = new Set(original.map((row) => JSON.stringify(row)));
  const additions = parenthesisDiagnosticMappings({
    text: text(before),
    mappings: original,
  }).filter((row) => !present.has(JSON.stringify(row)));
  const expected = {
    ...before,
    mappings: [...original, ...additions].sort((a, b) => a[0]! - b[0]!),
  };
  assert.deepEqual(
    mappingContract(after),
    mappingContract(expected),
    "all mapped authored bytes and feature contracts survive with exactly the owned parenthesis atoms",
  );
}

function certifiedDirectives(result: Record<string, unknown>, requireComplete: boolean) {
  const authored = authoredDirectives(
    {
      text: text(result),
      mappings: tuples(result.mappings),
      diagnosticDirectives: result.diagnosticDirectives,
    },
    requireComplete,
  );
  if (authored == null) {
    const { diagnosticDirectives: _removed, ...rest } = result;
    return rest;
  }
  return { ...result, diagnosticDirectives: authored };
}

function directiveContract(result: Record<string, unknown>, historical?: Set<string>) {
  if (result.diagnosticDirectives === undefined) return null;
  assert.ok(isRecord(result.diagnosticDirectives));
  const source = text(result),
    buffer = Buffer.from(source);
  return {
    unused: result.diagnosticDirectives.unusedExpectDirectiveDiagnostics,
    directives: tuples(result.diagnosticDirectives.directives).map(
      ([original, originalLength, start, end, policy, unused]) => {
        const directive = {
          original,
          originalLength,
          generated: bytes(source, start, end),
          policy,
          unused,
        };
        // #12458: an owned per-prop check keeps its plain predecessor's exact image.
        const predecessor = attrValuePredecessors(linePrefixAt(buffer, start), directive.generated)
          .map((generated) => ({ ...directive, generated }))
          .find((image) => historical?.has(JSON.stringify(image)));
        return predecessor ?? directive;
      },
    ),
  };
}

function assertDirectiveDeduplication(
  before: ReturnType<typeof directiveContract>,
  after: ReturnType<typeof directiveContract>,
) {
  if (before === null || after === null) {
    assert.deepEqual(
      after,
      before,
      "the slot host transition may not add or remove the only authored directive",
    );
    return;
  }
  assert.deepEqual(
    after.unused,
    before.unused,
    "unused authored directive diagnostics remain exact",
  );
  const counts = (directives: typeof before.directives) => {
      const result = new Map<string, number>();
      for (const directive of directives) {
        const key = JSON.stringify(directive);
        result.set(key, (result.get(key) ?? 0) + 1);
      }
      return result;
    },
    historical = counts(before.directives),
    current = counts(after.directives);
  assert.deepEqual(
    [...current.keys()].sort(),
    [...historical.keys()].sort(),
    "the slot host transition may only collapse exact duplicate authored directives",
  );
  for (const [directive, count] of current) {
    assert.ok(
      count <= historical.get(directive)!,
      "the slot host transition may not add authored directive copies",
    );
  }
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
  producer: Uint8Array = fs.readFileSync(
    new URL(
      "../../../crates/vize_canon/src/batch/virtual_project/content_mapper.rs",
      import.meta.url,
    ),
  ),
) {
  const fields = new Set([
    "text",
    "mappings",
    "diagnostics",
    "extension",
    "diagnosticDirectives",
    "semanticLinks",
  ]);
  for (const packet of [before, after])
    assert.ok(
      Object.keys(packet).every((key) => fields.has(key)),
      "unknown mapper packet fields are refused",
    );
  assert.deepEqual(
    Object.keys(after)
      .filter((key) => key !== "diagnosticDirectives")
      .sort(),
    Object.keys(before)
      .filter((key) => key !== "diagnosticDirectives")
      .sort(),
    "all mapper packet fields remain exact",
  );
  const baseText = text(before),
    headText = text(after),
    legacy = `__VizeSlotConstructor<typeof ${component}>)({`,
    directConstructorBranches = {
      withoutArguments: directConstructorOwners(headText, "$SlotConstructorWithoutArguments"),
      withArguments: directConstructorOwners(headText, "$SlotConstructorWithArguments"),
    },
    headSemanticLinks = semanticLinkContract(after);
  assert.deepEqual(
    directConstructorBranches,
    {
      withoutArguments: [component],
      withArguments: [component],
    },
    "the reused template host must retain each direct branch exactly once under the component owner",
  );
  assert.ok(!headText.includes(legacy), "the unconditional first argument must be gone");
  assert.ok(
    headText.includes("type $SlotConstructorArgumentLists<C> ="),
    "the complete constructor overload set is inspected",
  );
  assert.ok(
    headText.includes("type $SlotConstructorWithoutArguments<C> =") &&
      headText.includes("type $SlotConstructorWithArguments<C> ="),
    "both structurally valid direct constructor branches are emitted",
  );
  assert.ok(
    headText.includes("type $NativeSlotsFactory<C> = [C] extends [new"),
    "constructor unions use one non-distributive native factory",
  );
  assert.notEqual(headText, baseText, "the source repair must change the virtual program");
  assertConstructorWrapperMappings(
    { text: baseText, rows: tuples(before.mappings) },
    { text: headText, rows: tuples(after.mappings) },
    canonicalMappedGenerated,
    producer,
  );
  const historicalDirectives = directiveContract(certifiedDirectives(before, false));
  assertDirectiveDeduplication(
    historicalDirectives,
    directiveContract(
      certifiedDirectives(after, true),
      new Set(historicalDirectives?.directives.map((directive) => JSON.stringify(directive))),
    ),
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

const contentMapperHelperDeclarations = [
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
] as const;

function materializeSdk(project: string, provider: string, binary: string | null) {
  const modules = path.join(project, "node_modules");
  fs.mkdirSync(modules);
  for (const name of fs.readdirSync(path.join(provider, "node_modules")))
    if (name !== "vize")
      fs.symlinkSync(path.join(provider, "node_modules", name), path.join(modules, name), "dir");
  const source = path.join(provider, "node_modules/vize"),
    target = path.join(modules, "vize");
  fs.mkdirSync(target);
  const declarationRoots = new Set(
    contentMapperHelperDeclarations.map((name) => name.split("/", 1)[0]),
  );
  for (const name of fs.readdirSync(source)) {
    if (name === "package.json") continue;
    if (!declarationRoots.has(name))
      fs.symlinkSync(path.join(source, name), path.join(target, name));
  }
  for (const name of contentMapperHelperDeclarations) {
    const destination = path.join(target, name);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(path.join(source, name), destination);
  }
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

export function selectSdk(project: string, binary: string, declarations?: string) {
  const file = path.join(project, "node_modules/vize/package.json"),
    value = jsonValue(fs.readFileSync(file, "utf8"));
  assert.ok(
    isRecord(value) && isRecord(value.typescript) && isRecord(value.typescript.contentMapper),
  );
  value.typescript.contentMapper.exec = [binary, "content-mapper"];
  save(file, value);
  if (declarations)
    for (const name of contentMapperHelperDeclarations) {
      const destination = path.join(project, "node_modules/vize", name);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.copyFileSync(path.join(declarations, name), destination);
    }
}
