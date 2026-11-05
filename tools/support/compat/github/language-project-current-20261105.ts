import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
type Identity = { path: string; bytes: number; sha256: string };
type Record = {
  schema: string;
  qualificationCredit: boolean;
  transitions: Array<{
    path: string;
    before: Omit<Identity, "path">;
    after: Omit<Identity, "path">;
    gzip: Identity;
  }>;
  registration: { beforeSha256: string; afterSha256: string; archive: Identity };
};
type PluginSourceSuccessor = {
  schema: string;
  qualificationCredit: boolean;
  predecessor: Identity;
  transitions: Array<{
    path: string;
    before: Omit<Identity, "path">;
    after: Omit<Identity, "path">;
    gzip: Identity;
    edits: Array<{ before: string; after: string }>;
  }>;
};
export const CURRENT_LANGUAGE_PROJECT_ROOT =
  "tests/_fixtures/tooling/language-project-current-20261105/";
const custodySha256 = "6adafa56360823c8f6d39f5f3035c45e482ab7f9d06f6800b4bc50c6bb125b4d";
const pluginSuccessorSha256 = "1d6f0b13fab4363b84483a9911fc99968248e71806f5ea51836f29779be520e2";
const currentPluginSources = [
  {
    path: "npm/typescript-plugin/index.cjs",
    sha256: "97b1fbe39f60b72339257dee8a25d8c96d8b7869bab81cc4e76b79716df5a062",
    edits: 1,
  },
  {
    path: "npm/typescript-plugin/source-documents.cjs",
    sha256: "7ae5c061ecc432edc33fd92aa16e95b52abc28845a824a1bed70777639f4f293",
    edits: 2,
  },
  {
    path: "tests/tooling/language-project-classification-runtime.test.ts",
    sha256: "b8b7010ee6278f46ede52852cc157fa809233fd5841ebe7a04502baa3833c65b",
    edits: 1,
  },
];
const paths = [
  "tools/support/compat/github/language-project-fixtures.ts",
  "npm/typescript-plugin/language-service.cjs",
  "npm/typescript-plugin/virtual-modules.cjs",
  "tools/support/compat/github/language-project-classification-successor.ts",
];
const currentHashes = new Set([
  "2cac735af395f227853ea0c7aa122d828c40ed39531e36d3f63edfa93215f8a5",
  "b708919c50259a5e66758fc7f4b9b3c16e0c68e02b100e4768f27286e623da75",
  "74f82990fb1da4c17f80a0b51058b584f3f7a7991600501b48d5e19d5f8e0f8f",
  "f2cf0f5aa4b79a4b65df9a2d166f524c4ccb0c2673e3cc8351a726e2359ebc07",
]);
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
function requireCurrent(condition: unknown): asserts condition {
  if (!condition)
    throw new Error("invalid classification successor: current source custody differs");
}
function record(read: ReadFile): Record {
  const body = read("head", CURRENT_LANGUAGE_PROJECT_ROOT + "custody.json");
  requireCurrent(body && digest(body) === custodySha256);
  const value: Record = JSON.parse(body.toString("utf8"));
  requireCurrent(value.schema === "vize.languageProjectCurrentSources.v1");
  requireCurrent(value.qualificationCredit === false);
  requireCurrent(
    JSON.stringify(value.transitions.map((row) => row.path)) === JSON.stringify(paths),
  );
  return value;
}
function pinned(read: ReadFile, identity: Identity): Buffer {
  const body = read("head", identity.path);
  requireCurrent(body && body.length === identity.bytes && digest(body) === identity.sha256);
  return body;
}

/** The lazy SFC successor authenticates both complete source directions; it grants no parity. */
function historicalCurrentPluginSource(read: ReadFile, path: string, current: Buffer): Buffer {
  const body = read("head", CURRENT_LANGUAGE_PROJECT_ROOT + "plugin-source-successor.json");
  requireCurrent(body && digest(body) === pluginSuccessorSha256);
  const value: PluginSourceSuccessor = JSON.parse(body.toString("utf8"));
  requireCurrent(value.schema === "vize.languageProjectCurrentSourceSuccessor.v1");
  requireCurrent(value.qualificationCredit === false);
  requireCurrent(
    value.predecessor.path === CURRENT_LANGUAGE_PROJECT_ROOT + "custody.json" &&
      value.predecessor.sha256 === custodySha256,
  );
  pinned(read, value.predecessor);
  requireCurrent(
    JSON.stringify(value.transitions.map((entry) => entry.path)) ===
      JSON.stringify(currentPluginSources.map((entry) => entry.path)),
  );
  const entry = value.transitions.find((entry) => entry.path === path)!;
  requireCurrent(current.length === entry.after.bytes && digest(current) === entry.after.sha256);
  const original = gunzipSync(pinned(read, entry.gzip), { maxOutputLength: entry.before.bytes });
  requireCurrent(
    original.length === entry.before.bytes && digest(original) === entry.before.sha256,
  );
  requireCurrent(
    Array.isArray(entry.edits) &&
      entry.edits.length === currentPluginSources.find((entry) => entry.path === path)!.edits,
  );
  let forward = original.toString("utf8");
  for (const edit of entry.edits) {
    requireCurrent(
      edit.before.length > 0 &&
        edit.after.length > 0 &&
        forward.split(edit.before).length === 2 &&
        forward.split(edit.after).length === 1,
    );
    forward = forward.replace(edit.before, edit.after);
  }
  requireCurrent(Buffer.from(forward).equals(current));
  let inverse = current.toString("utf8");
  for (const edit of [...entry.edits].reverse()) {
    requireCurrent(
      inverse.split(edit.after).length === 2 && inverse.split(edit.before).length === 1,
    );
    inverse = inverse.replace(edit.after, edit.before);
  }
  requireCurrent(Buffer.from(inverse).equals(original));
  return original;
}

/** Authenticate complete current sources before replaying the frozen validation chain. */
export function historicalCurrentLanguageProjectSources(read: ReadFile): ReadFile {
  let custody: Record | undefined;
  return (side, path) => {
    const body = read(side, path);
    const plugin = currentPluginSources.find((entry) => entry.path === path);
    if (side === "head" && body && plugin && digest(body) === plugin.sha256)
      return historicalCurrentPluginSource(read, path, body);
    if (side !== "head" || !body || !paths.includes(path) || !currentHashes.has(digest(body)))
      return body;
    custody ??= record(read);
    const entry = custody.transitions.find((row) => row.path === path)!;
    // Unknown bytes remain actual bytes, so ordinary predecessor validation refuses them.
    if (body.length !== entry.after.bytes || digest(body) !== entry.after.sha256) return body;
    const original = gunzipSync(pinned(read, entry.gzip), { maxOutputLength: entry.before.bytes });
    requireCurrent(
      original.length === entry.before.bytes && digest(original) === entry.before.sha256,
    );
    return original;
  };
}

/** Keep the original registration intact; accept only the one complete current row. */
export function historicalClassificationRegistration(read: ReadFile, row: unknown): unknown {
  if (
    digest(Buffer.from(JSON.stringify(row))) !==
    "0ef670074ced17b07b43882c43ddbacb75aace2488b85856f7f1c84975f56e49"
  )
    return row;
  const custody = record(read);
  const original = pinned(read, custody.registration.archive);
  requireCurrent(digest(original) === custody.registration.beforeSha256);
  return JSON.parse(original.toString("utf8"));
}
