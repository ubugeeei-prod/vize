import assert from "node:assert/strict";
import { projectionKinds } from "./typechecker-template-emit-projection-history-model.ts";
import type {
  CoordinateSurfaceProof,
  HistoricalEdit,
  MaestroDocumentProof,
  ProjectionBytes,
  ProjectionCoordinateProof,
  ProjectionEditScript,
  ProjectionFamily,
  ProjectionKind,
  RetainedProjectionManifest,
  RetainedProjectionPackets,
} from "./typechecker-template-emit-projection-history-model.ts";
import {
  applyInverseEdits,
  asBuffer,
  historicalInterval,
  historicalOffset,
  retainedFamilies,
  sha256,
} from "./typechecker-template-emit-projection-history-shared.ts";

type CoordinateReplacement = (match: string, ...captures: string[]) => string;

interface CoordinateSurface {
  path: string;
  normalize: (row: string) => string;
}

export function textRows(bytes: ProjectionBytes): string[] {
  const text = asBuffer(bytes).toString("utf8");
  return text === ""
    ? []
    : text.split("\n").filter((row, index, rows) => row !== "" || index < rows.length - 1);
}

function rowsDigest(rows: readonly string[]): string {
  return sha256(
    Buffer.from(
      `${JSON.stringify([...rows].sort((left, right) => (left < right ? -1 : left > right ? 1 : 0)))}\n`,
    ),
  );
}

function generatedRange(
  startValue: number | string,
  endValue: number | string,
  edits: readonly HistoricalEdit[],
): [number, number] {
  const start = Number(startValue);
  const end = Number(endValue);
  assert(Number.isSafeInteger(start) && Number.isSafeInteger(end) && start >= 0 && end >= start);
  for (const edit of edits) {
    assert(
      end <= edit.afterStart || start >= edit.afterEnd,
      "generated span overlaps a changed helper range",
    );
  }
  return [historicalOffset(start, edits), historicalOffset(end, edits)];
}

export function replacePair(
  line: string,
  expression: RegExp,
  replace: CoordinateReplacement,
  expected = 1,
): string {
  let matches = 0;
  const result = line.replace(expression, (match: string, ...args: unknown[]) => {
    matches += 1;
    return replace(match, ...args.map(String));
  });
  assert.equal(matches, expected, `unexpected coordinate count in ${line}`);
  return result;
}

function normalizeCanonMapping(line: string, edits: readonly HistoricalEdit[]): string {
  let matches = 0;
  const result = line.replace(
    /(\d+):(\d+)>(\d+):(\d+)/gu,
    (_match, start, end, source, sourceEnd) => {
      matches += 1;
      const [previousStart, previousEnd] = generatedRange(start, end, edits);
      return `${previousStart}:${previousEnd}>${source}:${sourceEnd}`;
    },
  );
  assert(matches > 0, `missing canon mapping coordinate in ${line}`);
  return result;
}

function normalizeCanonLink(line: string, edits: readonly HistoricalEdit[]): string {
  return replacePair(
    line,
    /^(.*?:)(\d+):(\d+)>(\d+):(\d+)$/u,
    (_match, prefix, start, end, target, targetEnd) => {
      const [previousStart, previousEnd] = generatedRange(start, end, edits);
      const [previousTarget, previousTargetEnd] = generatedRange(target, targetEnd, edits);
      return `${prefix}${previousStart}:${previousEnd}>${previousTarget}:${previousTargetEnd}`;
    },
  );
}

function normalizeMapperMapping(line: string, edits: readonly HistoricalEdit[]): string {
  const row = JSON.parse(line) as number[];
  assert(Array.isArray(row) && row.length === 6, `invalid content-mapper row: ${line}`);
  const [start, length] = historicalInterval(row[0], row[1], edits);
  row[0] = start;
  row[1] = length;
  return `[${row.join(", ")}]`;
}

function normalizeMapperLink(line: string, edits: readonly HistoricalEdit[]): string {
  return replacePair(
    line,
    /^(\d+):(\d+)>(\d+):(\d+):([^:]+)$/u,
    (_match, source, sourceLength, target, length, kind) => {
      const [previousSource, previousSourceLength] = historicalInterval(
        source,
        sourceLength,
        edits,
      );
      const [previousTarget, previousLength] = historicalInterval(target, length, edits);
      return `${previousSource}:${previousSourceLength}>${previousTarget}:${previousLength}:${kind}`;
    },
  );
}

function normalizeMapperHit(line: string, edits: readonly HistoricalEdit[]): string {
  return replacePair(
    line,
    /^(.*?\|)(\d+):(\d+)>(\d+):(\d+)(\|.*)$/u,
    (_match, prefix, start, length, source, sourceLength, suffix) => {
      const [previousStart, previousLength] = historicalInterval(start, length, edits);
      return `${prefix}${previousStart}:${previousLength}>${source}:${sourceLength}${suffix}`;
    },
  );
}

function normalizeMaestroRow(
  line: string,
  edits: readonly HistoricalEdit[],
  templateUri: string,
  authored: boolean,
): string {
  const parts = line.split("|");
  assert(parts.length >= 4, `invalid maestro row: ${line}`);
  const uri = parts[authored ? 1 : 0];
  if (uri !== templateUri) return line;
  parts[2] = replacePair(
    parts[2],
    /^(\d+):(\d+)>(\d+):(\d+)$/u,
    (_match, source, sourceEnd, start, end) => {
      const [previousStart, previousEnd] = generatedRange(start, end, edits);
      return `${source}:${sourceEnd}>${previousStart}:${previousEnd}`;
    },
  );
  return parts.join("|");
}

function reconcileCoordinateRows(
  path: string,
  beforeBytes: ProjectionBytes,
  afterBytes: ProjectionBytes,
  normalize: (row: string) => string,
): CoordinateSurfaceProof {
  const baseline = textRows(beforeBytes);
  const incident = textRows(afterBytes);
  const remaining = new Map<string, number>();
  for (const row of baseline) remaining.set(row, (remaining.get(row) ?? 0) + 1);
  const matched: string[] = [];
  const added: string[] = [];
  const rejected: string[] = [];
  for (const row of incident) {
    try {
      const normalized = normalize(row);
      const count = remaining.get(normalized) ?? 0;
      if (count > 0) {
        remaining.set(normalized, count - 1);
        matched.push(normalized);
      } else added.push(normalized);
    } catch (error) {
      assert(error instanceof Error);
      rejected.push(`${row}\n${error.message}`);
    }
  }
  const missing: string[] = [];
  for (const row of baseline) {
    const count = remaining.get(row) ?? 0;
    if (count > 0) {
      missing.push(row);
      remaining.set(row, count - 1);
    }
  }
  return {
    path,
    baselineRows: baseline.length,
    incidentRows: incident.length,
    matchedRows: matched.length,
    addedRows: added.length,
    rejectedRows: rejected.length,
    missingRows: missing.length,
    matchedSha256: rowsDigest(matched),
    addedSha256: rowsDigest(added),
    rejectedSha256: rowsDigest(rejected),
    missingSha256: rowsDigest(missing),
  };
}

function maestroDocumentProof(
  family: ProjectionFamily,
  packets: RetainedProjectionPackets,
  editScript: ProjectionEditScript,
): MaestroDocumentProof {
  const manifests: Record<ProjectionKind, string[]> = { baseline: [], incident: [] };
  for (const kind of projectionKinds) {
    const path = `${family}/maestro/text.txt`;
    const manifestBytes = packets[kind].get(path);
    assert(manifestBytes, `missing retained maestro manifest ${family}/${kind}`);
    manifests[kind] = textRows(manifestBytes);
    assert(manifests[kind].length > 0, `${kind} maestro manifest is empty`);
    for (const [index, row] of manifests[kind].entries()) {
      const [uri, _language, bytes, digest] = row.split("|");
      assert(uri && bytes && digest, `invalid maestro document row: ${row}`);
      const document = packets[kind].get(`${family}/maestro-document-${index}/text.ts`);
      assert(document, `missing retained maestro document ${family}/${kind}/${index}`);
      assert.equal(document.length, Number(bytes));
      assert.equal(sha256(document), digest);
    }
  }
  assert.equal(manifests.baseline.length, manifests.incident.length);
  const normalized: string[] = [];
  let templateUri: string | undefined;
  for (const [index, row] of manifests.incident.entries()) {
    const [uri, language] = row.split("|");
    const before = packets.baseline.get(`${family}/maestro-document-${index}/text.ts`);
    const after = packets.incident.get(`${family}/maestro-document-${index}/text.ts`);
    assert(before && after);
    if (language === "Template") {
      assert.equal(templateUri, undefined, `duplicate template document for ${family}`);
      templateUri = uri;
      assert.deepEqual(applyInverseEdits(after, before, editScript.edits), before);
    } else assert.deepEqual(after, before, `unexpected non-template document drift: ${uri}`);
    normalized.push(`${uri}|${language}|${before.length}|${sha256(before)}`);
  }
  assert(templateUri, `missing template document for ${family}`);
  assert.deepEqual(normalized, manifests.baseline);
  return {
    family,
    templateUri,
    documents: normalized.length,
    normalizedManifestSha256: rowsDigest(normalized),
  };
}

/** Classify every coordinate as stable, added, removed, or helper-internal. */
export function projectionCoordinateProof(
  manifest: Pick<RetainedProjectionManifest, "editScripts">,
  packets: RetainedProjectionPackets,
): ProjectionCoordinateProof {
  const scripts = new Map<string, ProjectionEditScript>(
    manifest.editScripts.map((script) => [script.id, script]),
  );
  assert.equal(
    scripts.size,
    retainedFamilies.length * 3,
    "missing or duplicate projection edit script",
  );
  const surfaces: CoordinateSurfaceProof[] = [];
  const documents: MaestroDocumentProof[] = [];
  for (const family of retainedFamilies) {
    const get = (kind: ProjectionKind, path: string): Buffer => {
      const bytes = packets[kind].get(`${family}/${path}`);
      assert(bytes, `missing retained ${kind} path: ${family}/${path}`);
      return bytes;
    };
    const canon = scripts.get(`${family}/canon`);
    const mapper = scripts.get(`${family}/content-mapper`);
    const maestro = scripts.get(`${family}/maestro`);
    assert(canon && mapper && maestro);
    const coordinateSurfaces: CoordinateSurface[] = [
      { path: "canon/mappings.txt", normalize: (row) => normalizeCanonMapping(row, canon.edits) },
      {
        path: "canon/semantic-links.txt",
        normalize: (row) => normalizeCanonLink(row, canon.edits),
      },
      {
        path: "content-mapper/mappings.txt",
        normalize: (row) => normalizeMapperMapping(row, mapper.edits),
      },
      {
        path: "content-mapper/semantic-links.txt",
        normalize: (row) => normalizeMapperLink(row, mapper.edits),
      },
      {
        path: "content-mapper/authored-hits.txt",
        normalize: (row) => normalizeMapperHit(row, mapper.edits),
      },
    ];
    for (const { path, normalize } of coordinateSurfaces)
      surfaces.push(
        reconcileCoordinateRows(
          `${family}/${path}`,
          get("baseline", path),
          get("incident", path),
          normalize,
        ),
      );
    const document = maestroDocumentProof(family, packets, maestro);
    documents.push(document);
    for (const [path, authored] of [
      ["maestro/mappings.txt", false],
      ["maestro/authored-hits.txt", true],
    ] as const)
      surfaces.push(
        reconcileCoordinateRows(
          `${family}/${path}`,
          get("baseline", path),
          get("incident", path),
          (row) => normalizeMaestroRow(row, maestro.edits, document.templateUri, authored),
        ),
      );
  }
  for (const family of retainedFamilies) {
    const rows = surfaces.filter((proof) => proof.path.startsWith(`${family}/`));
    assert(
      rows.some((proof) => proof.matchedRows > 0),
      `no stable coordinate witness for ${family}`,
    );
  }
  const proof = { schema: 1, surfaces, documents };
  return { ...proof, sha256: sha256(Buffer.from(`${JSON.stringify(proof)}\n`)) };
}
