import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import {
  replacePair,
  textRows,
} from "./typechecker-template-emit-projection-history-coordinates.ts";
import {
  loadRetainedProjectionFixture,
  probeBytes,
} from "./typechecker-template-emit-projection-history-fixture.ts";
import type {
  ProjectionFamily,
  RetainedProjectionFixture,
} from "./typechecker-template-emit-projection-history-model.ts";
import { retainedFamilies, sha256 } from "./typechecker-template-emit-projection-history-shared.ts";

interface LiveProjectionProof {
  family: ProjectionFamily;
  sourceSha256: string;
  canonSha256: string;
  mapperSha256: string;
  documents: number;
  coordinates: { canon: number; mapper: number; maestro: number };
  exactCurrentSnapshotQualifiedBeforeReceipt: true;
}

interface LiveMaestroDocument {
  language: string;
  text: Buffer;
}

function assertGeneratedRange(
  startValue: number | string,
  endValue: number | string,
  limit: number,
  label: string,
): void {
  const start = Number(startValue);
  const end = Number(endValue);
  assert(Number.isSafeInteger(start) && Number.isSafeInteger(end), `${label}: non-integer range`);
  assert(start >= 0 && end >= start && end <= limit, `${label}: out-of-bounds range`);
}

function assertGeneratedInterval(
  startValue: number | string,
  lengthValue: number | string,
  limit: number,
  label: string,
): void {
  const start = Number(startValue);
  const length = Number(lengthValue);
  assert(Number.isSafeInteger(length) && length >= 0, `${label}: invalid interval length`);
  assertGeneratedRange(start, start + length, limit, label);
}

function assertSafeEmitSentinel(text: string, label: string): void {
  const sentinel = "const $emit = (void 0) as unknown as __EmitFn<";
  assert.equal(text.split(sentinel).length, 2, `${label}: missing or duplicate safe emit sentinel`);
  assert(
    !text.includes("const $emit = undefined"),
    `${label}: shadowable undefined sentinel returned`,
  );
}

/** Validate current raw output without comparing it to a stale generated digest. */
export function qualifyLiveProjectionCapture(
  rootValue: string,
  retained: RetainedProjectionFixture = loadRetainedProjectionFixture(),
): LiveProjectionProof[] {
  const root = resolve(rootValue);
  const read = (family: ProjectionFamily, path: string): Buffer =>
    readFileSync(join(root, family, path));
  const proofs: LiveProjectionProof[] = [];
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
      const values = JSON.parse(row) as number[];
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

    const documents = new Map<string, LiveMaestroDocument>();
    const documentRows = textRows(read(family, "maestro/text.txt"));
    for (const [index, row] of documentRows.entries()) {
      const [uri, language, bytes, digest] = row.split("|");
      assert(uri && language && bytes && digest, `${family}: malformed maestro document`);
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
    ] as const) {
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
