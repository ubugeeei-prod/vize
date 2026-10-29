import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { probeBytes } from "./typechecker-template-emit-projection-history-fixture.ts";
import type {
  FrozenProjectionBaseline,
  FrozenProjectionFamily,
  ProjectionFamily,
  RetainedInitializerEdit,
  RetainedProjectionPackets,
} from "./typechecker-template-emit-projection-history-model.ts";
import { retainedFamilies, sha256 } from "./typechecker-template-emit-projection-history-shared.ts";

const frozenBaselineUrl = new URL(
  "../../../tests/_fixtures/differential/typechecker/template-dollar-emit/projection-baseline.json",
  import.meta.url,
);
const oldEmitDeclaration = "const $emit = __ctx.$emit;";
const retainedEmitDeclarations: Record<ProjectionFamily, string> = {
  "dual-scripts-emits":
    "const $emit = undefined as unknown as __EmitFn<{ submit: [value: string] }>;",
  "define-model":
    'const $emit = undefined as unknown as __EmitFn<{\n"update:title": [value: string];\n"update:modelValue": [value: number];\n}>;',
};

interface FrozenBaselineProof {
  name: ProjectionFamily;
  sourceSha256: string;
  initializerBytesAdded: number;
  restoredPriorTextAndEveryMapping: true;
}

function byteLength(text: string): number {
  return Buffer.byteLength(text);
}

function digest(text: string, bytes = false): string {
  return `${bytes ? byteLength(text) : text === "" ? 0 : text.split("\n").length}:${sha256(Buffer.from(text))}`;
}

function spanDigest(text: string): string {
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

function rewindRetainedInitializer(
  text: string,
  declaration: string,
  allowAbsent = false,
): RetainedInitializerEdit {
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

function retainedInitializerOffset(value: number | string, edit: RetainedInitializerEdit): number {
  const offset = Number(value);
  assert(Number.isSafeInteger(offset) && offset >= 0, "invalid retained generated offset");
  if (offset <= edit.cut) return offset;
  assert(offset >= edit.end, "a retained coordinate entered the typed initializer");
  return offset - edit.delta;
}

function retainedGeneratedPairs(text: string, edit: RetainedInitializerEdit): string {
  return text.replace(
    /(\d+):(\d+)>(\d+):(\d+)/gu,
    (_match, start, end, source, sourceEnd) =>
      `${retainedInitializerOffset(start, edit)}:${retainedInitializerOffset(end, edit)}>${source}:${sourceEnd}`,
  );
}

function retainedTargetPairs(text: string, edit: RetainedInitializerEdit): string {
  return text.replace(
    /(\d+):(\d+)>(\d+):(\d+)/gu,
    (_match, source, sourceEnd, start, end) =>
      `${source}:${sourceEnd}>${retainedInitializerOffset(start, edit)}:${retainedInitializerOffset(end, edit)}`,
  );
}

function retainedIntervals(text: string, edit: RetainedInitializerEdit): string {
  return text.replace(
    /(\d+):(\d+)>(\d+):(\d+)/gu,
    (_match, start, length, source, sourceLength) => {
      const previous = retainedInitializerOffset(start, edit);
      return `${previous}:${retainedInitializerOffset(Number(start) + Number(length), edit) - previous}>${source}:${sourceLength}`;
    },
  );
}

/** Re-run the last successful receipt's frozen initializer and map proof. */
export function qualifyFrozenProjectionBaseline(
  packets: RetainedProjectionPackets,
): FrozenBaselineProof[] {
  const baselineBytes = readFileSync(frozenBaselineUrl);
  assert.equal(
    sha256(baselineBytes),
    "e7fcc4e2239ce80fb0081b8fd075aa7bcf52339de5787e439b821dcdbb61e1b9",
  );
  const baseline = JSON.parse(baselineBytes.toString("utf8")) as FrozenProjectionBaseline;
  const read = (family: ProjectionFamily, path: string): string => {
    const bytes = packets.baseline.get(`${family}/${path}`);
    assert(bytes, `missing retained baseline path: ${family}/${path}`);
    return bytes.toString("utf8");
  };
  const proofs: FrozenBaselineProof[] = [];
  const families = Object.entries(baseline) as [ProjectionFamily, FrozenProjectionFamily][];
  for (const [family, expected] of families) {
    assert(retainedFamilies.includes(family));
    const original = packets.baseline.get(`${family}/original.vue`);
    assert(original, `missing retained source: ${family}`);
    assert.equal(sha256(original), expected["source-sha256"]);
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
        const row = JSON.parse(line) as number[];
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
    const edits = new Map<string, RetainedInitializerEdit>();
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
    const maestroRows = (file: string, authored: boolean): string =>
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
