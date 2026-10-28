import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve, sep } from "node:path";
import { gunzipSync, gzipSync } from "node:zlib";

/**
 * Preserve the last successful projection receipt and the first failing
 * scheduled packet as independent evidence. Qualification has three layers:
 * exact artifact custody, byte-exact helper inverses, and fail-closed generated
 * coordinate inverses. Current output is checked separately and is never
 * rewritten to force an old digest.
 */
export const retainedFamilies = ["dual-scripts-emits", "define-model"];
export const retainedManifestUrl = new URL(
  "../../../tests/_fixtures/differential/typechecker/template-dollar-emit/projection-retained-10907.manifest.json",
  import.meta.url,
);
export const retainedArtifactCustody = {
  baseline: {
    sourceSha: "c62d96e4b2267987105f9d9d2698a000bf73aad9",
    driverSha: "c62d96e4b2267987105f9d9d2698a000bf73aad9",
    runId: 38236803949,
    jobId: 114769365483,
    artifactId: 11741738141,
    artifactBytes: 96630181,
    createdAt: "2026-10-12T10:45:14Z",
    expiresAt: "2026-11-11T10:45:13Z",
    archiveBytes: 96628392,
    archiveSha256: "3cdce277d4b2bccb725c95135b0c54fa7085ea2af2c1e9d9966ba4659917af48",
    manifestSha256: "92fd955ff55e3e68ac4f74264874eb4a2ae454f9a12088db4fc4142cf8f33361",
    receiptVerifierSha256: "ab51a5f0b704c31eece8af5614ff58fb027f97d325d66b401014f4ece37732c5",
    qualification: "success",
  },
  incident: {
    sourceSha: "b09ac660cd0684396c21d24249c4b1e2193e965a",
    driverSha: "b09ac660cd0684396c21d24249c4b1e2193e965a",
    runId: 40415562052,
    jobId: 121896883497,
    artifactId: 12694328570,
    artifactBytes: 101632777,
    createdAt: "2026-10-26T22:58:12Z",
    expiresAt: "2026-11-25T22:58:08Z",
    archiveBytes: 101630987,
    archiveSha256: "f14356db34626150e0f8056f16dff2ffc2beb36ebfa1652ada79074cbadb80f5",
    manifestSha256: "6592c2a4b0be8caeb2f6bbac8f7d6bf353fc1f577cdea42fc02fd142c1ef705f",
    qualification: "failed-before-receipt",
  },
};
// Updated only after regenerating the fixture twice from the two immutable
// artifacts and observing byte-identical output.
export const retainedManifestSha256 =
  "559ea18dbbde55fac6eaf80f63a5e32959672c11d0698cff9f4579e95caee4cc";

export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const asBuffer = (value) => (Buffer.isBuffer(value) ? value : Buffer.from(value));

function lines(bytes) {
  const rows = [];
  let start = 0;
  for (let index = 0; index < bytes.length; index += 1) {
    if (bytes[index] === 0x0a) {
      rows.push({ start, end: index + 1, bytes: bytes.subarray(start, index + 1) });
      start = index + 1;
    }
  }
  if (start < bytes.length) rows.push({ start, end: bytes.length, bytes: bytes.subarray(start) });
  return rows;
}

function boundary(rows, index, length) {
  return index === rows.length ? length : rows[index].start;
}

function trimEdit(before, after, edit) {
  while (
    edit.beforeStart < edit.beforeEnd &&
    edit.afterStart < edit.afterEnd &&
    before[edit.beforeStart] === after[edit.afterStart]
  ) {
    edit.beforeStart += 1;
    edit.afterStart += 1;
  }
  while (
    edit.beforeStart < edit.beforeEnd &&
    edit.afterStart < edit.afterEnd &&
    before[edit.beforeEnd - 1] === after[edit.afterEnd - 1]
  ) {
    edit.beforeEnd -= 1;
    edit.afterEnd -= 1;
  }
  return edit;
}

// The retained programs are line-oriented generated TypeScript. A line LCS keeps
// independent helper changes independent, then byte trimming narrows single-line
// replacements to their exact payload. The result is stored, not recomputed, by
// qualification so a later diff implementation cannot reinterpret the evidence.
export function diffGeneratedBytes(beforeValue, afterValue) {
  const before = asBuffer(beforeValue);
  const after = asBuffer(afterValue);
  const left = lines(before);
  const right = lines(after);
  assert(left.length < 65_536 && right.length < 65_536, "retained line count overflow");
  const table = Array.from({ length: left.length + 1 }, () => new Uint16Array(right.length + 1));
  for (let leftIndex = left.length - 1; leftIndex >= 0; leftIndex -= 1) {
    for (let rightIndex = right.length - 1; rightIndex >= 0; rightIndex -= 1) {
      table[leftIndex][rightIndex] = left[leftIndex].bytes.equals(right[rightIndex].bytes)
        ? table[leftIndex + 1][rightIndex + 1] + 1
        : Math.max(table[leftIndex + 1][rightIndex], table[leftIndex][rightIndex + 1]);
    }
  }
  const edits = [];
  let leftIndex = 0;
  let rightIndex = 0;
  let pending = null;
  const flush = () => {
    if (!pending) return;
    const edit = trimEdit(before, after, {
      beforeStart: boundary(left, pending.leftStart, before.length),
      beforeEnd: boundary(left, leftIndex, before.length),
      afterStart: boundary(right, pending.rightStart, after.length),
      afterEnd: boundary(right, rightIndex, after.length),
    });
    if (edit.beforeStart !== edit.beforeEnd || edit.afterStart !== edit.afterEnd) edits.push(edit);
    pending = null;
  };
  while (leftIndex < left.length || rightIndex < right.length) {
    if (
      leftIndex < left.length &&
      rightIndex < right.length &&
      left[leftIndex].bytes.equals(right[rightIndex].bytes)
    ) {
      flush();
      leftIndex += 1;
      rightIndex += 1;
    } else {
      pending ??= { leftStart: leftIndex, rightStart: rightIndex };
      if (
        rightIndex === right.length ||
        (leftIndex < left.length &&
          table[leftIndex + 1][rightIndex] >= table[leftIndex][rightIndex + 1])
      )
        leftIndex += 1;
      else rightIndex += 1;
    }
  }
  flush();
  assert.deepEqual(applyInverseEdits(after, before, edits), before);
  return edits;
}

/** Reconstruct the successful generated document from a newer retained packet. */
export function applyInverseEdits(afterValue, beforeValue, edits) {
  const after = asBuffer(afterValue);
  const before = asBuffer(beforeValue);
  const chunks = [];
  let cursor = 0;
  let previousBefore = 0;
  for (const edit of edits) {
    for (const key of ["beforeStart", "beforeEnd", "afterStart", "afterEnd"])
      assert(Number.isSafeInteger(edit[key]) && edit[key] >= 0, `invalid edit ${key}`);
    assert(edit.beforeStart >= previousBefore && edit.beforeEnd >= edit.beforeStart);
    assert(edit.afterStart >= cursor && edit.afterEnd >= edit.afterStart);
    assert(edit.beforeEnd <= before.length && edit.afterEnd <= after.length);
    assert(
      edit.beforeStart !== edit.beforeEnd || edit.afterStart !== edit.afterEnd,
      "empty historical edit",
    );
    chunks.push(after.subarray(cursor, edit.afterStart));
    chunks.push(before.subarray(edit.beforeStart, edit.beforeEnd));
    cursor = edit.afterEnd;
    previousBefore = edit.beforeEnd;
  }
  chunks.push(after.subarray(cursor));
  return Buffer.concat(chunks);
}

/** Map a stable generated offset back through every retained helper edit. */
export function historicalOffset(value, edits) {
  const offset = Number(value);
  assert(Number.isSafeInteger(offset) && offset >= 0, "invalid generated offset");
  let delta = 0;
  for (const edit of edits) {
    const beforeLength = edit.beforeEnd - edit.beforeStart;
    const afterLength = edit.afterEnd - edit.afterStart;
    if (offset < edit.afterStart) return offset + delta;
    if (offset === edit.afterStart) {
      assert(afterLength > 0 || beforeLength === 0, "offset meets a deleted historical range");
      return edit.beforeStart;
    }
    assert(offset >= edit.afterEnd, "generated offset entered a changed helper range");
    if (offset === edit.afterEnd) return edit.beforeEnd;
    delta += beforeLength - afterLength;
  }
  return offset + delta;
}

/** Map a stable generated interval and reject any interval touching changed code. */
export function historicalInterval(start, length, edits) {
  const currentStart = Number(start);
  const currentEnd = currentStart + Number(length);
  assert(Number.isSafeInteger(currentEnd) && Number(length) >= 0, "invalid generated interval");
  for (const edit of edits) {
    assert(
      currentEnd <= edit.afterStart || currentStart >= edit.afterEnd,
      "generated interval overlaps a changed helper range",
    );
  }
  const beforeStart = historicalOffset(currentStart, edits);
  const beforeEnd = historicalOffset(currentEnd, edits);
  return [beforeStart, beforeEnd - beforeStart];
}

function textRows(bytes) {
  const text = asBuffer(bytes).toString("utf8");
  return text === ""
    ? []
    : text.split("\n").filter((row, index, rows) => row !== "" || index < rows.length - 1);
}

function rowsDigest(rows) {
  return sha256(Buffer.from(`${JSON.stringify([...rows].sort())}\n`));
}

function generatedRange(startValue, endValue, edits) {
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

function replacePair(line, expression, replace, expected = 1) {
  let matches = 0;
  const result = line.replace(expression, (...args) => {
    matches += 1;
    return replace(...args);
  });
  assert.equal(matches, expected, `unexpected coordinate count in ${line}`);
  return result;
}

function normalizeCanonMapping(line, edits) {
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

function normalizeCanonLink(line, edits) {
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

function normalizeMapperMapping(line, edits) {
  const row = JSON.parse(line);
  assert(Array.isArray(row) && row.length === 6, `invalid content-mapper row: ${line}`);
  const [start, length] = historicalInterval(row[0], row[1], edits);
  row[0] = start;
  row[1] = length;
  return `[${row.join(", ")}]`;
}

function normalizeMapperLink(line, edits) {
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

function normalizeMapperHit(line, edits) {
  return replacePair(
    line,
    /^(.*?\|)(\d+):(\d+)>(\d+):(\d+)(\|.*)$/u,
    (_match, prefix, start, length, source, sourceLength, suffix) => {
      const [previousStart, previousLength] = historicalInterval(start, length, edits);
      return `${prefix}${previousStart}:${previousLength}>${source}:${sourceLength}${suffix}`;
    },
  );
}

function normalizeMaestroRow(line, edits, templateUri, authored) {
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

function reconcileCoordinateRows(path, beforeBytes, afterBytes, normalize) {
  const baseline = textRows(beforeBytes);
  const incident = textRows(afterBytes);
  const remaining = new Map();
  for (const row of baseline) remaining.set(row, (remaining.get(row) ?? 0) + 1);
  const matched = [];
  const added = [];
  const rejected = [];
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
  const missing = [];
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

function maestroDocumentProof(family, packets, editScript) {
  const manifests = {};
  for (const kind of ["baseline", "incident"]) {
    const path = `${family}/maestro/text.txt`;
    manifests[kind] = textRows(packets[kind].get(path));
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
  const normalized = [];
  let templateUri;
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
export function projectionCoordinateProof(manifest, packets) {
  const scripts = new Map(manifest.editScripts.map((script) => [script.id, script]));
  assert.equal(
    scripts.size,
    retainedFamilies.length * 3,
    "missing or duplicate projection edit script",
  );
  const surfaces = [];
  const documents = [];
  for (const family of retainedFamilies) {
    const get = (kind, path) => {
      const bytes = packets[kind].get(`${family}/${path}`);
      assert(bytes, `missing retained ${kind} path: ${family}/${path}`);
      return bytes;
    };
    const canon = scripts.get(`${family}/canon`);
    const mapper = scripts.get(`${family}/content-mapper`);
    const maestro = scripts.get(`${family}/maestro`);
    assert(canon && mapper && maestro);
    for (const [path, normalize] of [
      ["canon/mappings.txt", (row) => normalizeCanonMapping(row, canon.edits)],
      ["canon/semantic-links.txt", (row) => normalizeCanonLink(row, canon.edits)],
      ["content-mapper/mappings.txt", (row) => normalizeMapperMapping(row, mapper.edits)],
      ["content-mapper/semantic-links.txt", (row) => normalizeMapperLink(row, mapper.edits)],
      ["content-mapper/authored-hits.txt", (row) => normalizeMapperHit(row, mapper.edits)],
    ])
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
    ])
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

function walk(root, directory, output = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) =>
    a.name.localeCompare(b.name, "en"),
  )) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) walk(root, path, output);
    else {
      assert(entry.isFile(), `unsupported retained packet member: ${path}`);
      const bytes = readFileSync(path);
      const stat = statSync(path);
      output.push({
        path: relative(root, path).split(sep).join("/"),
        bytes: bytes.length,
        mode: stat.mode & 0o777,
        sha256: sha256(bytes),
      });
    }
  }
  return output;
}

function packet(root) {
  const files = walk(root, root).filter(({ path }) => path !== "receipts.json");
  files.sort((left, right) => left.path.localeCompare(right.path, "en"));
  return files;
}

function putBlob(blobs, bytes) {
  const digest = sha256(bytes);
  const encoded = bytes.toString("base64");
  const previous = blobs.get(digest);
  if (previous === undefined) blobs.set(digest, encoded);
  else assert.equal(previous, encoded, "SHA-256 collision in retained packet");
}

function probeBytes(original, virtual) {
  return Buffer.from(
    Array.from({ length: original + 1 }, (_value, index) => `o${index}>${index};`).join("") +
      Array.from({ length: virtual + 1 }, (_value, index) => `v${index}>${index};`).join(""),
  );
}

function representation(row, root, rows) {
  if (row.bytes === 0) return { representation: "empty" };
  if (row.path.endsWith("/canon/import-probes.txt")) {
    const prefix = row.path.slice(0, -"import-probes.txt".length);
    const original = rows.find(({ path }) => path === `${prefix}pre-rewrite-text.ts`);
    const virtual = rows.find(({ path }) => path === `${prefix}text.ts`);
    assert(original && virtual, `missing import-probe source for ${row.path}`);
    const derived = probeBytes(original.bytes, virtual.bytes);
    if (derived.length === row.bytes && sha256(derived) === row.sha256)
      return { representation: "derived-import-probes" };
  }
  if (row.path.endsWith("/canon/pre-rewrite-text.ts")) {
    const source = row.path.replace("pre-rewrite-text.ts", "text.ts");
    const sourceRow = rows.find((candidate) => candidate.path === source);
    assert(sourceRow, `missing canon text for ${row.path}`);
    if (row.sha256 === sourceRow.sha256) return { representation: "same-as", source };
  }
  const bytes = readFileSync(join(root, row.path));
  assert.equal(sha256(bytes), row.sha256);
  return { representation: "inline" };
}

/** Build the minimal content-addressed subset from two already verified artifacts. */
export function buildRetainedProjectionFixture({
  baselineRoot,
  incidentRoot,
  baselineArtifact,
  incidentArtifact,
  manifestPath,
  payloadPath,
}) {
  baselineRoot = resolve(baselineRoot);
  incidentRoot = resolve(incidentRoot);
  manifestPath = resolve(manifestPath);
  payloadPath = resolve(payloadPath);
  const packets = {
    baseline: { files: packet(baselineRoot) },
    incident: { files: packet(incidentRoot) },
  };
  const packetBytes = {
    baseline: new Map(
      packets.baseline.files.map((row) => [row.path, readFileSync(join(baselineRoot, row.path))]),
    ),
    incident: new Map(
      packets.incident.files.map((row) => [row.path, readFileSync(join(incidentRoot, row.path))]),
    ),
  };
  assert.deepEqual(
    packets.baseline.files.map((row) => row.path),
    packets.incident.files.map((row) => row.path),
    "retained packet membership changed",
  );
  const blobs = new Map();
  for (const [kind, root] of [
    ["baseline", baselineRoot],
    ["incident", incidentRoot],
  ]) {
    packets[kind].files = packets[kind].files.map((row) => ({
      ...row,
      ...representation(row, root, packets[kind].files),
    }));
    for (const row of packets[kind].files) {
      const bytes = readFileSync(join(root, row.path));
      assert.equal(bytes.length, row.bytes);
      assert.equal(sha256(bytes), row.sha256);
      if (row.representation === "inline") putBlob(blobs, bytes);
    }
  }
  const editScripts = [];
  for (const family of retainedFamilies) {
    for (const [surface, path] of [
      ["canon", `${family}/canon/text.ts`],
      ["content-mapper", `${family}/content-mapper/text.ts`],
      ["maestro", `${family}/maestro-document-0/text.ts`],
    ]) {
      const before = readFileSync(join(baselineRoot, path));
      const after = readFileSync(join(incidentRoot, path));
      editScripts.push({
        id: `${family}/${surface}`,
        family,
        surface,
        path,
        beforeBytes: before.length,
        beforeSha256: sha256(before),
        afterBytes: after.length,
        afterSha256: sha256(after),
        edits: diffGeneratedBytes(before, after),
      });
    }
  }
  const payload = {
    schema: 1,
    blobs: Object.fromEntries(
      [...blobs].sort(([left], [right]) => left.localeCompare(right, "en")),
    ),
  };
  const uncompressed = Buffer.from(`${JSON.stringify(payload)}\n`);
  const compressed = gzipSync(uncompressed, { level: 9, mtime: 0 });
  const manifest = {
    schema: 1,
    kind: "vize.template-emit-projection-history",
    families: retainedFamilies,
    artifacts: { baseline: baselineArtifact, incident: incidentArtifact },
    packets,
    editScripts,
    coordinateProof: projectionCoordinateProof({ editScripts }, packetBytes),
    payload: {
      file: basename(payloadPath),
      compression: "gzip",
      bytes: compressed.length,
      sha256: sha256(compressed),
      uncompressedBytes: uncompressed.length,
      uncompressedSha256: sha256(uncompressed),
      blobCount: blobs.size,
    },
  };
  mkdirSync(dirname(manifestPath), { recursive: true });
  writeFileSync(payloadPath, compressed);
  writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

/** Load and authenticate every retained byte before semantic qualification. */
export function loadRetainedProjectionFixture(manifestUrl = retainedManifestUrl) {
  const manifestBytes = readFileSync(manifestUrl);
  if (retainedManifestSha256 !== "REGENERATE")
    assert.equal(sha256(manifestBytes), retainedManifestSha256, "retained manifest custody drift");
  const manifest = JSON.parse(manifestBytes);
  assert.equal(manifest.schema, 1);
  assert.equal(manifest.kind, "vize.template-emit-projection-history");
  assert.deepEqual(manifest.families, retainedFamilies);
  const payloadUrl = new URL(manifest.payload.file, manifestUrl);
  const compressed = readFileSync(payloadUrl);
  assert.equal(compressed.length, manifest.payload.bytes);
  assert.equal(sha256(compressed), manifest.payload.sha256);
  assert.equal(manifest.payload.compression, "gzip");
  const uncompressed = gunzipSync(compressed);
  assert.equal(uncompressed.length, manifest.payload.uncompressedBytes);
  assert.equal(sha256(uncompressed), manifest.payload.uncompressedSha256);
  const payload = JSON.parse(uncompressed);
  assert.equal(payload.schema, 1);
  assert.equal(Object.keys(payload.blobs).length, manifest.payload.blobCount);
  const packets = {};
  for (const kind of ["baseline", "incident"]) {
    const files = new Map();
    const rows = new Map();
    let previous = "";
    for (const row of manifest.packets[kind].files) {
      assert(row.path > previous, `unsorted ${kind} packet`);
      previous = row.path;
      assert.equal(row.mode, 0o644);
      assert(!rows.has(row.path), `duplicate ${kind} packet path: ${row.path}`);
      rows.set(row.path, row);
      let bytes;
      if (row.representation === "inline") {
        const encoded = payload.blobs[row.sha256];
        assert.equal(typeof encoded, "string", `missing retained blob: ${row.sha256}`);
        bytes = Buffer.from(encoded, "base64");
      } else if (row.representation === "empty") bytes = Buffer.alloc(0);
      else if (!["same-as", "derived-import-probes"].includes(row.representation))
        assert.fail(`unknown retained representation: ${row.representation}`);
      if (bytes) files.set(row.path, bytes);
    }
    for (const row of rows.values()) {
      if (row.representation === "same-as") {
        assert.equal(typeof row.source, "string");
        const source = files.get(row.source);
        assert(source, `missing same-as source: ${row.source}`);
        files.set(row.path, source);
      }
    }
    for (const row of rows.values()) {
      if (row.representation === "derived-import-probes") {
        const prefix = row.path.slice(0, -"import-probes.txt".length);
        const original = files.get(`${prefix}pre-rewrite-text.ts`);
        const virtual = files.get(`${prefix}text.ts`);
        assert(original && virtual, `missing import-probe source: ${row.path}`);
        files.set(row.path, probeBytes(original.length, virtual.length));
      }
    }
    for (const row of rows.values()) {
      const bytes = files.get(row.path);
      assert(bytes, `unmaterialized retained path: ${row.path}`);
      assert.equal(bytes.length, row.bytes, row.path);
      assert.equal(sha256(bytes), row.sha256, row.path);
    }
    packets[kind] = files;
  }
  assert.deepEqual([...packets.baseline.keys()], [...packets.incident.keys()]);
  return { manifest, packets };
}

const frozenBaselineUrl = new URL(
  "../../../tests/_fixtures/differential/typechecker/template-dollar-emit/projection-baseline.json",
  import.meta.url,
);
const oldEmitDeclaration = "const $emit = __ctx.$emit;";
const retainedEmitDeclarations = {
  "dual-scripts-emits":
    "const $emit = undefined as unknown as __EmitFn<{ submit: [value: string] }>;",
  "define-model":
    'const $emit = undefined as unknown as __EmitFn<{\n"update:title": [value: string];\n"update:modelValue": [value: number];\n}>;',
};

function byteLength(text) {
  return Buffer.byteLength(text);
}

function digest(text, bytes = false) {
  return `${bytes ? byteLength(text) : text === "" ? 0 : text.split("\n").length}:${sha256(Buffer.from(text))}`;
}

function spanDigest(text) {
  let depth = 0;
  let count = text === "" ? 0 : 1;
  for (const character of text) {
    if (character === "[") depth += 1;
    if (character === "]") depth -= 1;
    assert(depth >= 0);
    if (character === "\n" && depth === 0) count += 1;
  }
  assert.equal(depth, 0);
  return `${count}:${sha256(Buffer.from(text))}`;
}

function rewindRetainedInitializer(text, declaration, allowAbsent = false) {
  const index = text.indexOf(declaration);
  if (allowAbsent && index === -1) {
    assert(!text.includes("const $emit = "));
    return { text, cut: Number.POSITIVE_INFINITY, end: Number.POSITIVE_INFINITY, delta: 0 };
  }
  assert(index >= 0, "the retained typed initializer is absent");
  assert.equal(text.indexOf(declaration, index + declaration.length), -1);
  const cut = byteLength(text.slice(0, index));
  return {
    text: text.slice(0, index) + oldEmitDeclaration + text.slice(index + declaration.length),
    cut,
    end: cut + byteLength(declaration),
    delta: byteLength(declaration) - byteLength(oldEmitDeclaration),
  };
}

function retainedInitializerOffset(value, edit) {
  const offset = Number(value);
  assert(Number.isSafeInteger(offset) && offset >= 0, "invalid retained generated offset");
  if (offset <= edit.cut) return offset;
  assert(offset >= edit.end, "a retained coordinate entered the typed initializer");
  return offset - edit.delta;
}

function retainedGeneratedPairs(text, edit) {
  return text.replace(
    /(\d+):(\d+)>(\d+):(\d+)/gu,
    (_match, start, end, source, sourceEnd) =>
      `${retainedInitializerOffset(start, edit)}:${retainedInitializerOffset(end, edit)}>${source}:${sourceEnd}`,
  );
}

function retainedTargetPairs(text, edit) {
  return text.replace(
    /(\d+):(\d+)>(\d+):(\d+)/gu,
    (_match, source, sourceEnd, start, end) =>
      `${source}:${sourceEnd}>${retainedInitializerOffset(start, edit)}:${retainedInitializerOffset(end, edit)}`,
  );
}

function retainedIntervals(text, edit) {
  return text.replace(
    /(\d+):(\d+)>(\d+):(\d+)/gu,
    (_match, start, length, source, sourceLength) => {
      const previous = retainedInitializerOffset(start, edit);
      return `${previous}:${retainedInitializerOffset(Number(start) + Number(length), edit) - previous}>${source}:${sourceLength}`;
    },
  );
}

/** Re-run the last successful receipt's frozen initializer and map proof. */
export function qualifyFrozenProjectionBaseline(packets) {
  const baselineBytes = readFileSync(frozenBaselineUrl);
  assert.equal(
    sha256(baselineBytes),
    "e7fcc4e2239ce80fb0081b8fd075aa7bcf52339de5787e439b821dcdbb61e1b9",
  );
  const baseline = JSON.parse(baselineBytes);
  const read = (family, path) => {
    const bytes = packets.baseline.get(`${family}/${path}`);
    assert(bytes, `missing retained baseline path: ${family}/${path}`);
    return bytes.toString("utf8");
  };
  const proofs = [];
  for (const [family, expected] of Object.entries(baseline)) {
    assert(retainedFamilies.includes(family));
    assert.equal(sha256(packets.baseline.get(`${family}/original.vue`)), expected["source-sha256"]);
    const declaration = retainedEmitDeclarations[family];
    assert(declaration);
    const canon = rewindRetainedInitializer(read(family, "canon/text.ts"), declaration);
    const preRewrite = rewindRetainedInitializer(
      read(family, "canon/pre-rewrite-text.ts"),
      declaration,
    );
    assert.equal(canon.delta, family === "dual-scripts-emits" ? 50 : 97);
    assert.equal(digest(canon.text, true), expected.canon.text);
    assert.equal(digest(preRewrite.text, true), expected.canon["pre-rewrite-text"]);
    assert.equal(
      spanDigest(retainedGeneratedPairs(read(family, "canon/mappings.txt"), canon)),
      expected.canon.mappings,
    );
    assert.equal(
      digest(
        read(family, "canon/semantic-links.txt").replace(
          /(\d+):(\d+)>(\d+):(\d+)/gu,
          (_match, start, end, target, targetEnd) =>
            `${retainedInitializerOffset(start, canon)}:${retainedInitializerOffset(end, canon)}>${retainedInitializerOffset(target, canon)}:${retainedInitializerOffset(targetEnd, canon)}`,
        ),
      ),
      expected.canon["semantic-links"],
    );
    assert.equal(
      `0:${sha256(Buffer.from(read(family, "canon/import-map.txt")))}`,
      expected.canon["import-source-map"],
    );
    const current = read(family, "canon/text.ts");
    const currentPreRewrite = read(family, "canon/pre-rewrite-text.ts");
    assert.equal(
      read(family, "canon/import-probes.txt"),
      probeBytes(byteLength(currentPreRewrite), byteLength(current)).toString("utf8"),
    );
    const previousProbes = probeBytes(byteLength(preRewrite.text), byteLength(canon.text));
    assert.equal(
      `${byteLength(preRewrite.text) + byteLength(canon.text) + 2}:${sha256(previousProbes)}`,
      expected.canon["import-source-map-probes"],
    );

    const mapper = rewindRetainedInitializer(read(family, "content-mapper/text.ts"), declaration);
    assert.equal(digest(mapper.text, true), expected["content-mapper"].text);
    const mapperMappings = read(family, "content-mapper/mappings.txt")
      .split("\n")
      .map((line) => {
        const row = JSON.parse(line);
        const [start, length, ...unchanged] = row;
        const previous = retainedInitializerOffset(start, mapper);
        return `[${[
          previous,
          retainedInitializerOffset(start + length, mapper) - previous,
          ...unchanged,
        ].join(", ")}]`;
      })
      .join("\n");
    assert.equal(digest(mapperMappings), expected["content-mapper"].mappings);
    const mapperLinks = read(family, "content-mapper/semantic-links.txt").replace(
      /(\d+):(\d+)>(\d+):(\d+):(\d+)/gu,
      (_match, source, sourceLength, target, length, kind) => {
        const previous = retainedInitializerOffset(target, mapper);
        return `${source}:${sourceLength}>${previous}:${retainedInitializerOffset(Number(target) + Number(length), mapper) - previous}:${kind}`;
      },
    );
    assert.equal(digest(mapperLinks), expected["content-mapper"]["semantic-links"]);
    assert.equal(
      digest(read(family, "content-mapper/diagnostics.txt")),
      expected["content-mapper"].diagnostics,
    );
    assert.equal(
      digest(retainedIntervals(read(family, "content-mapper/authored-hits.txt"), mapper)),
      expected["content-mapper"]["authored-hits"],
    );

    const documents = read(family, "maestro/text.txt").split("\n");
    const edits = new Map();
    let previousBytes = 0;
    const previousDocuments = documents.map((line, index) => {
      const [uri, language, size, documentSha] = line.split("|");
      const text = read(family, `maestro-document-${index}/text.ts`);
      assert.equal(`${byteLength(text)}:${sha256(Buffer.from(text))}`, `${size}:${documentSha}`);
      const edit = rewindRetainedInitializer(text, declaration, true);
      edits.set(uri, edit);
      previousBytes += byteLength(edit.text);
      return `${uri}|${language}|${byteLength(edit.text)}|${sha256(Buffer.from(edit.text))}`;
    });
    assert.equal([...edits.values()].filter((edit) => edit.delta > 0).length, 1);
    assert.equal(
      `${previousBytes}:${sha256(Buffer.from(previousDocuments.join("\n")))}`,
      expected.maestro.text,
    );
    const maestroRows = (file, authored) =>
      read(family, `maestro/${file}`)
        .split("\n")
        .map((line) => {
          const parts = line.split("|");
          const uri = parts[authored ? 1 : 0];
          const edit = edits.get(uri);
          assert(edit);
          parts[2] = retainedTargetPairs(parts[2], edit);
          return parts.join("|");
        })
        .join("\n");
    assert.equal(digest(maestroRows("mappings.txt", false)), expected.maestro.mappings);
    assert.equal(digest(maestroRows("authored-hits.txt", true)), expected.maestro["authored-hits"]);
    proofs.push({
      name: family,
      sourceSha256: expected["source-sha256"],
      initializerBytesAdded: canon.delta,
      restoredPriorTextAndEveryMapping: true,
    });
  }
  assert.deepEqual(proofs.map(({ name }) => name).sort(), [...retainedFamilies].sort());
  return proofs;
}

/**
 * Prove that the incident packet reverses to the successful artifact, that
 * stable map coordinates reverse with it, and that the successful packet still
 * reconstructs the older reviewed projection baseline.
 */
export function qualifyRetainedProjectionHistory(fixture = loadRetainedProjectionFixture()) {
  const { manifest, packets } = fixture;
  assert.deepEqual(manifest.artifacts, retainedArtifactCustody);
  assert.deepEqual(manifest.families, retainedFamilies);
  const expectedIds = retainedFamilies.flatMap((family) =>
    ["canon", "content-mapper", "maestro"].map((surface) => `${family}/${surface}`),
  );
  assert.deepEqual(
    manifest.editScripts.map(({ id }) => id),
    expectedIds,
  );
  const editProofs = [];
  for (const script of manifest.editScripts) {
    assert.equal(
      script.path,
      `${script.family}/${script.surface === "maestro" ? "maestro-document-0" : script.surface}/text.ts`,
    );
    const before = packets.baseline.get(script.path);
    const after = packets.incident.get(script.path);
    assert(before && after, `missing edit-script input: ${script.path}`);
    assert.equal(before.length, script.beforeBytes);
    assert.equal(after.length, script.afterBytes);
    assert.equal(sha256(before), script.beforeSha256);
    assert.equal(sha256(after), script.afterSha256);
    assert(script.edits.length > 0, `empty edit script: ${script.id}`);
    let previousBeforeEnd = 0;
    let previousAfterEnd = 0;
    for (const edit of script.edits) {
      assert(edit.beforeStart >= previousBeforeEnd && edit.afterStart >= previousAfterEnd);
      assert(edit.beforeEnd >= edit.beforeStart && edit.afterEnd >= edit.afterStart);
      assert(edit.beforeEnd <= before.length && edit.afterEnd <= after.length);
      assert(
        edit.beforeStart !== edit.beforeEnd || edit.afterStart !== edit.afterEnd,
        `empty edit in ${script.id}`,
      );
      const beforeSlice = before.subarray(edit.beforeStart, edit.beforeEnd);
      const afterSlice = after.subarray(edit.afterStart, edit.afterEnd);
      assert(!beforeSlice.equals(afterSlice), `no-op edit in ${script.id}`);
      if (beforeSlice.length > 0 && afterSlice.length > 0) {
        assert.notEqual(beforeSlice[0], afterSlice[0], `untrimmed edit prefix in ${script.id}`);
        assert.notEqual(
          beforeSlice[beforeSlice.length - 1],
          afterSlice[afterSlice.length - 1],
          `untrimmed edit suffix in ${script.id}`,
        );
      }
      previousBeforeEnd = edit.beforeEnd;
      previousAfterEnd = edit.afterEnd;
    }
    assert.deepEqual(applyInverseEdits(after, before, script.edits), before);
    editProofs.push({
      id: script.id,
      edits: script.edits.length,
      beforeBytes: before.length,
      afterBytes: after.length,
      restoredSha256: sha256(before),
    });
  }
  const coordinateProof = projectionCoordinateProof(manifest, packets);
  assert.deepEqual(coordinateProof, manifest.coordinateProof);
  const frozenBaseline = qualifyFrozenProjectionBaseline(packets);
  return {
    artifacts: manifest.artifacts,
    packetFiles: manifest.packets.baseline.files.length,
    editProofs,
    coordinateProof,
    frozenBaseline,
  };
}

function assertGeneratedRange(startValue, endValue, limit, label) {
  const start = Number(startValue);
  const end = Number(endValue);
  assert(Number.isSafeInteger(start) && Number.isSafeInteger(end), `${label}: non-integer range`);
  assert(start >= 0 && end >= start && end <= limit, `${label}: out-of-bounds range`);
}

function assertGeneratedInterval(startValue, lengthValue, limit, label) {
  const start = Number(startValue);
  const length = Number(lengthValue);
  assert(Number.isSafeInteger(length) && length >= 0, `${label}: invalid interval length`);
  assertGeneratedRange(start, start + length, limit, label);
}

function assertSafeEmitSentinel(text, label) {
  const sentinel = "const $emit = (void 0) as unknown as __EmitFn<";
  assert.equal(text.split(sentinel).length, 2, `${label}: missing or duplicate safe emit sentinel`);
  assert(
    !text.includes("const $emit = undefined"),
    `${label}: shadowable undefined sentinel returned`,
  );
}

/** Validate current raw output without comparing it to a stale generated digest. */
export function qualifyLiveProjectionCapture(
  rootValue,
  retained = loadRetainedProjectionFixture(),
) {
  const root = resolve(rootValue);
  const read = (family, path) => readFileSync(join(root, family, path));
  const proofs = [];
  for (const family of retainedFamilies) {
    const retainedSource = retained.packets.baseline.get(`${family}/original.vue`);
    const source = read(family, "original.vue");
    assert.deepEqual(source, retainedSource, `${family}: authored fixture drifted`);
    const canon = read(family, "canon/text.ts");
    const preRewrite = read(family, "canon/pre-rewrite-text.ts");
    assert.deepEqual(preRewrite, canon, `${family}: canon pre-rewrite text drifted`);
    assert.deepEqual(
      read(family, "canon/import-probes.txt"),
      probeBytes(preRewrite.length, canon.length),
      `${family}: import-probe coordinates drifted`,
    );
    assertSafeEmitSentinel(canon.toString("utf8"), `${family}/canon`);
    let canonCoordinates = 0;
    for (const row of textRows(read(family, "canon/mappings.txt"))) {
      let matches = 0;
      row.replace(/(\d+):(\d+)>(\d+):(\d+)/gu, (_match, start, end, sourceStart, sourceEnd) => {
        assertGeneratedRange(start, end, canon.length, `${family}/canon mapping`);
        assertGeneratedRange(sourceStart, sourceEnd, source.length, `${family}/canon source`);
        matches += 1;
        canonCoordinates += 1;
        return _match;
      });
      assert(matches > 0, `${family}: malformed canon mapping`);
    }
    for (const row of textRows(read(family, "canon/semantic-links.txt"))) {
      replacePair(
        row,
        /^(.*?:)(\d+):(\d+)>(\d+):(\d+)$/u,
        (_match, _kind, start, end, target, targetEnd) => {
          assertGeneratedRange(start, end, canon.length, `${family}/canon link source`);
          assertGeneratedRange(target, targetEnd, canon.length, `${family}/canon link target`);
          canonCoordinates += 2;
          return _match;
        },
      );
    }

    const mapper = read(family, "content-mapper/text.ts");
    assertSafeEmitSentinel(mapper.toString("utf8"), `${family}/content-mapper`);
    let mapperCoordinates = 0;
    for (const row of textRows(read(family, "content-mapper/mappings.txt"))) {
      const values = JSON.parse(row);
      assert(Array.isArray(values) && values.length === 6, `${family}: malformed mapper mapping`);
      assertGeneratedInterval(values[0], values[1], mapper.length, `${family}/mapper mapping`);
      assertGeneratedInterval(values[2], values[3], source.length, `${family}/mapper source`);
      mapperCoordinates += 1;
    }
    for (const row of textRows(read(family, "content-mapper/semantic-links.txt"))) {
      replacePair(
        row,
        /^(\d+):(\d+)>(\d+):(\d+):([^:]+)$/u,
        (_match, start, length, target, targetLength) => {
          assertGeneratedInterval(start, length, mapper.length, `${family}/mapper link source`);
          assertGeneratedInterval(
            target,
            targetLength,
            mapper.length,
            `${family}/mapper link target`,
          );
          mapperCoordinates += 2;
          return _match;
        },
      );
    }
    for (const row of textRows(read(family, "content-mapper/authored-hits.txt"))) {
      replacePair(
        row,
        /^(.*?\|)(\d+):(\d+)>(\d+):(\d+)(\|.*)$/u,
        (_match, _prefix, start, length, sourceStart, sourceLength) => {
          assertGeneratedInterval(start, length, mapper.length, `${family}/mapper hit`);
          assertGeneratedInterval(
            sourceStart,
            sourceLength,
            source.length,
            `${family}/mapper hit source`,
          );
          mapperCoordinates += 1;
          return _match;
        },
      );
    }
    assert.equal(read(family, "content-mapper/diagnostics.txt").length, 0);

    const documents = new Map();
    const documentRows = textRows(read(family, "maestro/text.txt"));
    for (const [index, row] of documentRows.entries()) {
      const [uri, language, bytes, digest] = row.split("|");
      const text = read(family, `maestro-document-${index}/text.ts`);
      assert.equal(text.length, Number(bytes), `${family}: maestro size drift`);
      assert.equal(sha256(text), digest, `${family}: maestro digest drift`);
      assert(!documents.has(uri), `${family}: duplicate maestro URI`);
      documents.set(uri, { language, text });
      if (language === "Template")
        assertSafeEmitSentinel(text.toString("utf8"), `${family}/maestro`);
    }
    assert.equal(
      [...documents.values()].filter(({ language }) => language === "Template").length,
      1,
      `${family}: missing or duplicate maestro template`,
    );
    let maestroCoordinates = 0;
    for (const [file, authored] of [
      ["mappings.txt", false],
      ["authored-hits.txt", true],
    ]) {
      for (const row of textRows(read(family, `maestro/${file}`))) {
        const parts = row.split("|");
        assert(parts.length >= 4, `${family}: malformed maestro ${file}`);
        const document = documents.get(parts[authored ? 1 : 0]);
        assert(document, `${family}: maestro coordinate references an unknown document`);
        replacePair(
          parts[2],
          /^(\d+):(\d+)>(\d+):(\d+)$/u,
          (_match, sourceStart, sourceEnd, start, end) => {
            assertGeneratedRange(sourceStart, sourceEnd, source.length, `${family}/maestro source`);
            assertGeneratedRange(start, end, document.text.length, `${family}/maestro target`);
            maestroCoordinates += 1;
            return _match;
          },
        );
      }
    }
    assert(canonCoordinates > 0 && mapperCoordinates > 0 && maestroCoordinates > 0);
    proofs.push({
      family,
      sourceSha256: sha256(source),
      canonSha256: sha256(canon),
      mapperSha256: sha256(mapper),
      documents: documents.size,
      coordinates: {
        canon: canonCoordinates,
        mapper: mapperCoordinates,
        maestro: maestroCoordinates,
      },
      exactCurrentSnapshotQualifiedBeforeReceipt: true,
    });
  }
  return proofs;
}
