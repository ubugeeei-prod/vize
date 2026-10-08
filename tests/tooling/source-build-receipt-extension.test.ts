import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { sha256 } from "../differential/sha256.ts";
import {
  qualifyBuildReceiptExtension,
  receiptAssertionDigests,
  receiptCallerPaths,
  receiptCompanionAppendix,
  receiptQualificationPaths,
  receiptWitnessDigests,
  type ReceiptExtensionReader,
  type ReceiptExtensionSide,
} from "../performance/support/warm-type-backed-build-receipt-extension.ts";

interface PublishedBody {
  mode: string;
  gitBlobSha1: string;
  bytes: number;
  sha256: string;
  base64: string;
}
interface PublishedEnvelope {
  schema: string;
  version: number;
  before: string;
  reviewed: string;
  files: { file: string; before: PublishedBody | null; after: PublishedBody | null }[];
}
interface Provenance {
  completeCallers: number;
  completeCommandAssertions: number;
  completePaths: number;
  sourceBodyBytes: number;
  nullBodies: number;
  envelope: {
    path: string;
    bytes: number;
    sha256: string;
    uncompressedBytes: number;
    uncompressedSha256: string;
  };
}

const root = fileURLToPath(new URL("../../", import.meta.url));
const witnessRoot = "tests/_fixtures/tooling/source-build-receipt-extension";
for (const [file, expected] of Object.entries(receiptWitnessDigests))
  assert.equal(sha256(fs.readFileSync(path.join(root, file))), expected, file);
// These complete actual published envelopes are checksum-bound before parsing.
const provenance: Provenance = JSON.parse(
  fs.readFileSync(path.join(root, witnessRoot, "provenance.json"), "utf8"),
);
const compressed = fs.readFileSync(path.join(root, witnessRoot, provenance.envelope.path));
assert.equal(compressed.length, provenance.envelope.bytes);
assert.equal(sha256(compressed), provenance.envelope.sha256);
const uncompressed = gunzipSync(compressed);
assert.equal(uncompressed.length, provenance.envelope.uncompressedBytes);
assert.equal(sha256(uncompressed), provenance.envelope.uncompressedSha256);
const evidence: PublishedEnvelope = JSON.parse(uncompressed.toString("utf8"));
const published = new Map<string, { before: Buffer | null; after: Buffer | null }>();
let sourceBytes = 0;
let nullBodies = 0;
for (const entry of evidence.files) {
  assert.ok(!published.has(entry.file), "every actual published path must remain unique");
  const decode = (body: PublishedBody | null) => {
    if (body === null) {
      nullBodies++;
      return null;
    }
    assert.match(body.mode, /^100(?:644|755)$/u);
    const bytes = Buffer.from(body.base64, "base64");
    assert.equal(bytes.toString("base64"), body.base64);
    assert.equal(bytes.length, body.bytes);
    assert.equal(sha256(bytes), body.sha256);
    assert.equal(
      createHash("sha1")
        .update(Buffer.from(`blob ${bytes.length}\0`))
        .update(bytes)
        .digest("hex"),
      body.gitBlobSha1,
      "complete bytes must retain the actual Git blob identity",
    );
    sourceBytes += bytes.length;
    return bytes;
  };
  published.set(entry.file, { before: decode(entry.before), after: decode(entry.after) });
}
assert.equal(evidence.schema, "vize.source-build-receipt-extension.published-bodies");
assert.equal(evidence.version, 1);
assert.equal(evidence.before, "b6d2e15e046f7f51e9cf0f78e19e8f726e99a086");
assert.equal(evidence.reviewed, "78940693d715f3b2961a02e373f13dcefa06d38c");
assert.equal(published.size, 83);
assert.equal(published.size, provenance.completePaths);
assert.equal(sourceBytes, provenance.sourceBodyBytes);
assert.equal(nullBodies, provenance.nullBodies);
assert.equal(provenance.completeCallers, 73);
assert.equal(provenance.completeCommandAssertions, 3);
const project = "tsconfig.source-build-receipt.json";
const companion = "docs/davinci/decisions/2026-10-08-native-typescript-build-receipts.md";
const canonical = "docs/davinci/decisions/2026-09-27-level-restructure.md";
const alreadyQualified = new Set([canonical]);
const scoped = JSON.parse(published.get(project)!.after!.toString("utf8")) as {
  files: string[];
};
assert.deepEqual(scoped, {
  extends: "./tsconfig.node.json",
  compilerOptions: { composite: false, incremental: false },
  include: [],
  files: ["tests/differential/build-receipt.ts", "tests/differential/sha256.ts"],
});
const currentScoped = { ...scoped, files: [...scoped.files, ...receiptQualificationPaths] };
assert.deepEqual(JSON.parse(fs.readFileSync(path.join(root, project), "utf8")), currentScoped);
const changed = [
  ...published.keys(),
  ...receiptQualificationPaths,
  ...Object.keys(receiptWitnessDigests),
  canonical,
];
const read: ReceiptExtensionReader = (side, file) => {
  if (side === "after") {
    if (file === project || file === companion || receiptQualificationPaths.some((p) => p === file))
      return fs.readFileSync(path.join(root, file));
    if (Object.hasOwn(receiptWitnessDigests, file)) return fs.readFileSync(path.join(root, file));
    if (Object.hasOwn(receiptAssertionDigests, file))
      return published
        .get(file)!
        .after!.toString("utf8")
        .replaceAll("build-receipt\\.mjs", "build-receipt\\.ts");
  }
  return published.get(file)?.[side] ?? null;
};
const qualify = (reader: ReceiptExtensionReader = read, files: readonly string[] = changed) =>
  qualifyBuildReceiptExtension(files, alreadyQualified, reader);
function altered(
  side: ReceiptExtensionSide,
  file: string,
  mutate: (value: Buffer | string | null) => Buffer | string | null,
): ReceiptExtensionReader {
  return (candidate, name) =>
    candidate === side && name === file ? mutate(read(candidate, name)) : read(candidate, name);
}
const append = (value: Buffer | string | null) =>
  Buffer.from(String(value) + "\n// foreign bytes\n");

test("complete actual published source envelopes qualify only the finite reviewed closure", () => {
  const result = qualify();
  assert.ok(result);
  assert.equal(result.completeCallers, 73);
  assert.equal(result.originalSource, evidence.before);
  assert.equal(result.reviewedSource, evidence.reviewed);
  assert.equal(result.qualifiedFinitePaths.length, 87);
  assert.ok(
    !result.qualifiedFinitePaths.includes(canonical),
    "the original classifier owns canonical",
  );
  assert.ok(
    qualify((side, file) => {
      const value = read(side, file);
      return Buffer.isBuffer(value) && !file.endsWith(".gz") ? value.toString("utf8") : value;
    }),
  );
});

test("omitted, extra and duplicate callers never receive partial qualification", () => {
  for (const file of receiptCallerPaths)
    assert.equal(
      qualify(
        read,
        changed.filter((name) => name !== file),
      ),
      null,
      file,
    );
  for (const file of [
    "tests/tooling/unreviewed.test.ts",
    "tests/_fixtures/differential/lsp/warm-type-backed-requests/inputs.json",
  ])
    assert.equal(qualify(read, [...changed, file]), null, file);
  assert.equal(qualify(read, [...changed, changed[0]]), null);
  assert.equal(qualify(read, [canonical]), null);
});

test("the existing workflow path needs its original classifier qualification", () => {
  const workflow = "tests/performance/support/warm-type-backed-workflow.ts";
  assert.equal(qualify(read, [...changed, workflow]), null);
  const oldQualified = new Set([...alreadyQualified, workflow]);
  assert.ok(qualifyBuildReceiptExtension([...changed, workflow], oldQualified, read));
  assert.equal(
    qualifyBuildReceiptExtension([...changed, "tests/tooling/unreviewed.ts"], oldQualified, read),
    null,
  );
});

test("every caller keeps all non-token bytes on both sides", () => {
  for (const file of receiptCallerPaths)
    for (const side of ["before", "after"] as const) {
      assert.equal(qualify(altered(side, file, append)), null, `${side}:${file}`);
      assert.equal(qualify(altered(side, file, () => null)), null, `${side}:${file}:absent`);
    }
  assert.equal(qualify(altered("after", receiptCallerPaths[0], () => Buffer.from([0xff]))), null);
});

test("provider, primitive and true move absence must match whole authenticated source", () => {
  for (const [side, file] of [
    ["before", "tests/differential/build-receipt.mjs"],
    ["after", "tests/differential/build-receipt.ts"],
    ["after", "tests/differential/sha256.ts"],
  ] as const)
    assert.equal(qualify(altered(side, file, append)), null, file);
  assert.equal(
    qualify(altered("after", "tests/differential/build-receipt.mjs", () => Buffer.from([0xff]))),
    null,
  );
  assert.equal(
    qualify(
      altered("after", "tests/differential/build-receipt.ts", (value) =>
        String(value).replace("cargo build --profile ci -p vize", "cargo build --release"),
      ),
    ),
    null,
  );
});

test("harness extraction and every original check command remain whole", () => {
  for (const file of ["tests/differential/harness.mjs", "tools/config/vite-plus/tasks/check.ts"])
    for (const side of ["before", "after"] as const)
      assert.equal(qualify(altered(side, file, append)), null, `${side}:${file}`);
  assert.equal(
    qualify(
      altered("after", "tools/config/vite-plus/tasks/check.ts", (value) =>
        String(value).replace("tsconfig.source-build-receipt.json", "tsconfig.node.json"),
      ),
    ),
    null,
  );
});

test("three command assertions permit only their literal escaped token correction", () => {
  for (const file of Object.keys(receiptAssertionDigests)) {
    assert.ok(published.get(file)!.before!.equals(published.get(file)!.after!));
    assert.equal(qualify(altered("after", file, append)), null, file);
    assert.equal(
      qualify(
        read,
        changed.filter((name) => name !== file),
      ),
      null,
      file,
    );
    assert.equal(qualify(altered("after", file, () => published.get(file)!.after)), null, file);
  }
});

test("strict four-root project rejects recipe, foundation and scope drift", () => {
  for (const [from, to] of [
    ['"composite": false', '"composite": true'],
    ["./tsconfig.node.json", "./tsconfig.loose.json"],
    ["tests/differential/build-receipt.ts", "tests/differential/unreviewed.ts"],
    ["tests/tooling/source-build-receipt-extension.test.ts", "tests/tooling/unknown.test.ts"],
  ])
    assert.equal(
      qualify(altered("after", project, (value) => String(value).replace(from, to))),
      null,
    );
  assert.equal(qualify(altered("after", project, () => "{")), null);
});

test("new scope and whole historical witnesses are finite and complete", () => {
  for (const file of [...receiptQualificationPaths, ...Object.keys(receiptWitnessDigests)]) {
    assert.equal(
      qualify(
        read,
        changed.filter((name) => name !== file),
      ),
      null,
      file,
    );
    assert.equal(qualify(altered("before", file, () => Buffer.from([0xff]))), null, file);
    assert.equal(qualify(altered("after", file, () => null)), null, file);
  }
  for (const file of Object.keys(receiptWitnessDigests))
    assert.equal(qualify(altered("after", file, append)), null, file);
  assert.equal(qualify(altered("after", companion, append)), null);
  assert.ok(qualify(altered("after", companion, () => published.get(companion)!.after)));
  assert.ok(
    qualify(
      altered(
        "after",
        companion,
        () => published.get(companion)!.after!.toString("utf8") + receiptCompanionAppendix,
      ),
    ),
  );
});
