import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { asArray, asRecord, asString, parseJson, type JsonRecord } from "../../json-value.ts";
import { type Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import { githubSnapshot } from "../release/quota/http.ts";
import { readReleaseOwnership } from "../release/quota_ownership.ts";
import { gitBlob, validateCapsule } from "../constructor_original27_public/authority.ts";

export const originalSource = "274777b2b18a8e43280a109cfbc4bb2a7286ee00";
const originalPaths = [
  "tests/_fixtures/differential/formatter-regressions/semicolon-cli-6098/cases.json",
  "crates/vize/tests/fmt_semicolon_cli.rs",
  "tests/tooling/formatter-semicolon-reference.test.ts",
  "crates/vize/tests/support/vite_config.rs",
  "tests/_fixtures/differential/lsp/editor-import-sorting-6098/App.vue.txt",
  "tests/_fixtures/differential/lsp/editor-import-sorting-6098/vite.config.mjs.txt",
  "tests/_fixtures/differential/lsp/editor-import-sorting-6098/case.json",
  "tests/_fixtures/differential/lsp/editor-import-sorting-6098/response.expected.json",
  "crates/vize_glyph/tests/fixtures/sort-imports/UserCard.sorted.vue",
  "crates/vize/tests/fmt_import_sorting_cli.rs",
  "crates/vize_maestro/src/server/handlers/tests/formatting/import_sorting.rs",
  "crates/vize_maestro/src/server/handlers/tests/formatting.rs",
  "tests/_fixtures/differential/lsp/art-range-formatting-6098/App.vue.txt",
  "tests/_fixtures/differential/lsp/art-range-formatting-6098/vite.config.mjs.txt",
  "tests/_fixtures/differential/lsp/art-range-formatting-6098/case.json",
  "tests/_fixtures/differential/lsp/art-range-formatting-6098/response.expected.json",
  "crates/vize_maestro/src/server/format/custom_blocks.rs",
  "crates/vize_carton/src/config/loader/js_cache.rs",
  "crates/vize_carton/src/config/loader/js.rs",
] as const;
export type OriginalReference = {
  path: string;
  gitBlob: string;
  bytes: number;
  sha256: string;
  rawBase64: string;
};
export type OriginalContract = {
  source: string;
  references: OriginalReference[];
  raw: JsonRecord;
};

export function validateOriginal(value: unknown): OriginalContract {
  const raw = asRecord(value);
  assert.equal(raw.schema, "vize.formatter-profiles-public-original.v1");
  assert.equal(raw.source, originalSource);
  assert.deepEqual(raw.containingFixes, {
    semi: originalSource,
    sort: "b51ac8f46e6e361c6396a1976a9762514c96aa33",
    art: "9c594186bae97cbc4b4c5441d9165b3354ccbec0",
  });
  const references = asArray(raw.references).map((value) => {
    const row = asRecord(value);
    const path = asString(row.path),
      encoded = asString(row.rawBase64);
    const bytes = Buffer.from(encoded, "base64");
    assert.equal(bytes.toString("base64"), encoded);
    assert.equal(bytes.length, row.bytes);
    assert.equal(sha256(bytes), row.sha256, path);
    assert.equal(gitBlob(bytes), row.gitBlob, path);
    return {
      path,
      rawBase64: encoded,
      bytes: bytes.length,
      sha256: asString(row.sha256),
      gitBlob: asString(row.gitBlob),
    };
  });
  assert.deepEqual(
    references.map((row) => row.path),
    originalPaths,
  );
  return { source: originalSource, references, raw };
}

export function originalContract(): OriginalContract {
  return validateOriginal(
    parseJson(
      readFileSync(
        new URL(
          "../../../../tests/tooling/fixtures/formatter-profiles-public-original.json",
          import.meta.url,
        ),
        "utf8",
      ),
    ),
  );
}

export function originalBytes(contract: OriginalContract, path: string): Buffer {
  const rows = contract.references.filter((row) => row.path === path);
  assert.equal(rows.length, 1, "one immutable original input: " + path);
  const row = rows[0]!;
  const bytes = Buffer.from(row.rawBase64, "base64");
  assert.equal(sha256(bytes), row.sha256);
  assert.equal(gitBlob(bytes), row.gitBlob);
  return bytes;
}

export async function authenticate(
  version: string,
  run: string,
  blob: string,
  original: OriginalContract,
  record: Recorder,
) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.env.GITHUB_REF, "refs/heads/main");
  assert.match(version, /^\d+\.\d+\.\d+$/u);
  assert.match(run, /^[1-9]\d*$/u);
  assert.match(blob, /^[0-9a-f]{40}$/u);
  const observations: unknown[] = [];
  const read = async (endpoint: string) => {
    const snapshot = await githubSnapshot(endpoint);
    observations.push({ endpoint, ...snapshot });
    record.json("github-observations.json", observations);
    return snapshot.value;
  };
  const encoded = asRecord(await read("git/blobs/" + blob));
  assert.equal(encoded.encoding, "base64");
  const raw = Buffer.from(asString(encoded.content), "base64");
  assert.equal(gitBlob(raw), blob);
  assert.equal(encoded.sha, blob);
  writeFileSync(join(record.root, "authority-capsule.json"), raw, { flag: "wx" });
  const capsule = asRecord(parseJson(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
  const tag = asRecord(await read("git/ref/tags/v" + version));
  assert.equal(tag.ref, "refs/tags/v" + version);
  const target = asRecord(tag.object);
  assert.equal(target.type, "commit");
  const candidate = asString(target.sha);
  const publication = asRecord(await read("releases/tags/v" + version));
  assert.equal(publication.tag_name, "v" + version);
  assert.equal(publication.draft, false);
  assert.equal(publication.prerelease, false);
  assert(Number.isFinite(Date.parse(asString(publication.published_at))));
  const producer = asRecord(await read("actions/runs/" + run));
  assert.equal(producer.status, "completed");
  assert.equal(producer.conclusion, "success");
  const ownership = await readReleaseOwnership(version, candidate, run, read);
  validateCapsule(capsule, version, candidate, ownership.producerSource);
  const ancestry = asRecord(
    await read("compare/" + original.source + "..." + ownership.producerSource),
  );
  assert.equal(asRecord(ancestry.base_commit).sha, original.source);
  assert.equal(asRecord(ancestry.merge_base_commit).sha, original.source);
  assert.equal(ancestry.behind_by, 0);
  assert(["ahead", "identical"].includes(asString(ancestry.status)));
  for (const reference of original.references) {
    const source = asRecord(await read("contents/" + reference.path + "?ref=" + original.source));
    assert.equal(source.type, "file");
    assert.equal(source.path, reference.path);
    assert.equal(source.encoding, "base64");
    const bytes = Buffer.from(asString(source.content), "base64");
    assert.equal(source.sha, reference.gitBlob);
    assert.equal(gitBlob(bytes), reference.gitBlob);
    assert.deepEqual(bytes, originalBytes(original, reference.path));
  }
  record.json("original-source-contract.json", original.raw);
  record.json("release-authority.json", {
    version,
    run,
    blob,
    capsuleSha256: sha256(raw),
    candidate,
    ownership,
    originalSource: original.source,
    controllerSource: process.env.GITHUB_SHA,
    consumerRun: process.env.GITHUB_RUN_ID,
    consumerAttempt: process.env.GITHUB_RUN_ATTEMPT,
  });
  return { capsule, candidate, ownership, version, run };
}
export type Authority = Awaited<ReturnType<typeof authenticate>>;
