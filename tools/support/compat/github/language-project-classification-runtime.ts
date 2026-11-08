import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import { historicalCurrentLanguageProjectSources } from "./language-project-current-20261105.ts";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const CLASSIFICATION_RUNTIME_ROOT =
  "tests/_fixtures/tooling/typescript-classification-runtime-11664/";
export const CLASSIFICATION_RUNTIME_ID =
  "typechecker/language-project/classification-runtime-11664";
const custodyHash = "0032136c61a94f26c192d61799a0f4ae8137d405e0071a39847b05ff71c7d7ab";
const predecessorPath =
  "tests/_fixtures/tooling/typescript-classification-successor-11664/custody.json";
const predecessorHash = "04628e3b129e38c7ee0cc721848a428fa34518993a4646ea7bef806827c68005";
const transitions = [
  ["tests/typescript-plugin/packed-consumer.ts", "packed-consumer.before.ts.gz"],
  [
    "editors/vscode/test/suite/typescript-provider-host.cjs",
    "typescript-provider-host.before.cjs.gz",
  ],
  [
    "editors/vscode/test/run-typescript-provider-host.mjs",
    "run-typescript-provider-host.before.mjs.gz",
  ],
  [".github/workflows/language-project-fixtures.yml", "language-project-fixtures.before.yml.gz"],
  [".github/workflows/typescript-plugin.yml", "typescript-plugin.before.yml.gz"],
  [".github/workflows/typescript-provider-vsix.yml", "typescript-provider-vsix.before.yml.gz"],
  ["tests/typescript-plugin/classifications.test.cjs", "classifications.before.cjs.gz"],
] as const;
const consumers = [
  "tests/typescript-plugin/classification-probe.cjs",
  "tests/typescript-plugin/classification-consumer.ts",
  "tests/typescript-plugin/classifications.test.cjs",
  "editors/vscode/test/suite/classification-provider.cjs",
  "tests/tooling/language-project-classification-runtime.test.ts",
  "tests/tooling/language-project-classification-position.test.ts",
  "tests/tooling/language-project-classification-records.test.ts",
  "tools/support/compat/github/language-project-classification-successor.ts",
  "tools/support/compat/github/language-project-fixtures.ts",
  "npm/typescript-plugin/language-service.cjs",
];
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
function requireCustody(value: unknown, message: string): asserts value {
  if (!value) throw new Error(`invalid classification successor runtime: ${message}`);
}
function object(value: unknown, keys: string[]): Record<string, any> {
  requireCustody(value && typeof value === "object" && !Array.isArray(value), "object required");
  requireCustody(
    JSON.stringify(Object.keys(value).sort()) === JSON.stringify([...keys].sort()),
    "closed fields differ",
  );
  return value as Record<string, any>;
}
function pinned(read: ReadFile, raw: unknown, path: string): Buffer {
  const row = object(raw, ["path", "sha256", "bytes"]);
  requireCustody(row.path === path, "closure path differs");
  const body = read("head", path);
  requireCustody(
    body && body.length === row.bytes && digest(body) === row.sha256,
    `unreviewed producer registration (missing or changed closure body): ${path}`,
  );
  return body;
}

/** Authenticate all live consumers before exposing any archived historical bytes. */
export function authenticateClassificationRuntime(read: ReadFile): Map<string, Buffer> {
  const custody = read("head", CLASSIFICATION_RUNTIME_ROOT + "custody.json");
  requireCustody(custody && digest(custody) === custodyHash, "custody missing or changed");
  return validateClassificationRuntimeRecord(read, JSON.parse(custody.toString("utf8")));
}

/** Schema and body checks after authenticateClassificationRuntime verifies exact custody bytes. */
export function validateClassificationRuntimeRecord(
  rawRead: ReadFile,
  value: unknown,
): Map<string, Buffer> {
  const read = historicalCurrentLanguageProjectSources(rawRead);
  const record = object(value, [
    "schema",
    "id",
    "predecessor",
    "transitions",
    "input",
    "consumers",
  ]);
  requireCustody(
    record.schema === "vize.classificationRuntimeSuccessor.v1" &&
      record.id === CLASSIFICATION_RUNTIME_ID,
    "unregistered successor",
  );
  const predecessor = pinned(read, record.predecessor, predecessorPath);
  requireCustody(digest(predecessor) === predecessorHash, "predecessor changed");
  requireCustody(
    Array.isArray(record.transitions) && record.transitions.length === transitions.length,
    "complete transition closure required",
  );
  const originals = new Map<string, Buffer>();
  for (const [index, [path, gzip]] of transitions.entries()) {
    const row = object(record.transitions[index], ["before", "after", "gzip"]);
    const before = object(row.before, ["path", "sha256", "bytes"]);
    requireCustody(
      before.path === path && Number.isSafeInteger(before.bytes) && before.bytes > 0,
      "historical path differs",
    );
    const compressed = pinned(read, row.gzip, CLASSIFICATION_RUNTIME_ROOT + gzip);
    const original = gunzipSync(compressed, { maxOutputLength: before.bytes });
    requireCustody(
      original.length === before.bytes && digest(original) === before.sha256,
      "historical bytes differ",
    );
    pinned(read, row.after, path);
    originals.set(path, original);
  }
  pinned(read, record.input, CLASSIFICATION_RUNTIME_ROOT + "input.json");
  requireCustody(
    Array.isArray(record.consumers) && record.consumers.length === consumers.length,
    "complete current consumer closure required",
  );
  for (const [index, path] of consumers.entries()) pinned(read, record.consumers[index], path);
  return originals;
}

/** Base reads remain actual; only seven exact, authenticated head transitions have a historical view. */
export function historicalClassificationConsumers(rawRead: ReadFile): ReadFile {
  const read = historicalCurrentLanguageProjectSources(rawRead);
  let originals: Map<string, Buffer> | undefined;
  return (side, path) => {
    const body = read(side, path);
    if (side === "base" || !transitions.some(([source]) => source === path)) return body;
    // An older checkout may predate this successor. The frozen before identity,
    // rather than the presence of metadata, determines that finite legacy case.
    const expected = historicalHashes.get(path)!;
    if (body && digest(body) === expected) return body;
    originals ??= authenticateClassificationRuntime(read);
    return originals.get(path)!;
  };
}

const historicalHashes = new Map<string, string>([
  [
    "tests/typescript-plugin/classifications.test.cjs",
    "3767491fc2e414aca6635646b40b1e780509b93492ccf3aa13729b9d176a9efa",
  ],
  [
    ".github/workflows/typescript-plugin.yml",
    "3c9ed480a8ff672eaca469a3de1304d44c76425a7c55c76279abfb0e3d34303b",
  ],
  [
    ".github/workflows/typescript-provider-vsix.yml",
    "77c6d8a9aaed31dc5af2069b6d3ccc7b85ffefcd1d55faaeae7621981f59c744",
  ],
  [
    "tests/typescript-plugin/packed-consumer.ts",
    "7a1d073b558da95a4b3491b5b9599618209b13861f22e73d323c9e70663ef8d5",
  ],
  [
    "editors/vscode/test/suite/typescript-provider-host.cjs",
    "f3b0e86688733e8867f1ed67ef8e33dfe467ad4d3cc695c091da2768bf9de3ec",
  ],
  [
    "editors/vscode/test/run-typescript-provider-host.mjs",
    "df51b3db29ba4452a6585fe0e6ddf71db69d8f9b53611bbd93a79d4b049f3229",
  ],
  [
    ".github/workflows/language-project-fixtures.yml",
    "f36d936ba5b9b4bfaa48f148150710fb0c35e692ebfc6efb05f5fc1abb46efc6",
  ],
]);
