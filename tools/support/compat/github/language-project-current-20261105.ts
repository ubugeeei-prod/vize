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
export const CURRENT_LANGUAGE_PROJECT_ROOT =
  "tests/_fixtures/tooling/language-project-current-20261105/";
const custodySha256 = "6adafa56360823c8f6d39f5f3035c45e482ab7f9d06f6800b4bc50c6bb125b4d";
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

/** Authenticate complete current sources before replaying the frozen validation chain. */
export function historicalCurrentLanguageProjectSources(read: ReadFile): ReadFile {
  let custody: Record | undefined;
  return (side, path) => {
    const body = read(side, path);
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
